use crate::types::{Annotation, Artifact, MessageSource};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: uuid::Uuid,
    pub source: MessageSource,
    pub text: String,
    pub artifacts: sqlx::types::Json<Vec<Artifact>>,
    pub annotations: sqlx::types::Json<Vec<Annotation>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Message {
    pub fn new(text: impl Into<String>) -> Self {
        let now = chrono::Utc::now();

        Self {
            id: uuid::Uuid::new_v4(),
            source: MessageSource::Unknown,
            text: text.into(),
            artifacts: sqlx::types::Json::from(vec![]),
            annotations: sqlx::types::Json::from(vec![]),
            created_at: now,
            updated_at: now,
        }
    }
}
