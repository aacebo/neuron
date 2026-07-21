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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("executor=info")))
        .init();

    let config = Config::from_env();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to create pool");

    let socket = amqp::new(&config.rabbitmq_url)
        .with_app_id("neuron::executor")
        .with_queue("actor.create".parse()?)
        .connect()
        .await
        .expect("Failed to connect to AMQP");

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

        let actor_id = match &event.data {
            types::events::Data::Actor { actor } => Some(actor.id),
            _ => None,
        };

        let span = tracing::info_span!(
            "event.delivery",
            event_key = %event.key,
            event_id = %event.id,
            trace_id = %event.trace_id,
            actor_id = tracing::field::Empty,
        );

        if let Some(actor_id) = actor_id {
            span.record("actor_id", tracing::field::display(actor_id));
        }

        async {
            tracing::trace!("received event delivery");

            let ctx = EventContext::new(&ctx, &delivery, &event);
            let result = match &event.data {
                types::events::Data::Actor { actor } => events::actor::on_create(ctx, actor).await,
                _ => {
                    tracing::warn!("actor.create event did not contain actor data");
                    ctx.ack().await.map_err(|error| Box::new(error) as Box<dyn std::error::Error>)
                }
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
