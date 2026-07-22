use crate::{actors, data};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Chat {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub name: Option<String>,
    pub created_by: actors::ActorPartial,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub closed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatPartial {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub name: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub id: uuid::Uuid,
    pub chat: ChatPartial,
    pub content: Vec<data::Content>,
    pub metadata: data::Metadata,
    #[serde(skip)]
    pub embedding: Option<Vec<f32>>,
    pub created_by: actors::ActorPartial,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
