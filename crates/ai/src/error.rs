#[derive(Debug)]
pub enum Error {
    Parse(String),
    Load(String),
    Tokenize(String),
    Inference(String),
    Network(String),
    Auth(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(message) => write!(f, "{message}"),
            Self::Load(message) => write!(f, "failed to load model: {message}"),
            Self::Tokenize(message) => write!(f, "failed to tokenize input: {message}"),
            Self::Inference(message) => write!(f, "inference failed: {message}"),
            Self::Network(message) => write!(f, "request failed: {message}"),
            Self::Auth(message) => write!(f, "authentication failed: {message}"),
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn parse(err: impl std::fmt::Display) -> Self {
        Self::Parse(err.to_string())
    }

    pub fn load(err: impl std::fmt::Display) -> Self {
        Self::Load(err.to_string())
    }

    pub fn tokenize(err: impl std::fmt::Display) -> Self {
        Self::Tokenize(err.to_string())
    }

    pub fn inference(err: impl std::fmt::Display) -> Self {
        Self::Inference(err.to_string())
    }

    pub fn network(err: impl std::fmt::Display) -> Self {
        Self::Network(err.to_string())
    }

    pub fn auth(err: impl std::fmt::Display) -> Self {
        Self::Auth(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<Error> for error::Error {
    fn from(value: Error) -> Self {
        ::error::new("AI", value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_to_common_error() {
        let error: error::Error = Error::inference("bad tensor").into();
        assert_eq!(error.name(), "AI");
        assert_eq!(error.message(), "inference failed: bad tensor");
    }
}
