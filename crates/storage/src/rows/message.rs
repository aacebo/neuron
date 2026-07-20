#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: uuid::Uuid,
    pub chat_id: uuid::Uuid,
    pub content: sqlx::types::Json<Vec<types::data::Content>>,
    pub metadata: sqlx::types::Json<types::data::Metadata>,
    pub created_by_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
