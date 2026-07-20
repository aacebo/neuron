#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Artifact {
    pub id: uuid::Uuid,
    pub chat_id: uuid::Uuid,
    pub message_id: Option<uuid::Uuid>,
    pub task_id: Option<uuid::Uuid>,
    pub name: String,
    pub content: sqlx::types::Json<Vec<types::data::Content>>,
    #[serde(with = "embedding::serde")]
    pub embedding: Option<pgvector::Vector>,
    pub metadata: sqlx::types::Json<types::data::Metadata>,
    pub created_by_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

mod embedding {
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
