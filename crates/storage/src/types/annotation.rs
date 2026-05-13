use crate::types::Span;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Annotation {
    pub label: String,
    pub text: String,
    pub score: f64,
    pub spans: Vec<Span>,
}
