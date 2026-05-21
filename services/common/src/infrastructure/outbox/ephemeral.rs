use async_nats::jetstream::{
    self,
    consumer::{pull::Config, DeliverPolicy, PullConsumer},
};

use crate::infrastructure::outbox::stream::{ensure_stream, OutboxStreamConfig};

/// Create an ephemeral pull consumer that delivers only events produced **after**
/// the consumer attaches. Used by realtime fan-out where missed events are not
/// recoverable (the WebSocket client would not be able to replay them anyway).
///
/// The consumer auto-cleans when the connection drops; no durable state is kept
/// in JetStream. `filter_subject` accepts wildcards (e.g. `"chat.>"`).
pub async fn ephemeral_fanout_consumer(
    nats_url: &str,
    stream_config: &OutboxStreamConfig,
    filter_subject: &str,
) -> Result<PullConsumer, async_nats::Error> {
    let client = async_nats::connect(nats_url).await?;
    let jetstream = jetstream::new(client);
    let stream = ensure_stream(&jetstream, stream_config).await?;

    let consumer = stream
        .create_consumer(Config {
            deliver_policy: DeliverPolicy::New,
            filter_subject: filter_subject.to_string(),
            inactive_threshold: std::time::Duration::from_secs(60),
            ..Default::default()
        })
        .await?;

    Ok(consumer)
}
