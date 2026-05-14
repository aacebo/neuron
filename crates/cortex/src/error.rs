#[derive(Debug)]
pub enum CortexError {
    Pipeline(rust_bert::RustBertError),
}

impl From<rust_bert::RustBertError> for CortexError {
    fn from(value: rust_bert::RustBertError) -> Self {
        Self::Pipeline(value)
    }
}

impl std::error::Error for CortexError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Pipeline(v) => Some(v),
        }
    }
}

impl std::fmt::Display for CortexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pipeline(v) => write!(f, "{}", v),
        }
    }
}
