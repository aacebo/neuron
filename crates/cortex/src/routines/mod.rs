mod embeddings;
mod entity_extraction;
mod keyword_extraction;
mod pii_extraction;
mod sentiment;
mod summarization;

pub use embeddings::*;
pub use entity_extraction::*;
pub use keyword_extraction::*;
pub use pii_extraction::*;
pub use sentiment::*;
pub use summarization::*;

pub struct Pipeline<'a> {
    pub embeddings: Option<Embeddings<'a>>,
    pub entity_extraction: Option<EntityExtraction<'a>>,
    pub keyword_extraction: Option<KeywordExtraction<'a>>,
    pub pii_extraction: Option<PIIExtraction<'a>>,
    pub sentiment: Option<Sentiment<'a>>,
    pub summarization: Option<Summarization<'a>>,
}

impl<'a> Pipeline<'a> {
    pub fn routines(&self) -> Vec<&dyn crate::Routine> {
        let mut v: Vec<&dyn crate::Routine> = Vec::new();

        if let Some(r) = &self.embeddings {
            v.push(r);
        }

        if let Some(r) = &self.entity_extraction {
            v.push(r);
        }

        if let Some(r) = &self.keyword_extraction {
            v.push(r);
        }

        if let Some(r) = &self.pii_extraction {
            v.push(r);
        }

        if let Some(r) = &self.sentiment {
            v.push(r);
        }

        if let Some(r) = &self.summarization {
            v.push(r);
        }

        v
    }
}
