use rust_bert::pipelines::ner;

use crate::{CortexError, CortexInput, CortexOutput, Routine, types};

pub struct EntityExtraction<'a> {
    model: &'a ner::NERModel,
}

impl<'a> EntityExtraction<'a> {
    pub fn new(model: &'a ner::NERModel) -> Self {
        Self { model }
    }
}

impl<'a> Routine for EntityExtraction<'a> {
    fn name(&self) -> &'static str {
        "entity-extraction"
    }

    fn invoke(&self, input: CortexInput<'_>) -> Result<CortexOutput, CortexError> {
        let out = self.model.predict_full_entities(input.text);
        let mut output = CortexOutput::default();

        for entities in out {
            for entity in entities {
                output.annotations.push(types::CortexAnnotation {
                    r#type: String::from("entity"),
                    label: match entity.label.as_str() {
                        "ORG" => "organization".to_string(),
                        "PER" => "person".to_string(),
                        "LOC" => "location".to_string(),
                        other => other.to_lowercase(), // Fallback for safety
                    },
                    text: entity.word,
                    score: entity.score,
                    spans: vec![types::Span::new(entity.offset.begin, entity.offset.end)],
                });
            }
        }

        Ok(output)
    }
}
