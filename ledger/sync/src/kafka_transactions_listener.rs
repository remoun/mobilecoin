// Copyright (c) 2018-2022 The MobileCoin Foundation

//! Listener for transactions from validator nodes via Kafka.

use displaydoc::Display;
use futures::{channel::oneshot, future::FutureExt, stream::StreamExt};
use mc_common::{
    logger::{log, Logger},
    lru::LruCache,
    trace_time, ResponderId,
};
use mc_transaction_core::{Block, BlockData, BlockIndex};
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    error::KafkaError,
    message::BorrowedMessage,
    Message as KafkaMessage,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    convert::TryFrom,
    sync::{Arc, Mutex},
};
use url::Url;

/// Maximum number of pre-fetched blocks to keep in cache.
pub const MAX_PREFETCHED_BLOCKS: usize = 10000;

#[derive(Clone, Debug, Display)]
pub enum KafkaTransactionListenerError {
    /// Received an invalid block from {0}: {1}
    InvalidBlockReceived(String, String),

    /// Could not find this block in any of the Kafka topics we checked.
    NotFound,

    /// Kafka error on {0}
    Kafka(KafkaError),

    /// IO error on {0}: {1:?}
    IO(String, std::io::ErrorKind),

    /// Internal error
    Internal(String),
}

impl From<KafkaError> for KafkaTransactionListenerError {
    fn from(k: KafkaError) -> Self {
        KafkaTransactionListenerError::Kafka(k)
    }
}

impl<T> From<std::sync::PoisonError<T>> for KafkaTransactionListenerError {
    fn from(p: std::sync::PoisonError<T>) -> Self {
        KafkaTransactionListenerError::Internal(format!("lock poisoned {}", p))
    }
}

type KResult<T> = Result<T, KafkaTransactionListenerError>;

type SubscriberCache = HashMap<String, Subscriber>;

type BlockCache = LruCache<BlockIndex, BlockData>;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct KafkaSubscriberConfig {
    pub brokers: Vec<String>,
    pub topic: String,
}

impl TryFrom<&str> for KafkaSubscriberConfig {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let url = Url::parse(value).map_err(|x| format!("Failed to parse URL: {}", x))?;
        let addrs = url
            .socket_addrs(|| None)
            .map_err(|x| format!("Failed to parse URL: {}", x))?;
        let brokers: Vec<String> = addrs.iter().map(|s| s.to_string()).collect();
        let topic = url.path().trim_matches('/').to_string();
        Ok(KafkaSubscriberConfig { brokers, topic })
    }
}

impl TryFrom<&String> for KafkaSubscriberConfig {
    type Error = String;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        <Self as TryFrom<&str>>::try_from(value)
    }
}

pub struct KafkaTransactionsListener {
    subscriber_cache: Mutex<SubscriberCache>,
    block_cache: Arc<Mutex<BlockCache>>,
    logger: Logger,
}

impl KafkaTransactionsListener {
    pub fn new(sources: &[KafkaSubscriberConfig], logger: Logger) -> KResult<Self> {
        let subscriber_cache = Mutex::new(SubscriberCache::new());
        let block_cache = Arc::new(Mutex::new(BlockCache::new(MAX_PREFETCHED_BLOCKS)));
        let listener = Self {
            subscriber_cache,
            block_cache,
            logger,
        };
        for source in sources {
            listener.subscribe(source)?;
        }
        Ok(listener)
    }

    pub fn subscribe(&self, config: &KafkaSubscriberConfig) -> KResult<()> {
        let topic = config.topic.clone();
        let subscriber = Subscriber::new(config, self.block_cache.clone(), self.logger.clone())?;
        let mut cache = self.subscriber_cache.lock().expect("lock poisoned");
        cache.insert(topic, subscriber);
        Ok(())
    }

    pub fn get_cached_block(
        &self,
        _safe_responder_ids: &[ResponderId],
        block: &Block,
    ) -> Option<BlockData> {
        let mut cache = self.block_cache.lock().expect("lock poisoned");
        // Note: If this block index is in the cache, we take it out under the
        // assumption that our primary caller, LedgerSyncService, is not
        // going to try and fetch the same block twice if it managed to get
        // a valid block.
        cache.pop(&block.index)
    }
}

