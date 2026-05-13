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

impl Message {
    pub fn new(text: impl Into<String>) -> Self {
        let now = chrono::Utc::now();

        Self {
            id: uuid::Uuid::new_v4(),
            source: MessageSource::Unknown,
            text: text.into(),
            annotations: sqlx::types::Json::from(vec![]),
            created_at: now,
            updated_at: now,
        }
    }
}
