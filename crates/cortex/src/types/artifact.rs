#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CortexArtifact {
    Summary(CortexSummaryArtifact),
    Embedding(CortexEmbeddingArtifact),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CortexSummaryArtifact {
    pub text: String,
}

impl From<CortexSummaryArtifact> for CortexArtifact {
    fn from(value: CortexSummaryArtifact) -> Self {
        Self::Summary(value)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CortexEmbeddingArtifact {
    pub vector: Vec<f32>,
}

impl From<CortexEmbeddingArtifact> for CortexArtifact {
    fn from(value: CortexEmbeddingArtifact) -> Self {
        Self::Embedding(value)
    }
}
