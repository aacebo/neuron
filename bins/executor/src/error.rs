#[derive(Debug, Clone)]
pub enum Error {
    Config(String),
    Embedding(String),
}

impl Error {
    pub fn config(error: impl std::fmt::Display) -> Self {
        Self::Config(error.to_string())
    }

    pub fn embedding(error: impl std::fmt::Display) -> Self {
        Self::Embedding(error.to_string())
    }
}

impl From<Error> for error::Error {
    fn from(value: Error) -> Self {
        error::new("Executor", value)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(message) => write!(f, "configuration failed: {message}"),
            Self::Embedding(message) => write!(f, "invalid embedding output: {message}"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_to_common_error() {
        let error: error::Error = Error::embedding("expected one artifact").into();
        assert_eq!(error.name(), "Executor");
        assert_eq!(error.message(), "invalid embedding output: expected one artifact");
    }
}
