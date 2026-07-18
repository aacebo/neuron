use crate::{actors, chats, resources, skills, tasks};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Event {
    pub id: uuid::Uuid,
    pub trace_id: uuid::Uuid,
    pub key: String,
    pub data: Data,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Data {
    Actor { actor: actors::Actor },
    Chat { chat: chats::Chat },
    Message { message: chats::Message },
    Task { task: tasks::Task },
    Artifact { artifact: resources::Artifact },
    Annotation { annotation: resources::Annotation },
    Skill { skill: skills::Skill, version: skills::Version },
}
