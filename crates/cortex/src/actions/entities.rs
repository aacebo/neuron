use rust_bert::pipelines::ner;

use crate::{Action, types};

pub struct Entities<'a> {
    model: &'a ner::NERModel,
}

impl<'a> Entities<'a> {
    pub fn new(model: &'a ner::NERModel) -> Self {
        Self { model }
    }
}

impl<'a> Action for Entities<'a> {
    fn name(&self) -> &'static str {
        "entities::extract"
    }

    fn invoke(&self, ctx: &mut crate::Context) -> Result<(), Box<dyn std::error::Error>> {
        let out = self.model.predict_full_entities(ctx.input);

        for entities in out {
            for entity in entities
                .into_iter()
                .filter(|e| e.score as f32 >= ctx.options.min_score)
            {
                ctx.annotations.push(types::CortexAnnotation {
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

        Ok(())
    }
}
