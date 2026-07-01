use rust_bert::pipelines::sentiment;

use crate::{Action, types};

pub struct Sentiment<'a> {
    model: &'a sentiment::SentimentModel,
}

impl<'a> Sentiment<'a> {
    pub fn new(model: &'a sentiment::SentimentModel) -> Self {
        Self { model }
    }
}

impl<'a> Action for Sentiment<'a> {
    fn name(&self) -> &'static str {
        "sentiment"
    }

    fn invoke(&self, ctx: &mut crate::Context) -> Result<(), Box<dyn std::error::Error>> {
        let out = self.model.predict(ctx.input);

        for (i, sentiment) in out
            .into_iter()
            .filter(|v| v.score as f32 >= ctx.options.min_score)
            .enumerate()
        {
            let polarity = match sentiment.polarity {
                sentiment::SentimentPolarity::Negative => "negative",
                sentiment::SentimentPolarity::Positive => "positive",
            };

            ctx.annotations.push(types::CortexAnnotation {
                r#type: String::from("sentiment"),
                label: polarity.to_string(),
                text: polarity.to_string(),
                score: sentiment.score,
                spans: vec![types::Span::new(0, ctx.input[i].len() as u32)],
            });
        }

        Ok(())
    }
}
