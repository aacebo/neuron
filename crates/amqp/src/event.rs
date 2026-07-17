use crate::Key;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Event<TBody> {
    pub id: uuid::Uuid,
    pub trace_id: uuid::Uuid,
    pub key: Key,
    pub body: TBody,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl<TBody> Event<TBody> {
    pub fn new(trace_id: uuid::Uuid, key: Key, body: TBody) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            trace_id,
            key,
            body,
            created_at: chrono::Utc::now(),
        }
    }
}

impl Event<String> {
    pub fn cast<'a, TBody: serde::Deserialize<'a>>(&'a self) -> Result<Event<TBody>, serde_json::Error> {
        let body = serde_json::from_str::<TBody>(&self.body)?;

        Ok(Event {
            id: self.id,
            trace_id: self.trace_id,
            key: self.key.clone(),
            body,
            created_at: self.created_at,
        })
    }
}
