use rust_bert::pipelines::keywords_extraction;

use crate::{CortexError, CortexInput, CortexOutput, Routine, types};

pub struct KeywordExtraction<'a> {
    model: &'a keywords_extraction::KeywordExtractionModel<'a>,
}

impl<'a> KeywordExtraction<'a> {
    pub fn new(model: &'a keywords_extraction::KeywordExtractionModel) -> Self {
        Self { model }
    }
}

impl<'a> Routine for KeywordExtraction<'a> {
    fn name(&self) -> &'static str {
        "keyword-extraction"
    }

    fn invoke(&self, input: CortexInput<'_>) -> Result<CortexOutput, CortexError> {
        let out = self.model.predict(input.text).map_err(CortexError::from)?;
        let mut output = CortexOutput::default();

        for keywords in out {
            for keyword in keywords {
                output.annotations.push(types::CortexAnnotation {
                    r#type: String::from("keyword"),
                    label: keyword.text.clone(),
                    text: keyword.text,
                    score: keyword.score as f64,
                    spans: keyword
                        .offsets
                        .iter()
                        .map(|offset| types::Span::new(offset.begin, offset.end))
                        .collect(),
                });
            }
        }

        Ok(output)
    }
}
