use sqlx::postgres::PgPoolOptions;
use tracing::Instrument;
use tracing_subscriber::EnvFilter;

mod config;
mod context;
mod events;

pub use config::Config;
pub use context::Context;

use crate::context::EventContext;

#[tokio::main]
async fn main() -> ::error::Result<()> {
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("worker=info")))
        .init();

    let config = Config::from_env()?;
    let pool = PgPoolOptions::new().max_connections(5).connect(&config.database_url).await?;
    let socket = amqp::new(&config.rabbitmq_url)
        .with_app_id("neuron::worker")
        .with_queue("actor.create".parse()?)
        .with_queue("actor.update".parse()?)
        .with_queue("message.create".parse()?)
        .with_queue("message.inbound".parse()?)
        .connect()
        .await?;
    let mut consumer = socket.consume("*.*").await?;

    tracing::info!("listening...");

    while let Some(result) = consumer.dequeue().await {
        let (delivery, event) = match result {
            Ok(delivery) => delivery,
            Err(error) => {
                tracing::error!(%error, "failed to consume event");
                continue;
            }
        };

        let span = tracing::info_span!(
            "event.delivery",
            event_key = %event.key,
            event_id = %event.id,
            trace_id = %event.trace_id,
        );

        let ctx = Context::new(&pool, span, &socket);

        async {
            tracing::trace!("received event delivery");
            let ctx = EventContext::new(&ctx, &delivery, &event);

            if let Err(error) = events::run(&ctx).await {
                tracing::error!(%error, "failed to settle event delivery");
            }
        }
        .instrument(ctx.span().clone())
        .await;
    }

    Ok(())
}
