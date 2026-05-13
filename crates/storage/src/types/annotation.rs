#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Annotation {
    pub label: String,
    pub text: String,
    pub score: f32,
    pub start: u32,
    pub end: u32,
}
