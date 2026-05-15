use crate::types::MessageSource;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: uuid::Uuid,
    pub source: MessageSource,
    pub text: String,
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
            created_at: now,
            updated_at: now,
        }
    }
}
