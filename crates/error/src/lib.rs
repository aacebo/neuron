pub fn new(name: impl std::fmt::Display, message: impl std::fmt::Display) -> Error {
    Error {
        name: name.to_string(),
        message: message.to_string(),
    }
}

pub fn not_found(message: impl std::fmt::Display) -> Error {
    new("NotFound", message)
}

pub fn bad_request(message: impl std::fmt::Display) -> Error {
    new("BadRequest", message)
}

pub fn unauthorized(message: impl std::fmt::Display) -> Error {
    new("Unauthorized", message)
}

pub trait IntoError {
    fn into_error(self) -> Error;
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Error {
    name: String,
    message: String,
}

impl Error {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error::{} => {}", self.name, self.message)
    }
}

impl<T: Into<Error>> IntoError for T {
    fn into_error(self) -> Error {
        self.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructors_expose_and_serialize_the_common_shape() {
        let error = not_found("actor 42");

        assert_eq!(error.name(), "NotFound");
        assert_eq!(error.message(), "actor 42");
        assert_eq!(error.to_string(), "error::NotFound => actor 42");
        assert_eq!(
            serde_json::to_value(error).unwrap(),
            serde_json::json!({"name": "NotFound", "message": "actor 42"})
        );
    }

    #[test]
    fn into_error_uses_from_conversions() {
        let error = bad_request("invalid input").into_error();
        assert_eq!(error.name(), "BadRequest");
    }
}
