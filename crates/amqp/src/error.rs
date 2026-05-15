#[derive(Debug)]
pub enum AMQPError {
    Lapin(lapin::Error),
    Json(serde_json::Error),
    Custom(String, String),
}

impl AMQPError {
    pub fn custom(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Custom(name.into(), message.into())
    }
}

impl std::fmt::Display for AMQPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lapin(err) => write!(f, "amqp: {err}"),
            Self::Json(err) => write!(f, "amqp json: {err}"),
            Self::Custom(name, message) => write!(f, "amqp {name}: {message}"),
        }
    }
}

impl std::error::Error for AMQPError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Lapin(err) => Some(err),
            Self::Json(err) => Some(err),
            Self::Custom(_, _) => None,
        }
    }
}

impl From<lapin::Error> for AMQPError {
    fn from(value: lapin::Error) -> Self {
        Self::Lapin(value)
    }
}

impl From<serde_json::Error> for AMQPError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}
