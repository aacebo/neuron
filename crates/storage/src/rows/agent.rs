#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Agent {
    pub actor_id: uuid::Uuid,
    pub status: AgentStatus,
    pub description: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum AgentStatus {
    Online,
    Offline,
}

impl From<AgentStatus> for types::actors::AgentStatus {
    fn from(value: AgentStatus) -> Self {
        match value {
            AgentStatus::Online => types::actors::AgentStatus::Online,
            AgentStatus::Offline => types::actors::AgentStatus::Offline,
        }
    }
}
