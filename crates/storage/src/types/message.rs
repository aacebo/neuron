use crate::types::{Annotation, MessageSource};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: uuid::Uuid,
    pub source: MessageSource,
    pub text: String,
    pub annotations: sqlx::types::Json<Vec<Annotation>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
