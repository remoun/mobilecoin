// Copyright (c) 2018-2022 The MobileCoin Foundation

//! Mock BlockStream

use crate::{BlockData, BlockStream, Result};
use futures::Stream;
use mc_blockchain_test_utils::get_blocks;
use mc_blockchain_types::BlockVersion;
use mc_util_test_helper::get_seeded_rng;

/// Mock implementation of BlockStream, backed by pre-defined data.
#[derive(Clone, Debug)]
pub struct MockStream {
    items: Vec<Result<BlockData>>,
}

impl MockStream {
    /// Instantiate a MockStream with the given items.
    /// A subset of the items will be cloned for each `get_block_stream` call.
    pub fn new(items: Vec<Result<BlockData>>) -> Self {
        Self { items }
    }

    /// Instantiate a MockStream with the given blocks.
    pub fn from_blocks(src: Vec<BlockData>) -> Self {
        let items: Vec<Result<BlockData>> = src.into_iter().map(Ok).collect();
        Self::new(items)
    }

    /// Create a mock stream with blocks that resemble those seen in productoin
    ///
    /// * `num_blocks`: total number of simulated blocks to create.
    /// * `num_recipients`: number of randomly generated recipients.
    /// * `num_tokens`: number of distinct token ids per block.
    /// * `num_tx_outs_per_recipient_per_token_per_block`: number of outputs for
    ///   each token ID per recipient per block.
    ///
    /// Returns a MockStream that when driven will produce blocks with the
    /// contents specified in the above parameters
    pub fn with_custom_block_contents(
        num_blocks: usize,
        num_recipients: usize,
        num_tokens: usize,
        num_tx_outs_per_recipient_per_token_per_block: usize,
    ) -> MockStream {
        MockStream::from_blocks(get_blocks(
            BlockVersion::MAX,
            num_blocks,
            num_recipients,
            num_tokens as u64,
            num_tx_outs_per_recipient_per_token_per_block,
            1 << 20,
            None,
            &mut get_seeded_rng(),
        ))
    }
}

impl BlockStream for MockStream {
    type Stream<'s> = impl Stream<Item = Result<BlockData>> + 's;

    fn get_block_stream(&self, starting_height: u64) -> Result<Self::Stream<'_>> {
        let start_index = starting_height as usize;
        let items = self.items.iter().cloned().skip(start_index);
        Ok(futures::stream::iter(items))
    }
}
