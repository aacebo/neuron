use pgvector::Vector;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ArtifactContent {
    Summary(SummaryArtifact),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SummaryArtifact {
    pub text: String,
}

impl From<SummaryArtifact> for ArtifactContent {
    fn from(value: SummaryArtifact) -> Self {
        Self::Summary(value)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct MessageArtifact {
    pub id: uuid::Uuid,
    pub message_id: uuid::Uuid,
    pub r#type: String,
    pub content: sqlx::types::Json<ArtifactContent>,
    #[serde(with = "embedding::serde")]
    pub embedding: Option<Vector>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl MessageArtifact {
    pub fn new(
        message_id: uuid::Uuid,
        r#type: impl Into<String>,
        content: ArtifactContent,
        embedding: Option<Vec<f32>>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            message_id,
            r#type: r#type.into(),
            content: sqlx::types::Json::from(content),
            embedding: embedding.map(Vector::from),
            created_at: chrono::Utc::now(),
        }
    }
}

pub mod embedding {
    pub mod serde {
        use pgvector::Vector;
        use serde::{Deserialize, Deserializer, Serialize, Serializer};

        pub fn serialize<S: Serializer>(value: &Option<Vector>, s: S) -> Result<S::Ok, S::Error> {
            value.as_ref().map(Vector::to_vec).serialize(s)
        }

        pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Vector>, D::Error> {
            Ok(Option::<Vec<f32>>::deserialize(d)?.map(Vector::from))
        }
    }
}
