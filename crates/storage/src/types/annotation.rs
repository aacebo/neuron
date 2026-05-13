use crate::types::Span;

/// Describes some sub span of text in a message
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Annotation {
    pub r#type: String,
    pub label: String,
    pub text: String,
    pub score: f64,
    pub spans: Vec<Span>,
}
