#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Error {
    Sql(String),
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self::Sql(value.to_string())
    }
}

impl From<Error> for error::Error {
    fn from(value: Error) -> Self {
        ::error::new("Storage", value)
    }
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sql(value) => write!(f, "{value}"),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_to_common_error() {
        let error: error::Error = Error::from(sqlx::Error::RowNotFound).into();
        assert_eq!(error.name(), "Storage");
        assert_eq!(
            error.message(),
            "no rows returned by a query that expected to return at least one row"
        );
    }
}
