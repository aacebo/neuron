use std::collections::BTreeMap;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Metadata(BTreeMap<String, serde_json::Value>);

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Content {
    Text {
        text: String,
    },
    File {
        name: Option<String>,
        #[serde(flatten)]
        file: FileContent,
    },
    Json {
        json: serde_json::Value,
    },
}

impl Content {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { text } => Some(text),
            _ => None,
        }
    }

    pub fn as_file(&self) -> Option<(Option<&str>, &FileContent)> {
        match self {
            Self::File { name, file } => Some((name.as_deref(), file)),
            _ => None,
        }
    }

    pub fn as_json(&self) -> Option<&serde_json::Value> {
        match self {
            Self::Json { json } => Some(json),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileContent {
    Uri { uri: url::Url },
    Base64 { base64: String },
}

impl std::fmt::Display for FileContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Uri { uri } => write!(f, "{uri}"),
            Self::Base64 { base64 } => write!(f, "{base64}"),
        }
    }
}
