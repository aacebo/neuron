use actix_web::{HttpResponse, post};
use rust_bert::pipelines::keywords_extraction;
use rust_bert::pipelines::ner;
use rust_bert::pipelines::sentiment;
use rust_bert::pipelines::summarization;
// use rust_bert::pipelines::sentence_embeddings as embeddings;
use storage::types::Annotation;
use storage::types::Message;
use storage::types::Span;

use crate::RequestContext;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CreateMessage {
    pub text: String,
}

#[post("/chats/{chat}/messages")]
pub async fn create(
    ctx: RequestContext,
    body: actix_web::web::Json<CreateMessage>,
) -> Result<HttpResponse, Box<dyn std::error::Error>> {
    let mut message = Message::new(&body.text);
    let model = ner::NERModel::new(Default::default())?;
    let out = model.predict(&[&message.text]);

    for entities in out {
        for entity in entities {
            message.annotations.push(Annotation {
                label: entity.label,
                text: entity.word,
                score: entity.score,
                spans: vec![Span::new(entity.offset.begin, entity.offset.end)],
            });
        }
    }

    let model = keywords_extraction::KeywordExtractionModel::new(Default::default())?;
    let out = model.predict(&[&message.text])?;

    for keywords in out {
        for keyword in keywords {
            message.annotations.push(Annotation {
                label: String::from("keyword"),
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

    let model = summarization::SummarizationModel::new(Default::default())?;
    let out = model.summarize(&[&message.text])?;

    for summary in out {
        message.annotations.push(Annotation {
            label: String::from("summary"),
            text: summary,
            score: 1.0,
            spans: vec![Span::new(0, message.text.len() as u32)],
        });
    }

    let model = sentiment::SentimentModel::new(Default::default())?;
    let out = model.predict(&[message.text.as_str()]);

    for sentiment in out {
        message.annotations.push(Annotation {
            label: String::from("sentiment"),
            text: match sentiment.polarity {
                sentiment::SentimentPolarity::Negative => "negative",
                sentiment::SentimentPolarity::Positive => "positive",
            }
            .to_string(),
            score: sentiment.score,
            spans: vec![Span::new(0, message.text.len() as u32)],
        });
    }

    Ok(HttpResponse::Created().json(message))
}
