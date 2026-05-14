use rust_bert::pipelines::summarization;

use crate::{CortexError, CortexInput, CortexOutput, Routine, types};

pub struct Summarization<'a> {
    model: &'a summarization::SummarizationModel,
}

impl<'a> Summarization<'a> {
    pub fn new(model: &'a summarization::SummarizationModel) -> Self {
        Self { model }
    }
}

impl<'a> Routine for Summarization<'a> {
    fn name(&self) -> &'static str {
        "summarize"
    }

    fn invoke(&self, input: CortexInput<'_>) -> Result<CortexOutput, CortexError> {
        let out = self
            .model
            .summarize(input.text)
            .map_err(CortexError::from)?;
        let mut output = CortexOutput::default();

        for summary in out {
            output
                .artifacts
                .push(types::SummaryArtifact { text: summary }.into());
        }

        Ok(output)
    }
}
