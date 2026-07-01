use rust_bert::pipelines::{sentence_embeddings, summarization};

use crate::{Action, CortexError, CortexInput, CortexOutput, Routine, types};

pub struct Summarization<'a> {
    model: &'a summarization::SummarizationModel,
    embed_model: Option<&'a sentence_embeddings::SentenceEmbeddingsModel>,
}

impl<'a> Summarization<'a> {
    pub fn new(
        model: &'a summarization::SummarizationModel,
        embed_model: Option<&'a sentence_embeddings::SentenceEmbeddingsModel>,
    ) -> Self {
        Self { model, embed_model }
    }
}

impl<'a> Action for Summarization<'a> {
    fn name(&self) -> &'static str {
        "summarize"
    }

    fn invoke(&self, ctx: &mut crate::Context) -> Result<(), Box<dyn std::error::Error>> {
        let text = ctx
            .input
            .iter()
            .copied()
            .filter(|text| text.split_whitespace().count() >= 8)
            .collect::<Vec<_>>();

        if text.is_empty() {
            return Ok(());
        }

        let out = self.model.summarize(&text)?;

        for summary in out {
            let artifact = if let Some(model) = self.embed_model {
                let vector = model.encode(&[&summary])?;
                types::CortexTextArtifact {
                    name: "summary".to_string(),
                    text: summary,
                    vector: vector.first().cloned(),
                }
            } else {
                types::CortexTextArtifact {
                    name: "summary".to_string(),
                    text: summary,
                    vector: None,
                }
            };

            ctx.artifacts.push(artifact.into());
        }

        Ok(())
    }
}
