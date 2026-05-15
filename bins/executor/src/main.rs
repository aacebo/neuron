mod config;

use amqp::{Action, Key};

use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env();
    let socket = amqp::new(&config.rabbitmq_url)
        .with_app_id("sigmod[worker]")
        .with_queue(Key::new("workspace", Action::Create))
        .connect()
        .await?;

    let mut consumer = socket
        .consume(Key::new("workspace", Action::Create))
        .await?;

    println!("waiting for messages on workspace.create...");

    while let Some(res) = consumer.dequeue::<String>().await {
        let _ = match res {
            Err(err) => return Err(err.into()),
            Ok(v) => v,
        };
    }

    Ok(())
}
