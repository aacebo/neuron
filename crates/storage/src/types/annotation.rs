use crate::types::Span;

/// Describes some sub span of text in a message
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct MessageAnnotation {
    pub id: uuid::Uuid,
    pub message_id: uuid::Uuid,
    pub r#type: String,
    pub label: String,
    pub text: String,
    pub score: f64,
    pub spans: sqlx::types::Json<Vec<Span>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl MessageAnnotation {
    pub fn new(
        message_id: uuid::Uuid,
        r#type: impl Into<String>,
        label: impl Into<String>,
        text: impl Into<String>,
        score: f64,
        spans: Vec<Span>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            message_id,
            r#type: r#type.into(),
            label: label.into(),
            text: text.into(),
            score,
            spans: sqlx::types::Json::from(spans),
            created_at: chrono::Utc::now(),
        }
    }
}
