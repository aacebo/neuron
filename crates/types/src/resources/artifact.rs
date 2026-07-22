use crate::{actors, data};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Artifact {
    pub id: uuid::Uuid,
    pub name: String,
    pub content: Vec<data::Content>,
    #[serde(skip)]
    pub embedding: Option<Vec<f32>>,
    pub metadata: data::Metadata,
    pub created_by: actors::Actor,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
