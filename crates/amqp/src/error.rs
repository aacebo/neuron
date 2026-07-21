#[derive(Debug)]
pub enum Error {
    Lapin(lapin::Error),
    Json(serde_json::Error),
    Custom(String, String),
}

impl Error {
    pub fn custom(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Custom(name.into(), message.into())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lapin(err) => write!(f, "amqp: {err}"),
            Self::Json(err) => write!(f, "amqp json: {err}"),
            Self::Custom(name, message) => write!(f, "amqp {name}: {message}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Lapin(err) => Some(err),
            Self::Json(err) => Some(err),
            Self::Custom(_, _) => None,
        }
    }
}

impl From<lapin::Error> for Error {
    fn from(value: lapin::Error) -> Self {
        Self::Lapin(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<Error> for error::Error {
    fn from(value: Error) -> Self {
        ::error::new("AMQP", value)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_to_common_error() {
        let error: error::Error = Error::custom("parse", "bad key").into();
        assert_eq!(error.name(), "AMQP");
        assert_eq!(error.message(), "amqp parse: bad key");
    }
}
