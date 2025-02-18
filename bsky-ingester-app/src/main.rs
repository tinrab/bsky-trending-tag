use async_ringbuf::{traits::*, AsyncHeapRb};
use atrium_api::{app::bsky::richtext::facet::MainFeaturesItem, record::KnownRecord, types::Union};
use jetstream_oxide::{
    events::{commit::CommitEvent, JetstreamEvent},
    DefaultJetstreamEndpoints, JetstreamCompression, JetstreamConfig, JetstreamConnector,
};
use log::{error, info};
use serde::Serialize;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

use bsky_ingester_app::{
    broker::NatsBroker,
    config::{AppConfig, BrokerConfig},
    shutdown::make_shutdown_signal,
};

const TAGS_BROKER_SUBJECT: &str = "tags";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TagMessage {
    value: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config = AppConfig::get();

    let broker = match &config.broker {
        BrokerConfig::Nats(config) => NatsBroker::connect(config).await?,
        BrokerConfig::Memory => todo!(),
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

    // Consume the bluesky's jetstream feed.
    let (mut tag_producer, mut tag_consumer) = AsyncHeapRb::<String>::new(1000).split();
    task_set.spawn(async move {
        let js = JetstreamConnector::new(JetstreamConfig {
            endpoint: DefaultJetstreamEndpoints::USEastOne.into(),
            wanted_collections: vec!["app.bsky.feed.post".parse().unwrap()],
            wanted_dids: Vec::new(),
            compression: JetstreamCompression::Zstd,
            cursor: None,
        })?;
        let js_receiver = js.connect().await?;

        while let Ok(event) = js_receiver.recv_async().await {
            // Get post records
            if let JetstreamEvent::Commit(CommitEvent::Create { commit, .. }) = event {
                if let KnownRecord::AppBskyFeedPost(record) = commit.record {
                    // Extract tags and send them to ringbuffer
                    for tag in record
                        .facets
                        .iter()
                        .flatten()
                        .flat_map(|x| x.features.iter())
                        .filter_map(|feature| match feature {
                            Union::Refs(MainFeaturesItem::Tag(tag)) => Some(tag),
                            _ => None,
                        })
                    {
                        let _ = tag_producer.try_push(tag.data.tag.clone());
                    }
                }
            }
        }

        anyhow::Ok(())
    });

    // Push tags to NATS broker.
    task_set.spawn(async move {
        while let Some(value) = tag_consumer.pop().await {
            broker
                .publish(
                    TAGS_BROKER_SUBJECT,
                    serde_json::to_vec(&TagMessage { value }).unwrap(),
                )
                .await?;
            // time::sleep(Duration::from_millis(1000)).await;
        }

        anyhow::Ok(())
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
