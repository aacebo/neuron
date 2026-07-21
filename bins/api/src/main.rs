use actix_web::{App, HttpServer, web};
use sqlx::postgres::PgPoolOptions;

mod config;
mod context;
mod error;
mod routes;

pub use config::Config;
pub use context::*;
pub use error::*;

#[actix_web::main]
async fn main() -> ::error::Result<()> {
    let config = Config::from_env()?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .map_err(storage::Error::from)?;

    sqlx::migrate!("../../crates/storage/migrations")
        .run(&pool)
        .await
        .map_err(Error::server)?;

    let socket = amqp::new(&config.rabbitmq_url)
        .with_app_id("neuron::api")
        .with_queue("message.create".parse()?)
        .connect()
        .await?;

    let ctx = Context::new(pool, socket);
    println!("Starting server at http://0.0.0.0:{}", config.port);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(ctx.clone()))
            .wrap(RequestContextMiddleware)
            .service(routes::index::get)
            .service(routes::agents::connect)
            .service(routes::agents::create)
        // .service(routes::console::get)
        // .service(routes::console::get_run)
        // .service(routes::chats::messages::create)
        // .service(routes::messages::get)
        // .service(routes::messages::get_events)
    })
    .bind(("0.0.0.0", config.port))
    .map_err(Error::server)?
    .run()
    .await
    .map_err(Error::server)?;

    Ok(())
}
