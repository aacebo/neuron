use crate::{actors, chats, resources, tasks};

pub fn new(trace_id: uuid::Uuid, key: impl std::fmt::Display, data: impl Into<Data>) -> Event {
    Event {
        id: uuid::Uuid::new_v4(),
        trace_id,
        key: key.to_string(),
        data: data.into(),
        created_at: chrono::Utc::now(),
    }
}

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
}

impl Data {
    pub fn actor_id(&self) -> Option<uuid::Uuid> {
        match self {
            Self::Actor { actor } => Some(actor.id),
            _ => None,
        }
    }

    pub fn chat_id(&self) -> Option<uuid::Uuid> {
        match self {
            Self::Chat { chat } => Some(chat.id),
            _ => None,
        }
    }

    pub fn message_id(&self) -> Option<uuid::Uuid> {
        match self {
            Self::Message { message } => Some(message.id),
            _ => None,
        }
    }

    pub fn task_id(&self) -> Option<uuid::Uuid> {
        match self {
            Self::Task { task } => Some(task.id),
            _ => None,
        }
    }

    pub fn artifact_id(&self) -> Option<uuid::Uuid> {
        match self {
            Self::Artifact { artifact } => Some(artifact.id),
            _ => None,
        }
    }

    pub fn annotation_id(&self) -> Option<uuid::Uuid> {
        match self {
            Self::Annotation { annotation } => Some(annotation.id),
            _ => None,
        }
    }
}

impl From<actors::Actor> for Data {
    fn from(actor: actors::Actor) -> Self {
        Self::Actor { actor }
    }
}

impl From<chats::Chat> for Data {
    fn from(chat: chats::Chat) -> Self {
        Self::Chat { chat }
    }
}

impl From<chats::Message> for Data {
    fn from(message: chats::Message) -> Self {
        Self::Message { message }
    }
}

impl From<tasks::Task> for Data {
    fn from(task: tasks::Task) -> Self {
        Self::Task { task }
    }
}

impl From<resources::Artifact> for Data {
    fn from(artifact: resources::Artifact) -> Self {
        Self::Artifact { artifact }
    }
}

impl From<resources::Annotation> for Data {
    fn from(annotation: resources::Annotation) -> Self {
        Self::Annotation { annotation }
    }
}
