#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: uuid::Uuid,
    pub source: MessageSource,
    pub text: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Message {
    pub fn new(text: impl Into<String>) -> Self {
        let now = chrono::Utc::now();

        Self {
            id: uuid::Uuid::new_v4(),
            source: MessageSource::Unknown,
            text: text.into(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MessageSource {
    User,
    Chat,
    Email,
    Sms,
    Webhook,
    Api,
    Unknown,
}

impl std::fmt::Display for MessageSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "user"),
            Self::Chat => write!(f, "chat"),
            Self::Email => write!(f, "email"),
            Self::Sms => write!(f, "sms"),
            Self::Webhook => write!(f, "webhook"),
            Self::Api => write!(f, "api"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}