struct Subscriber {
    id: String,
    consumer: Arc<StreamConsumer>,
    block_cache: Arc<Mutex<BlockCache>>,
    stop_sender: Option<oneshot::Sender<bool>>,
    join_handle: Option<tokio::task::JoinHandle<()>>,
    logger: Logger,
}

impl Subscriber {
    pub fn new(
        config: &KafkaSubscriberConfig,
        block_cache: Arc<Mutex<BlockCache>>,
        logger: Logger,
    ) -> KResult<Self> {
        let pid = std::process::id();
        let id = format!("transaction-subscriber-{}-{}", config.topic, pid);
        let brokers = config.brokers.join(",");
        let consumer: Arc<StreamConsumer> = Arc::new(
            rdkafka::ClientConfig::new()
                .set("bootstrap.servers", &brokers)
                .set("group.id", &id)
                .set("enable.auto.commit", "true")
                .set("enable.partition.eof", "false")
                .set("session.timeout.ms", "6000")
                .create()?,
        );

        consumer.subscribe(&[&config.topic])?;

        let mut subscriber = Self {
            id,
            consumer,
            block_cache,
            stop_sender: None,
            join_handle: None,
            logger,
        };

        subscriber.start();

        Ok(subscriber)
    }

    pub fn stop(&mut self) {
        if let Some(stop_sender) = self.stop_sender.take() {
            stop_sender.send(true).unwrap_or_default();
        }
        if let Some(handle) = self.join_handle.take() {
            let _ = tokio::runtime::Handle::current().block_on(handle);
        }
    }

    fn start(&mut self) {
        self.stop();

        let (stop_sender, stop_receiver) = oneshot::channel::<bool>();
        self.stop_sender = Some(stop_sender);

        // Copy these pointers since spawn() requires 'static, so can't reference self
        // directly.
        let block_cache = self.block_cache.clone();
        let logger = self.logger.clone();
        let consumer = self.consumer.clone();
        let id = self.id.clone();
        self.join_handle = Some(tokio::spawn(async move {
            consumer
                .stream()
                .take_until(stop_receiver)
                .for_each_concurrent(None, |message| {
                    handle_message(message, id.clone(), block_cache.clone(), logger.clone())
                        .map(|_| ())
                })
                .await
        }));
    }
}

use mc_api::blockchain::ArchiveBlock;
use protobuf::Message as ProtoMessage;

async fn handle_message(
    message: Result<BorrowedMessage<'_>, KafkaError>,
    id: String,
    block_cache: Arc<Mutex<BlockCache>>,
    logger: Logger,
) -> KResult<()> {
    trace_time!(logger, "handle_message[{}]", &id);
    let message = message?;
    log::info!(
        logger,
        "got message with {}-byte key and {}-byte payload",
        message.key_len(),
        message.payload_len()
    );
    let bytes = message.payload();
    if bytes.is_none() {
        return Err(KafkaTransactionListenerError::IO(
            id,
            std::io::ErrorKind::InvalidData,
        ));
    }
    let archive_block = ArchiveBlock::parse_from_bytes(bytes.unwrap()).map_err(|err| {
        KafkaTransactionListenerError::InvalidBlockReceived(
            id.clone(),
            format!("protobuf parse failed: {:?}", err),
        )
    })?;

    let block_data = BlockData::try_from(&archive_block).map_err(|err| {
        KafkaTransactionListenerError::InvalidBlockReceived(id.clone(), err.to_string())
    })?;
    log::info!(
        logger,
        "parsed block from message with index {}, containing {} outputs",
        block_data.block().index,
        block_data.contents().outputs.len()
    );

    let mut cache = block_cache.lock()?;
    cache.put(block_data.block().index, block_data);

    Ok(())
}

impl Drop for Subscriber {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mc_common::logger::test_with_logger;

    #[test_with_logger]
    fn can_connect(logger: Logger) {
        let listener = KafkaTransactionsListener::new(&[], logger.clone()).unwrap();
        let block = Block::new_origin_block(&[]); // TODO
        let data = listener.get_cached_block(&[], &block);
        assert!(data.is_some());
    }
}
