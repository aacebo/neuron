use actix_web::{App, HttpServer, web};
use sqlx::postgres::PgPoolOptions;

mod config;
mod context;
mod extract;
mod routes;

pub use config::{Config, ConsoleConfig};
pub use context::*;

#[actix_web::main]
async fn main() -> ::error::Result<()> {
    let config = Config::from_env()?;
    let pool = PgPoolOptions::new().max_connections(5).connect(&config.database_url).await?;

    sqlx::migrate!("../../crates/storage/migrations")
        .run(&pool)
        .await
        .map_err(error::sql)?;

    let socket = amqp::new(&config.rabbitmq_url)
        .with_app_id("neuron::broker")
        .connect()
        .await?;

    let events = config.console.enabled.then(|| tokio::sync::broadcast::channel(1024).0);
    let ctx = Context::new(pool, socket, config.console.clone(), events);
    let console_enabled = config.console.enabled;
    println!("Starting server at http://0.0.0.0:{}", config.port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(ctx.clone()))
            .wrap(RequestContextMiddleware)
            .service(routes::index::get)
            .service(routes::agents::connect)
            .service(routes::agents::create)
            .service(routes::messages::create)
            .configure(move |services| {
                if console_enabled {
                    routes::console::configure(services);
                }
            })
    })
    .bind(("0.0.0.0", config.port))?
    .run()
    .await?;

    Ok(())
}
