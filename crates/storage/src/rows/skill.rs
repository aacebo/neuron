#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Skill {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub name: String,
    pub display_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct SkillVersion {
    pub id: uuid::Uuid,
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
    pub prerelease: Option<String>,
    pub status: VersionStatus,
    pub description: String,
    pub tags: Vec<String>,
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum VersionStatus {
    Draft,
    Published,
    Deprecated,
}
