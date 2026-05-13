use actix_web::Error;
use actix_web::{HttpResponse, post};
use rust_bert::pipelines::keywords_extraction;
use rust_bert::pipelines::ner;
use rust_bert::pipelines::sentence_embeddings as embeddings;
use rust_bert::pipelines::sentiment;
use rust_bert::pipelines::summarization;
use storage::types::Span;
use storage::types::{Annotation, SummaryArtifact};
use storage::types::{EmbeddingArtifact, Message};

use crate::RequestContext;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CreateMessage {
    pub text: String,
}

#[post("/chats/{chat}/messages")]
pub async fn create(
    ctx: RequestContext,
    body: actix_web::web::Json<CreateMessage>,
) -> Result<HttpResponse, Error> {
    let result: Result<Message, String> = actix_web::web::block(move || {
        let mut message = Message::new(&body.text);
        let model = ner::NERModel::new(Default::default()).map_err(|err| err.to_string())?;
        let out = model.predict_full_entities(&[&message.text]);

        for entities in out {
            for entity in entities {
                message.annotations.push(Annotation {
                    r#type: String::from("entity"),
                    label: entity.label,
                    text: entity.word,
                    score: entity.score,
                    spans: vec![Span::new(entity.offset.begin, entity.offset.end)],
                });
            }
        }

        let model = keywords_extraction::KeywordExtractionModel::new(Default::default())
            .map_err(|err| err.to_string())?;
        let out = model
            .predict(&[&message.text])
            .map_err(|err| err.to_string())?;

        for keywords in out {
            for keyword in keywords {
                message.annotations.push(Annotation {
                    r#type: String::from("keyword"),
                    label: keyword.text.clone(),
                    text: keyword.text,
                    score: keyword.score as f64,
                    spans: keyword
                        .offsets
                        .iter()
                        .map(|offset| Span::new(offset.begin, offset.end))
                        .collect(),
                });
            }
        }

        let model = summarization::SummarizationModel::new(Default::default())
            .map_err(|err| err.to_string())?;
        let out = model
            .summarize(&[&message.text])
            .map_err(|err| err.to_string())?;

        for summary in out {
            message
                .artifacts
                .push(SummaryArtifact { text: summary }.into());
        }

        let model =
            sentiment::SentimentModel::new(Default::default()).map_err(|err| err.to_string())?;
        let out = model.predict([message.text.as_str()]);

        for sentiment in out {
            let polarity = match sentiment.polarity {
                sentiment::SentimentPolarity::Negative => "negative",
                sentiment::SentimentPolarity::Positive => "positive",
            };

            message.annotations.push(Annotation {
                r#type: String::from("sentiment"),
                label: polarity.to_string(),
                text: polarity.to_string(),
                score: sentiment.score,
                spans: vec![Span::new(0, message.text.len() as u32)],
            });
        }

        let model = embeddings::SentenceEmbeddingsBuilder::remote(
            embeddings::SentenceEmbeddingsModelType::AllMiniLmL12V2,
        )
        .create_model()
        .map_err(|err| err.to_string())?;

        let out = model
            .encode(&[&message.text])
            .map_err(|err| err.to_string())?;

        for vector in out {
            message.artifacts.push(EmbeddingArtifact { vector }.into());
        }

        Ok(message)
    })
    .await?;

    match result {
        Err(err) => Err(actix_web::error::ErrorInternalServerError(err)),
        Ok(mut message) => {
            message = match ctx.storage().messages().create(&message).await {
                Err(err) => return Err(actix_web::error::ErrorInternalServerError(err)),
                Ok(v) => v,
            };

            Ok(HttpResponse::Created().json(message))
        }
    }
}
