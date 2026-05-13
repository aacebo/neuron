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
