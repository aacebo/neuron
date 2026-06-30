use rust_bert::pipelines::{sentence_embeddings, summarization};

use crate::{CortexError, CortexInput, CortexOutput, Routine, types};

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

impl<'a> Routine for Summarization<'a> {
    fn name(&self) -> &'static str {
        "summarize"
    }

    fn invoke(&self, input: CortexInput<'_>) -> Result<CortexOutput, CortexError> {
        let mut output = CortexOutput::default();
        let text = input
            .text
            .iter()
            .copied()
            .filter(|text| text.split_whitespace().count() >= 8)
            .collect::<Vec<_>>();

        if text.is_empty() {
            return Ok(output);
        }

        let out = self.model.summarize(&text).map_err(CortexError::from)?;

        for summary in out {
            let artifact = if let Some(model) = self.embed_model {
                let vector = model.encode(&[&summary]).map_err(CortexError::from)?;
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

            output.artifacts.push(artifact.into());
        }

        Ok(output)
    }
}
