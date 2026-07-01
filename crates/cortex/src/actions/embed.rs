use rust_bert::pipelines::sentence_embeddings;

use crate::{Action, Context, types};

pub struct Embed<'a> {
    model: &'a sentence_embeddings::SentenceEmbeddingsModel,
}

impl<'a> Embed<'a> {
    pub fn new(model: &'a sentence_embeddings::SentenceEmbeddingsModel) -> Self {
        Self { model }
    }
}

impl<'a> Action for Embed<'a> {
    fn name(&self) -> &'static str {
        "embed"
    }

    fn invoke(&self, ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>> {
        let out = self.model.encode(ctx.input)?;

        for (vector, text) in out.into_iter().zip(ctx.input) {
            ctx.artifacts.push(
                types::CortexTextArtifact {
                    name: "embedding".to_string(),
                    text: text.to_string(),
                    vector: Some(vector),
                }
                .into(),
            );
        }

        Ok(())
    }
}
