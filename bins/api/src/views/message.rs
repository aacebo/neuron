use storage::types::{Annotation, Artifact, Message, MessageSource, Task, TaskStatus};

#[derive(Debug, serde::Serialize)]
pub struct MessageView {
    pub id: uuid::Uuid,
    pub text: String,
    pub source: MessageSource,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub annotations: Vec<Annotation>,
    pub artifacts: Vec<Artifact>,
    pub tasks: Vec<Task>,
}

impl MessageView {
    pub async fn get(storage: &storage::Storage<'_>, id: uuid::Uuid) -> Result<Option<Self>, sqlx::Error> {
        let message = match storage.messages().get(id).await? {
            Some(message) => message,
            None => return Ok(None),
        };

        let annotations = storage.annotations().get_by_message(id).await?;
        let artifacts = storage.artifacts().get_by_message(id).await?;
        let tasks = storage.tasks().get_by_message(id).await?;

        Ok(Some(Self::new(message, annotations, artifacts, tasks)))
    }

    pub fn new(message: Message, annotations: Vec<Annotation>, artifacts: Vec<Artifact>, tasks: Vec<Task>) -> Self {
        Self {
            id: message.id,
            text: message.text,
            source: message.source,
            created_at: message.created_at,
            updated_at: message.updated_at,
            annotations,
            artifacts,
            tasks,
        }
    }

    pub fn status(&self) -> Option<TaskStatus> {
        self.tasks.iter().map(|task| task.status).min_by_key(|status| *status as u8)
    }
}
