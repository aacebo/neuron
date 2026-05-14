use rust_bert::pipelines::ner;

use crate::{CortexError, CortexInput, CortexOutput, Routine, types};

pub struct PIIExtraction<'a> {
    model: &'a ner::NERModel,
}

impl<'a> PIIExtraction<'a> {
    pub fn new(model: &'a ner::NERModel) -> Self {
        Self { model }
    }
}

impl<'a> Routine for PIIExtraction<'a> {
    fn name(&self) -> &'static str {
        "pii-extraction"
    }

    fn invoke(&self, input: CortexInput<'_>) -> Result<CortexOutput, CortexError> {
        let out = self.model.predict_full_entities(input.text);
        let mut output = CortexOutput::default();

        for entities in out {
            for entity in entities {
                output.annotations.push(types::Annotation {
                    r#type: String::from("pii"),
                    label: entity.label.to_lowercase(),
                    text: entity.word,
                    score: entity.score,
                    spans: vec![types::Span::new(
                        entity.offset.begin + 1,
                        entity.offset.end + 1,
                    )],
                });
            }
        }

        Ok(output)
    }
}
