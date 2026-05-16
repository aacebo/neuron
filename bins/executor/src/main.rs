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
        .connect()
        .await
        .expect("Failed to connect to AMQP");

    let ctx = Context::new(&pool, &socket);
    let mut consumer = socket
        .consume(amqp::Key::new("message", amqp::Action::Create))
        .await?;

    println!("waiting for events...");

    while let Some(res) = consumer.dequeue::<storage::types::Message>().await {
        let (delivery, event) = match res {
            Err(err) => return Err(err.into()),
            Ok(v) => v,
        };

        events::message::on_create(EventContext::new(&ctx, &delivery, &event)).await?;
    }

    Ok(())
}
