use amqp::{Action, Key};
use sqlx::postgres::PgPoolOptions;

mod config;
mod context;

pub use config::Config;
pub use context::Context;

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
        .connect()
        .await
        .expect("Failed to connect to AMQP");

    let _ctx = Context::new(pool, socket.clone());
    let mut consumer = socket.consume(Key::new("message", Action::Create)).await?;

    println!("waiting for events...");

    while let Some(res) = consumer.dequeue::<String>().await {
        let _ = match res {
            Err(err) => return Err(err.into()),
            Ok(v) => v,
        };
    }

    Ok(())
}
