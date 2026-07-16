use rust_bert::pipelines::common::{ModelResource, ModelType, ONNXModelResources};
use rust_bert::pipelines::token_classification::{LabelAggregationOption, TokenClassificationConfig};
use rust_bert::resources::RemoteResource;
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
        .connect()
        .await
        .expect("Failed to connect to AMQP");

    let cortex = tokio::task::spawn_blocking(|| {
        cortex::CortexConfig::new()
            .with_sentence_embeddings(rust_bert::pipelines::sentence_embeddings::SentenceEmbeddingsConfig::from(
                rust_bert::pipelines::sentence_embeddings::SentenceEmbeddingsModelType::AllMiniLmL12V2,
            ))
            .with_entity_extraction(rust_bert::pipelines::token_classification::TokenClassificationConfig::default())
            .with_keyword_extraction(rust_bert::pipelines::keywords_extraction::KeywordExtractionConfig::default())
            .with_pii_extraction(TokenClassificationConfig::new(
                ModelType::Bert,
                ModelResource::ONNX(ONNXModelResources {
                    encoder_resource: Some(Box::new(RemoteResource::new(
                        "https://huggingface.co/rtrigoso/bert-small-pii-detection-ONNX/resolve/main/onnx/model.onnx",
                        "bert-small-pii-detection",
                    ))),
                    ..Default::default()
                }),
                RemoteResource::new(
                    "https://huggingface.co/rtrigoso/bert-small-pii-detection-ONNX/resolve/main/config.json",
                    "bert-small-pii-detection",
                ),
                RemoteResource::new(
                    "https://huggingface.co/rtrigoso/bert-small-pii-detection-ONNX/resolve/main/vocab.txt",
                    "bert-small-pii-detection",
                ),
                None::<RemoteResource>,
                false,
                None,
                None,
                LabelAggregationOption::Mode,
            ))
            .with_sentiment(rust_bert::pipelines::sentiment::SentimentConfig::default())
            .with_summarization({
                let mut config = rust_bert::pipelines::summarization::SummarizationConfig::default();
                config.min_length = 8;
                config.max_length = Some(64);
                config
            })
            .build()
    })
    .await?
    .expect("Failed to spawn cortex build task");

    let ctx = Context::new(&pool, &socket, &cortex);
    let mut message_consumer = socket.consume(amqp::Key::new("message", amqp::Action::Create)).await?;
    let mut job_consumer = socket.consume(amqp::Key::new("job", amqp::Action::Create)).await?;

    println!("waiting for events...");

    tokio::try_join!(
        async {
            while let Some(res) = message_consumer.dequeue::<storage::types::Message>().await {
                let (delivery, event) = res.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
                events::message::on_create(EventContext::new(&ctx, &delivery, &event)).await?;
            }
            Ok::<_, Box<dyn std::error::Error>>(())
        },
        async {
            while let Some(res) = job_consumer.dequeue::<storage::types::Job>().await {
                let (delivery, event) = res.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
                events::job::on_attempt(EventContext::new(&ctx, &delivery, &event)).await?;
            }
            Ok::<_, Box<dyn std::error::Error>>(())
        },
    )?;

    Ok(())
}
