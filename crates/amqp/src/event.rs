use crate::Key;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Event<TBody> {
    pub id: uuid::Uuid,
    pub key: Key,
    pub body: TBody,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl<TBody> Event<TBody> {
    pub fn new(key: Key, body: TBody) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            key,
            body,
            created_at: chrono::Utc::now(),
        }
    }
}
