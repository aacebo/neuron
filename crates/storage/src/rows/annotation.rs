/// Describes some sub span of text in a message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Annotation {
    pub id: uuid::Uuid,
    pub message_id: uuid::Uuid,
    pub task_id: Option<uuid::Uuid>,
    #[sqlx(rename = "type")]
    pub ty: String,
    pub label: String,
    pub text: String,
    pub score: f64,
    pub spans: sqlx::types::Json<Vec<types::resources::Span>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
