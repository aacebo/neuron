#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Artifact {
    Summary(SummaryArtifact),
    Embedding(EmbeddingArtifact),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SummaryArtifact {
    pub text: String,
}

impl From<SummaryArtifact> for Artifact {
    fn from(value: SummaryArtifact) -> Self {
        Self::Summary(value)
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EmbeddingArtifact {
    pub vector: Vec<f32>,
}

impl From<EmbeddingArtifact> for Artifact {
    fn from(value: EmbeddingArtifact) -> Self {
        Self::Embedding(value)
    }
}
