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
