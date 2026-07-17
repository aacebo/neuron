#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Log {
    pub id: uuid::Uuid,
    pub trace_id: uuid::Uuid,
    pub level: LogLevel,
    pub source: String,
    pub message: String,
    pub context: Option<sqlx::types::JsonValue>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Log {
    pub fn trace(trace_id: uuid::Uuid, source: impl std::fmt::Display, message: impl std::fmt::Display) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            trace_id,
            level: LogLevel::Trace,
            source: source.to_string(),
            message: message.to_string(),
            context: None,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn info(trace_id: uuid::Uuid, source: impl std::fmt::Display, message: impl std::fmt::Display) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            trace_id,
            level: LogLevel::Info,
            source: source.to_string(),
            message: message.to_string(),
            context: None,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn warn(trace_id: uuid::Uuid, source: impl std::fmt::Display, message: impl std::fmt::Display) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            trace_id,
            level: LogLevel::Warn,
            source: source.to_string(),
            message: message.to_string(),
            context: None,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn error(trace_id: uuid::Uuid, source: impl std::fmt::Display, message: impl std::fmt::Display) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            trace_id,
            level: LogLevel::Error,
            source: source.to_string(),
            message: message.to_string(),
            context: None,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn with(mut self, context: impl Into<sqlx::types::JsonValue>) -> Self {
        self.context = Some(context.into());
        self
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    #[default]
    Trace,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trace => write!(f, "trace"),
            Self::Info => write!(f, "info"),
            Self::Warn => write!(f, "warn"),
            Self::Error => write!(f, "error"),
        }
    }
}
