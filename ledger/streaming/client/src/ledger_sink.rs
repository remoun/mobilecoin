// Copyright (c) 2018-2022 The MobileCoin Foundation

//! Creates a block sink stream factory

use futures::stream::{Stream, StreamExt};
use mc_common::logger::{log, Logger};
use mc_ledger_db::Ledger;
use mc_ledger_streaming_api::{BlockData, BlockIndex, Error, Result, Streamer};
use tokio::sync::mpsc::{channel, Receiver, Sender};

/// A block sink that takes blocks from a passed stream and puts them into
/// ledger db. This sink should live downstream from a verification source that
/// has already done block content, scp, and avr validation and thus is
/// considered a trusted stream
pub struct DbStream<US: Streamer<Result<BlockData>, BlockIndex> + 'static, L: Ledger + 'static> {
    /// Upstream block stream combinator
    upstream: US,

    /// Block database ledger object
    ledger: L,

    /// Pass through already synced blocks
    pass_through_synced_blocks: bool,

    /// Logger
    logger: Logger,
}

impl<US: Streamer<Result<BlockData>, BlockIndex> + 'static, L: Ledger + Clone + 'static>
    DbStream<US, L>
{
    /// Initialize a stream factory from an upstream source
    pub fn new(upstream: US, ledger: L, pass_through_synced_blocks: bool, logger: Logger) -> Self {
        Self {
            upstream,
            ledger,
            pass_through_synced_blocks,
            logger,
        }
    }
}

impl<US: Streamer<Result<BlockData>, BlockIndex> + 'static, L: Ledger + Clone + 'static>
    Streamer<Result<BlockData>, BlockIndex> for DbStream<US, L>
{
    type Stream<'s> = impl Stream<Item = Result<BlockData>> + 's;

    /// Get block stream that performs block sinking
    fn get_stream(&self, starting_height: BlockIndex) -> Result<Self::Stream<'_>> {
        let num_blocks = self.ledger.num_blocks()?;
        // Check to ensure we don't start more than 1 block ahead of what's currently in
        // the ledger (doing so will cause errors if we try to append a block)
        if num_blocks > 0 && starting_height > num_blocks {
            let err = Error::DBAccess(format!(
                "attempted to start at block {} but ledger height is {}",
                starting_height, num_blocks,
            ));
            log::error!(self.logger, "{:?}", err);
            return Err(err);
        }

        // If our local ledger is already ahead, but we still want to forward on blocks,
        // determine at which block we'll start syncing.
        let sync_start_height = if self.pass_through_synced_blocks && num_blocks > 0 {
            num_blocks
        } else {
            // If we're not okay with it, throw an error
            if starting_height < num_blocks {
                let err = Error::DBAccess(format!(
                    "ledger height is currently: {} attempted to start at: {}",
                    num_blocks, starting_height
                ));
                log::error!(self.logger, "{:?}", err);
                return Err(err);
            }

            starting_height
        };

        // Get the upstream, start our thread, and initialize our sink management object
        let (tx, rcv) = start_sink_thread(self.ledger.clone(), self.logger.clone());
        let state = SinkState::new(tx, rcv, 0, 0, sync_start_height, self.logger.clone());
        // Pin the stream since `StreamExt::next()` requires the stream to implement
        // Unpin, and we don't require it in our trait def.
        let stream = Box::pin(self.upstream.get_stream(starting_height)?);

        // Create the stream
        let output_stream =
            futures::stream::unfold((stream, state), |(mut stream, mut state)| async move {
                if let Some(result) = stream.next().await {
                    if let Ok(block_data) = result {
                        state.last_block_received = block_data.block().index;
                        if state.can_start_sync() {
                            // If we're above what's in the ledger, starting syncing the blocks
                            if state.sender.send(block_data).await.is_err() {
                                // TODO: Discuss whether thread error should stop stream or
                                // self-heal. It's possible to restart the upstream & thread here so
                                // that the downstream doesn't necessarily notice
                                log::error!(
                                    state.logger,
                                    "ledger sync thread stopped, aborting stream"
                                );
                                return None;
                            }
                        } else {
                            // Pass them through
                            return Some((Ok(block_data), (stream, state)));
                        }
                    } else {
                        return Some((result, (stream, state)));
                    }
                } else {
                    // If we're behind, wait for the rest of the blocks to sync then end
                    if state.is_behind() {
                        log::debug!(
                            state.logger,
                            "upstream terminated, waiting for the rest of the blocks to sync"
                        );
                    } else {
                        log::warn!(state.logger, "upstream stopped, ending stream");
                        return None;
                    }
                }
                if let Some(block_data) = state.receiver.recv().await {
                    state.last_block_synced = block_data.block().index;
                    Some((Ok(block_data), (stream, state)))
                } else {
                    // TODO: Discuss whether we want to heal the stream or not
                    log::error!(state.logger, "sink thread stopped, ending stream");
                    None
                }
            });
        Ok(Box::pin(output_stream))
    }
}

