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
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("executor=info")))
        .init();

    let config = Config::from_env()?;
    let pool = PgPoolOptions::new().max_connections(5).connect(&config.database_url).await?;

    let socket = amqp::new(&config.rabbitmq_url)
        .with_app_id("neuron::executor")
        .with_queue("actor.create".parse()?)
        .connect()
        .await?;

    let ctx = Context::new(&pool, &socket);
    let mut consumer = socket.consume("actor.create").await?;

    tracing::info!(queue = "actor.create", "waiting for events");

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

        async {
            tracing::trace!("received event delivery");
            let ctx = EventContext::new(&ctx, &delivery, &event);
            let result = match &event.data {
                types::events::Data::Actor { actor } => match event.key.as_str() {
                    "actor.create" | "actor.update" => events::actor::on_event(ctx, actor).await,
                    _ => {
                        tracing::info!(?actor, "unsupported routing key");
                        Ok(())
                    }
                },
                _ => ctx.nack().await,
            };

            if let Err(error) = result {
                tracing::error!(%error, "failed to settle event delivery");
            }
        }
        .instrument(span)
        .await;
    }

    Ok(())
}
