use rust_bert::pipelines::sentiment;

use crate::{CortexError, CortexInput, CortexOutput, Routine, types};

pub struct Sentiment<'a> {
    model: &'a sentiment::SentimentModel,
}

impl<'a> Sentiment<'a> {
    pub fn new(model: &'a sentiment::SentimentModel) -> Self {
        Self { model }
    }
}

impl<'a> Routine for Sentiment<'a> {
    fn name(&self) -> &'static str {
        "sentiment"
    }

    fn invoke(&self, input: CortexInput<'_>) -> Result<CortexOutput, CortexError> {
        let out = self.model.predict(input.text);
        let mut output = CortexOutput::default();

        for (i, sentiment) in out.into_iter().enumerate() {
            let polarity = match sentiment.polarity {
                sentiment::SentimentPolarity::Negative => "negative",
                sentiment::SentimentPolarity::Positive => "positive",
            };

            output.annotations.push(types::Annotation {
                r#type: String::from("sentiment"),
                label: polarity.to_string(),
                text: polarity.to_string(),
                score: sentiment.score,
                spans: vec![types::Span::new(0, input.text[i].len() as u32)],
            });
        }

        Ok(output)
    }
}
