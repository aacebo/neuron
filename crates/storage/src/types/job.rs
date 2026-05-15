#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Job {
    pub id: uuid::Uuid,
    pub name: String,
    pub status: JobStatus,
    pub error: Option<sqlx::types::JsonValue>,
    pub attempts: i32,
    pub max_attempts: i32,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ended_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Job {
    pub fn new(name: impl Into<String>) -> Self {
        let now = chrono::Utc::now();

        Self {
            id: uuid::Uuid::new_v4(),
            name: name.into(),
            max_attempts: 3,
            created_at: now,
            updated_at: now,
            ..Default::default()
        }
    }

    pub fn start(mut self) -> Self {
        assert!(self.status != JobStatus::Running);
        let now = chrono::Utc::now();

        self.status = JobStatus::Running;
        self.attempts += 1;
        self.started_at = Some(now);
        self.error = None;
        self.ended_at = None;
        self.updated_at = now;
        self
    }

    pub fn success(mut self) -> Self {
        assert!(self.status == JobStatus::Running);
        let now = chrono::Utc::now();

        self.status = JobStatus::Success;
        self.ended_at = Some(now);
        self.updated_at = now;
        self
    }

    pub fn fail(mut self, error: impl Into<sqlx::types::JsonValue>) -> Self {
        assert!(self.status == JobStatus::Running);
        let now = chrono::Utc::now();

        self.error = Some(error.into());
        self.status = JobStatus::Failure;
        self.ended_at = Some(now);
        self.updated_at = now;
        self
    }

    pub fn cancel(mut self) -> Self {
        assert!(self.status == JobStatus::Running);
        let now = chrono::Utc::now();

        self.status = JobStatus::Cancelled;
        self.ended_at = Some(now);
        self.updated_at = now;
        self
    }
}

#[derive(
    Debug, Default, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type,
)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    #[default]
    Queued,
    Running,
    Success,
    Failure,
    Cancelled,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Queued => write!(f, "queued"),
            Self::Running => write!(f, "running"),
            Self::Success => write!(f, "success"),
            Self::Failure => write!(f, "failure"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum JobSource {
    Message(uuid::Uuid),
}

impl JobSource {
    pub fn message(message_id: uuid::Uuid) -> Self {
        Self::Message(message_id)
    }
}
