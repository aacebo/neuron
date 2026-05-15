use crate::types::Span;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CortexAnnotation {
    pub r#type: String,
    pub label: String,
    pub text: String,
    pub score: f64,
    pub spans: Vec<Span>,
}
