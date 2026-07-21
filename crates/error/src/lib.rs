pub trait IntoError {
    fn into_error(self) -> Error;
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn new(name: impl std::fmt::Display, message: impl std::fmt::Display) -> Error {
    Error {
        trace_id: None,
        name: name.to_string(),
        message: message.to_string(),
    }
}

pub fn amqp(message: impl std::fmt::Display) -> Error {
    new("amqp", message)
}

pub fn io(message: impl std::fmt::Display) -> Error {
    new("io", message)
}

pub fn parse(message: impl std::fmt::Display) -> Error {
    new("parse", message)
}

pub fn sql(message: impl std::fmt::Display) -> Error {
    new("sql", message)
}

pub fn http(message: impl std::fmt::Display) -> Error {
    new("http", message)
}

pub fn json(message: impl std::fmt::Display) -> Error {
    new("json", message)
}

pub fn config(message: impl std::fmt::Display) -> Error {
    new("config", message)
}

pub fn ai(message: impl std::fmt::Display) -> Error {
    new("ai", message)
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Error {
    trace_id: Option<String>,
    name: String,
    message: String,
}

impl Error {
    pub fn trace(mut self, value: impl std::fmt::Display) -> Self {
        self.trace_id = Some(value.to_string());
        self
    }

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

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        json(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        io(value)
    }
}

#[cfg(feature = "amqp")]
impl From<lapin::Error> for Error {
    fn from(value: lapin::Error) -> Self {
        amqp(value)
    }
}

#[cfg(feature = "ai")]
impl From<candle_core::Error> for Error {
    fn from(value: candle_core::Error) -> Self {
        ai(value)
    }
}

#[cfg(feature = "ai")]
impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        ai(value)
    }
}

#[cfg(feature = "ai")]
impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        ai(value)
    }
}

#[cfg(feature = "storage")]
impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        sql(value)
    }
}

#[cfg(feature = "storage")]
impl From<sqlx::migrate::MigrateError> for Error {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        sql(value)
    }
}

#[cfg(feature = "web")]
impl From<actix_web::Error> for Error {
    fn from(value: actix_web::Error) -> Self {
        http(value)
    }
}

#[cfg(feature = "web")]
impl actix_web::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self.name() {
            "NotFound" => actix_web::http::StatusCode::NOT_FOUND,
            "BadRequest" => actix_web::http::StatusCode::BAD_REQUEST,
            "Unauthorized" => actix_web::http::StatusCode::UNAUTHORIZED,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code()).json(self)
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

    #[test]
    fn serde_json_errors_convert_without_a_feature() {
        let error = serde_json::from_str::<serde_json::Value>("{").unwrap_err().into_error();
        assert_eq!(error.name(), "JSON");
        assert!(!error.message().is_empty());
    }

    #[test]
    fn contextual_constructors_preserve_domain_messages() {
        assert_eq!(ai::load("missing weights").message(), "failed to load model: missing weights");
        assert_eq!(amqp::invalid_key("bad key").message(), "amqp parse: bad key");
        assert_eq!(amqp::invalid_action("bad action").message(), "amqp parse: bad action");
        assert_eq!(amqp::queue_not_found().message(), "amqp not-found: queue not found");
        assert_eq!(api::config("bad port").message(), "configuration failed: bad port");
        assert_eq!(api::request("bad frame").message(), "invalid request: bad frame");
        assert_eq!(
            executor::embedding("wrong size").message(),
            "invalid embedding output: wrong size"
        );
    }

    #[cfg(feature = "ai")]
    #[test]
    fn ai_from_conversions_use_the_ai_domain() {
        let candle = candle_core::Error::Msg("bad tensor".to_string()).into_error();
        assert_eq!(candle.name(), "AI");
        assert_eq!(candle.message(), "inference failed: bad tensor");

        let url = url::Url::parse(":").unwrap_err().into_error();
        assert_eq!(url.name(), "AI");

        let request = reqwest::Client::new().get("://").build().unwrap_err().into_error();
        assert_eq!(request.name(), "AI");
        assert!(request.message().starts_with("request failed: "));
    }

    #[cfg(feature = "amqp")]
    #[test]
    fn amqp_from_conversion_uses_the_amqp_domain() {
        let error = lapin::Error::ChannelsLimitReached.into_error();
        assert_eq!(error.name(), "AMQP");
        assert!(error.message().starts_with("amqp: "));
    }

    #[cfg(feature = "storage")]
    #[test]
    fn storage_from_conversion_uses_the_storage_domain() {
        let error = sqlx::Error::RowNotFound.into_error();
        assert_eq!(error.name(), "Storage");
        assert_eq!(
            error.message(),
            "no rows returned by a query that expected to return at least one row"
        );
    }

    #[cfg(feature = "web")]
    #[actix_web::test]
    async fn actix_responses_map_statuses_and_json() {
        use actix_web::ResponseError;
        use actix_web::body::to_bytes;
        use actix_web::http::StatusCode;

        let cases = [
            (bad_request("bad"), StatusCode::BAD_REQUEST),
            (unauthorized("no"), StatusCode::UNAUTHORIZED),
            (not_found("gone"), StatusCode::NOT_FOUND),
            (storage("down"), StatusCode::INTERNAL_SERVER_ERROR),
        ];

        for (error, status) in cases {
            let expected = serde_json::to_value(&error).unwrap();
            let response = error.error_response();
            assert_eq!(response.status(), status);
            let body = to_bytes(response.into_body()).await.unwrap();
            assert_eq!(serde_json::from_slice::<serde_json::Value>(&body).unwrap(), expected);
        }
    }
}
