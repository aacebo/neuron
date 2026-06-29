#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CortexArtifact {
    Text(CortexTextArtifact),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CortexTextArtifact {
    pub name: String,
    pub text: String,
    pub vector: Option<Vec<f32>>,
}

impl From<CortexTextArtifact> for CortexArtifact {
    fn from(value: CortexTextArtifact) -> Self {
        Self::Text(value)
    }
}
