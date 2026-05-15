use rust_bert::pipelines::sentence_embeddings;

use crate::{CortexError, CortexInput, CortexOutput, Routine, types};

pub struct Embeddings<'a> {
    model: &'a sentence_embeddings::SentenceEmbeddingsModel,
}

impl<'a> Embeddings<'a> {
    pub fn new(model: &'a sentence_embeddings::SentenceEmbeddingsModel) -> Self {
        Self { model }
    }
}

impl<'a> Routine for Embeddings<'a> {
    fn name(&self) -> &'static str {
        "embeddings"
    }

    fn invoke(&self, input: CortexInput<'_>) -> Result<CortexOutput, CortexError> {
        let out = self.model.encode(input.text).map_err(CortexError::from)?;
        let mut output = CortexOutput::default();

        for vector in out {
            output
                .artifacts
                .push(types::CortexEmbeddingArtifact { vector }.into());
        }

        Ok(output)
    }
}
