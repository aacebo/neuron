use storage::types::{Job, JobStatus, Message, MessageAnnotation, MessageArtifact, MessageSource};

#[derive(Debug, serde::Serialize)]
pub struct MessageView {
    pub id: uuid::Uuid,
    pub text: String,
    pub source: MessageSource,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub annotations: Vec<MessageAnnotation>,
    pub artifacts: Vec<MessageArtifact>,
    pub jobs: Vec<Job>,
}

impl MessageView {
    pub async fn get(
        storage: &storage::Storage<'_>,
        id: uuid::Uuid,
    ) -> Result<Option<Self>, sqlx::Error> {
        let message = match storage.messages().get(id).await? {
            Some(message) => message,
            None => return Ok(None),
        };

        let annotations = storage.annotations().get_by_message(id).await?;
        let artifacts = storage.artifacts().get_by_message(id).await?;
        let jobs = storage.jobs().get_by_message(id).await?;

        Ok(Some(Self::new(message, annotations, artifacts, jobs)))
    }

    pub fn new(
        message: Message,
        annotations: Vec<MessageAnnotation>,
        artifacts: Vec<MessageArtifact>,
        jobs: Vec<Job>,
    ) -> Self {
        Self {
            id: message.id,
            text: message.text,
            source: message.source,
            created_at: message.created_at,
            updated_at: message.updated_at,
            annotations,
            artifacts,
            jobs,
        }
    }

    pub fn status(&self) -> Option<JobStatus> {
        self.jobs
            .iter()
            .map(|job| job.status)
            .min_by_key(|status| *status as u8)
    }
}
