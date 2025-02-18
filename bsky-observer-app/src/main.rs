use async_ringbuf::{traits::*, AsyncHeapRb};
use futures::StreamExt;
use log::{error, info};
use serde::Deserialize;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use bsky_observer_app::{
    broker::NatsBroker,
    cache::RedisCache,
    config::{AppConfig, BrokerConfig, CacheConfig},
    shutdown::make_shutdown_signal,
};

const TAG_TRENDS_BROKER_SUBJECT: &str = "tagtrends";
const TAG_RANK_CACHE_KEY: &str = "tagrank";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TagTrendMessage {
    value: String,
    rank: u32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config = AppConfig::get();

    let broker = match &config.broker {
        BrokerConfig::Nats(config) => NatsBroker::connect(config).await?,
        BrokerConfig::Memory => todo!(),
    };
    let cache = match &config.cache {
        CacheConfig::Redis(config) => RedisCache::connect(config).await?,
        CacheConfig::Memory => todo!(),
    };

    let mut task_set = JoinSet::new();

    // Set up task cancellation.
    let cancel = CancellationToken::new();
    task_set.spawn({
        let cancel = cancel.clone();
        async move {
            make_shutdown_signal().await;
            info!("ctrl+c received, exiting...");
            cancel.cancel();

            anyhow::Ok(())
        }
    });

    // Subscribe to tag trends.
    let (mut tag_producer, mut tag_consumer) = AsyncHeapRb::<TagTrendMessage>::new(1000).split();
    task_set.spawn(async move {
        let mut subscriber = broker.subscribe(TAG_TRENDS_BROKER_SUBJECT).await?;

        while let Some(message) = subscriber.next().await {
            match serde_json::from_slice::<TagTrendMessage>(&message.payload) {
                Ok(message) => {
                    let _ = tag_producer.try_push(message);
                }
                Err(err) => {
                    error!("failed to deserialize message: {:?}", err);
                }
            }
        }

        anyhow::Ok(())
    });

    // Update tag ranks in Redis cache.
    task_set.spawn({
        let cache = cache.clone();
        async move {
            while let Some(value) = tag_consumer.pop().await {
                cache
                    .hset(TAG_RANK_CACHE_KEY, &value.rank.to_string(), &value.value)
                    .await?;
            }

            anyhow::Ok(())
        }
    });

    if let Some(result) = task_set.join_next().await {
        match result {
            Ok(Ok(_)) => {
                // Stopped
            }
            Ok(Err(err)) => {
                error!("{:?}", err);
            }
            Err(err) => {
                error!("task panicked: {:?}", err);
            }
        }
    }

    info!("exit");

    Ok(())
}
