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
