#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Event {
    pub id: uuid::Uuid,
    pub trace_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub actor_id: Option<uuid::Uuid>,
    pub chat_id: Option<uuid::Uuid>,
    pub message_id: Option<uuid::Uuid>,
    pub task_id: Option<uuid::Uuid>,
    pub key: String,
    pub data: sqlx::types::Json<EventData>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventData {
    Actor {
        actor: types::actors::Actor,
    },
    Chat {
        chat: types::chats::Chat,
    },
    Message {
        message: types::chats::Message,
    },
    Task {
        task: types::tasks::Task,
    },
    Artifact {
        artifact: types::resources::Artifact,
    },
    Annotation {
        annotation: types::resources::Annotation,
    },
    Skill {
        skill: types::skills::Skill,
        version: types::skills::Version,
    },
}
