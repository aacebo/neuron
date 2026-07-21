use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use error::IntoError;

#[derive(Debug, Clone)]
pub enum Error {
    Common(error::Error),
    Config(String),
    Server(String),
    Request(String),
}

impl Error {
    pub fn config(error: impl std::fmt::Display) -> Self {
        Self::Config(error.to_string())
    }

    pub fn server(error: impl std::fmt::Display) -> Self {
        Self::Server(error.to_string())
    }

    pub fn request(error: impl std::fmt::Display) -> Self {
        Self::Request(error.to_string())
    }
}

impl From<error::Error> for Error {
    fn from(value: ::error::Error) -> Self {
        Self::Common(value)
    }
}

impl From<Error> for error::Error {
    fn from(value: Error) -> Self {
        match value {
            Error::Common(error) => error,
            Error::Request(message) => ::error::bad_request(message),
            api_error @ (Error::Config(_) | Error::Server(_)) => ::error::new("API", api_error),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Common(error) => error.fmt(f),
            Self::Config(message) => write!(f, "configuration failed: {message}"),
            Self::Server(message) => write!(f, "server failed: {message}"),
            Self::Request(message) => write!(f, "invalid request: {message}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Common(error) => Some(error),
            _ => None,
        }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self.clone().into_error().name() {
            "NotFound" => StatusCode::NOT_FOUND,
            "BadRequest" => StatusCode::BAD_REQUEST,
            "Unauthorized" => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.clone().into_error())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use actix_web::ResponseError;
    use actix_web::body::to_bytes;
    use actix_web::http::StatusCode;

    use super::*;

    #[actix_web::test]
    async fn common_errors_map_to_statuses_and_json() {
        let cases = [
            (::error::bad_request("bad"), StatusCode::BAD_REQUEST),
            (::error::unauthorized("no"), StatusCode::UNAUTHORIZED),
            (::error::not_found("gone"), StatusCode::NOT_FOUND),
            (::error::new("Storage", "down"), StatusCode::INTERNAL_SERVER_ERROR),
        ];

        for (common, status) in cases {
            let expected = serde_json::to_value(&common).unwrap();
            let response = Error::from(common).error_response();
            assert_eq!(response.status(), status);

            let body = to_bytes(response.into_body()).await.unwrap();
            assert_eq!(serde_json::from_slice::<serde_json::Value>(&body).unwrap(), expected);
        }
    }

    #[actix_web::test]
    async fn api_errors_convert_to_common_json() {
        let response = Error::server("socket closed").error_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = to_bytes(response.into_body()).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
            serde_json::json!({"name": "API", "message": "server failed: socket closed"})
        );
    }
}
