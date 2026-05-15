mod error;
pub mod routines;
pub mod types;

pub use error::*;

use rust_bert::pipelines::*;

pub trait Routine {
    fn name(&self) -> &'static str;
    fn invoke(&self, input: CortexInput<'_>) -> Result<CortexOutput, CortexError>;
}

#[derive(Debug, Copy, Clone)]
pub struct CortexInput<'a> {
    pub text: &'a [&'a str],
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct CortexOutput {
    pub annotations: Vec<types::CortexAnnotation>,
    pub artifacts: Vec<types::CortexArtifact>,
}

#[derive(Default)]
pub struct CortexConfig {
    sentence_embeddings: Option<sentence_embeddings::SentenceEmbeddingsConfig>,
    entity_extraction: Option<token_classification::TokenClassificationConfig>,
    keyword_extraction: Option<keywords_extraction::KeywordExtractionConfig<'static>>,
    pii_extraction: Option<token_classification::TokenClassificationConfig>,
    sentiment: Option<sentiment::SentimentConfig>,
    summarization: Option<summarization::SummarizationConfig>,
}

impl CortexConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_sentence_embeddings(
        mut self,
        config: sentence_embeddings::SentenceEmbeddingsConfig,
    ) -> Self {
        self.sentence_embeddings = Some(config);
        self
    }

    pub fn with_entity_extraction(
        mut self,
        config: token_classification::TokenClassificationConfig,
    ) -> Self {
        self.entity_extraction = Some(config);
        self
    }

    pub fn with_keyword_extraction(
        mut self,
        config: keywords_extraction::KeywordExtractionConfig<'static>,
    ) -> Self {
        self.keyword_extraction = Some(config);
        self
    }

    pub fn with_pii_extraction(
        mut self,
        config: token_classification::TokenClassificationConfig,
    ) -> Self {
        self.pii_extraction = Some(config);
        self
    }

    pub fn with_sentiment(mut self, config: sentiment::SentimentConfig) -> Self {
        self.sentiment = Some(config);
        self
    }

    pub fn with_summarization(mut self, config: summarization::SummarizationConfig) -> Self {
        self.summarization = Some(config);
        self
    }

    pub fn build(self) -> Result<Cortex, CortexError> {
        let mut cortex = Cortex::default();

        if let Some(config) = self.sentence_embeddings {
            cortex.sentence_embeddings =
                Some(sentence_embeddings::SentenceEmbeddingsModel::new(config)?);
        }

        if let Some(config) = self.entity_extraction {
            cortex.entity_extraction = Some(ner::NERModel::new(config)?);
        }

        if let Some(config) = self.keyword_extraction {
            cortex.keyword_extraction =
                Some(keywords_extraction::KeywordExtractionModel::new(config)?);
        }

        if let Some(config) = self.pii_extraction {
            cortex.pii_extraction = Some(ner::NERModel::new(config)?);
        }

        if let Some(config) = self.sentiment {
            cortex.sentiment = Some(sentiment::SentimentModel::new(config)?);
        }

        if let Some(config) = self.summarization {
            cortex.summarization = Some(summarization::SummarizationModel::new(config)?);
        }

        Ok(cortex)
    }
}

#[derive(Default)]
pub struct Cortex {
    sentence_embeddings: Option<sentence_embeddings::SentenceEmbeddingsModel>,
    entity_extraction: Option<ner::NERModel>,
    keyword_extraction: Option<keywords_extraction::KeywordExtractionModel<'static>>,
    pii_extraction: Option<ner::NERModel>,
    sentiment: Option<sentiment::SentimentModel>,
    summarization: Option<summarization::SummarizationModel>,
}

impl Cortex {
    pub fn pipeline(&self) -> routines::Pipeline<'_> {
        routines::Pipeline {
            embeddings: self
                .sentence_embeddings
                .as_ref()
                .map(|m| routines::Embeddings::new(m)),
            entity_extraction: self
                .entity_extraction
                .as_ref()
                .map(|m| routines::EntityExtraction::new(m)),
            keyword_extraction: self
                .keyword_extraction
                .as_ref()
                .map(|m| routines::KeywordExtraction::new(m)),
            pii_extraction: self
                .pii_extraction
                .as_ref()
                .map(|m| routines::PIIExtraction::new(m)),
            sentiment: self.sentiment.as_ref().map(|m| routines::Sentiment::new(m)),
            summarization: self
                .summarization
                .as_ref()
                .map(|m| routines::Summarization::new(m)),
        }
    }
}
