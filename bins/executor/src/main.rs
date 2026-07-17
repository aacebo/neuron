use sqlx::postgres::PgPoolOptions;

mod config;
mod context;
mod events;

pub use config::Config;
pub use context::Context;

use crate::context::EventContext;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to create pool");

    let socket = amqp::new(&config.rabbitmq_url)
        .with_app_id("neuron::executor")
        .with_queue(amqp::Key::new("message", amqp::Action::Create))
        .with_queue(amqp::Key::new("job", amqp::Action::Create))
        .with_queue(amqp::Key::new("log", amqp::Action::Create))
        .connect()
        .await
        .expect("Failed to connect to AMQP");

    let ctx = Context::new(&pool, &socket);
    let mut message_consumer = socket.consume(amqp::Key::new("message", amqp::Action::Create)).await?;
    let mut job_consumer = socket.consume(amqp::Key::new("job", amqp::Action::Create)).await?;
    let mut log_consumer = socket.consume(amqp::Key::new("log", amqp::Action::Create)).await?;

    println!("waiting for events...");

    tokio::try_join!(
        async {
            while let Some(res) = message_consumer.dequeue::<storage::types::Message>().await {
                let (delivery, event) = res?;
                events::message::on_create(EventContext::new(&ctx, &delivery, &event)).await?;
            }

            Ok::<_, Box<dyn std::error::Error>>(())
        },
        async {
            while let Some(res) = job_consumer.dequeue::<storage::types::Job>().await {
                let (delivery, event) = res?;
                events::job::on_attempt(EventContext::new(&ctx, &delivery, &event)).await?;
            }

            Ok::<_, Box<dyn std::error::Error>>(())
        },
        async {
            while let Some(res) = log_consumer.dequeue::<storage::types::Log>().await {
                let (delivery, event) = res?;
                events::log::on_create(EventContext::new(&ctx, &delivery, &event)).await?;
            }

            Ok::<_, Box<dyn std::error::Error>>(())
        },
    )?;

    Ok(())
}