/// Object to manage the state of the ledger sink process
struct SinkState {
    /// Channel to send blocks ledger sink thread to be synced
    sender: Sender<BlockData>,

    /// Channel to receive blocks that have been synced
    receiver: Receiver<BlockData>,

    /// Last block we've received from the upstream
    last_block_received: u64,

    /// Last block we've synced into the ledger
    last_block_synced: u64,

    /// Block at which we'll start syncing upstream blocks into the ledger
    sync_start_height: u64,

    /// Logger
    logger: Logger,
}

impl SinkState {
    /// Create new manager for the block sink
    fn new(
        sender: Sender<BlockData>,
        receiver: Receiver<BlockData>,
        last_block_received: u64,
        last_block_synced: u64,
        sync_start_height: u64,
        logger: Logger,
    ) -> Self {
        SinkState {
            sender,
            receiver,
            last_block_received,
            last_block_synced,
            sync_start_height,
            logger,
        }
    }

    /// Determine whether blocks we've synced are behind blocks we've been sent
    fn is_behind(&self) -> bool {
        self.last_block_received > self.last_block_synced
    }

    /// Determine if we're able to begin syncing blocks to the ledger
    fn can_start_sync(&self) -> bool {
        self.last_block_received >= self.sync_start_height
    }
}

fn start_sink_thread(
    mut ledger: impl Ledger + 'static,
    logger: Logger,
) -> (Sender<BlockData>, Receiver<BlockData>) {
    // Initialize sending and receiving channels
    let (send_out, rcv_out) = channel::<BlockData>(10000);
    let (send_in, mut rcv_in) = channel::<BlockData>(10000);

    // Launch ledger sink thread
    std::thread::spawn(move || {
        while let Some(block_data) = rcv_in.blocking_recv() {
            // If there's an error syncing the blocks, end thread
            if let Err(err) = ledger.append_block(
                block_data.block(),
                block_data.contents(),
                block_data.signature().clone(),
            ) {
                log::error!(
                    logger,
                    "Error during attempt to write block {}: {}",
                    block_data.block().index,
                    err,
                );
                break;
            };

            // If message channels are broken, end thread
            if let Err(err) = send_out.try_send(block_data) {
                log::error!(
                    logger,
                    "sending block data to stream failed with error {:?}",
                    err
                );
                break;
            }
        }
    });
    (send_in, rcv_out)
}

#[cfg(test)]
mod tests {

    use super::*;
    use mc_common::logger::{test_with_logger, Logger};
    use mc_ledger_db::test_utils::get_mock_ledger;
    use mc_ledger_streaming_api::test_utils::{make_blocks, MockStream};

    #[test_with_logger]
    fn test_sink_from_start_block(logger: Logger) {
        let blocks = make_blocks(420);
        let upstream = MockStream::from_blocks(blocks);
        let dest_ledger = get_mock_ledger(0);
        let bs = DbStream::new(upstream, dest_ledger, true, logger);
        let mut go_stream = bs.get_stream(0).unwrap();

        futures::executor::block_on(async move {
            let mut count = 0;
            while let Some(block_data) = go_stream.next().await {
                assert_eq!(block_data.unwrap().block().index, count);
                count += 1;
            }

            assert_eq!(count, 420);
        });
    }

    #[test_with_logger]
    fn test_blocks_lower_than_stored_pass_through(logger: Logger) {
        let blocks = make_blocks(420);
        let upstream = MockStream::from_blocks(blocks);
        let dest_ledger = get_mock_ledger(42);
        let bs = DbStream::new(upstream, dest_ledger, true, logger);
        let mut stream = bs.get_stream(20).unwrap();

        futures::executor::block_on(async move {
            let mut count = 20;
            while let Some(block_data) = stream.next().await {
                assert_eq!(block_data.unwrap().block().index, count);
                count += 1;
            }

            assert_eq!(count, 420);
        });
    }

    #[test_with_logger]
    fn test_can_start_at_arbitrary_valid_height(logger: Logger) {
        let blocks = make_blocks(420);
        let upstream = MockStream::from_blocks(blocks);
        let dest_ledger = get_mock_ledger(42);
        let bs = DbStream::new(upstream, dest_ledger, true, logger);
        let mut stream = bs.get_stream(42).unwrap();

        futures::executor::block_on(async move {
            let mut count = 42;
            while let Some(block_data) = stream.next().await {
                assert_eq!(block_data.unwrap().block().index, count);
                count += 1;
            }

            assert_eq!(count, 420);
        });
    }

    #[test_with_logger]
    fn test_stream_creation_fails_if_requesting_blocks_above_ledger_height(logger: Logger) {
        let blocks = make_blocks(3);
        let upstream = MockStream::from_blocks(blocks);
        let dest_ledger = get_mock_ledger(1);
        let bs = DbStream::new(upstream, dest_ledger, true, logger);
        let stream = bs.get_stream(2);
        assert!(matches!(stream, Err(Error::DBAccess(_))));
    }
}