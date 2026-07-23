use error::Result;
use sqlx::PgPool;
use sqlx::types::Json;

use crate::project;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EventCursor {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub id: uuid::Uuid,
}

impl From<&types::events::Event> for EventCursor {
    fn from(event: &types::events::Event) -> Self {
        Self {
            created_at: event.created_at,
            id: event.id,
        }
    }
}

pub struct EventStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> EventStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(&self, id: uuid::Uuid) -> Result<Option<types::events::Event>> {
        let query = format!(
            "SELECT {} FROM events stored_event WHERE stored_event.id = $1",
            project::event("stored_event")
        );

        let event = sqlx::query_scalar::<_, Json<types::events::Event>>(&query)
            .bind(id)
            .fetch_optional(self.pool)
            .await?;

        Ok(event.map(|Json(event)| event))
    }

    pub async fn get_by_trace(&self, trace_id: uuid::Uuid) -> Result<Vec<types::events::Event>> {
        let query = format!(
            r#"
            SELECT {}
            FROM events stored_event
            WHERE stored_event.trace_id = $1
            ORDER BY stored_event.created_at, stored_event.id
            "#,
            project::event("stored_event")
        );
        let events = sqlx::query_scalar::<_, Json<types::events::Event>>(&query)
            .bind(trace_id)
            .fetch_all(self.pool)
            .await?;

        Ok(events.into_iter().map(|Json(event)| event).collect())
    }

    pub async fn get_by_task(&self, task_id: uuid::Uuid) -> Result<Vec<types::events::Event>> {
        let query = format!(
            r#"
            SELECT {}
            FROM events
            WHERE events.task_id = $1
            ORDER BY events.created_at, events.id
            "#,
            project::event("events")
        );

        let events = sqlx::query_scalar::<_, Json<types::events::Event>>(&query)
            .bind(task_id)
            .fetch_all(self.pool)
            .await?;

        Ok(events.into_iter().map(|Json(event)| event).collect())
    }

    pub async fn latest_cursor(&self, tenant_id: uuid::Uuid) -> Result<Option<EventCursor>> {
        let row = sqlx::query_as::<_, (chrono::DateTime<chrono::Utc>, uuid::Uuid)>(
            r#"
            SELECT created_at, id
            FROM events
            WHERE tenant_id = $1
            ORDER BY created_at DESC, id DESC
            LIMIT 1
            "#,
        )
        .bind(tenant_id)
        .fetch_optional(self.pool)
        .await?;

        Ok(row.map(|(created_at, id)| EventCursor { created_at, id }))
    }

    pub async fn list_after(
        &self,
        tenant_id: uuid::Uuid,
        cursor: Option<EventCursor>,
        limit: u32,
    ) -> Result<Vec<types::events::Event>> {
        if limit == 0 {
            return Err(error::bad_request("event replay limit must be greater than zero"));
        }

        let query = format!(
            r#"
            SELECT {}
            FROM events
            WHERE events.tenant_id = $1
              AND (
                  $2::TIMESTAMPTZ IS NULL
                  OR (events.created_at, events.id) > ($2, $3)
              )
            ORDER BY events.created_at, events.id
            LIMIT $4
            "#,
            project::event("events")
        );
        let events = sqlx::query_scalar::<_, Json<types::events::Event>>(&query)
            .bind(tenant_id)
            .bind(cursor.map(|cursor| cursor.created_at))
            .bind(cursor.map(|cursor| cursor.id))
            .bind(i64::from(limit))
            .fetch_all(self.pool)
            .await?;

        Ok(events.into_iter().map(|Json(event)| event).collect())
    }

    pub async fn create(
        &self,
        actor_id: Option<uuid::Uuid>,
        chat_id: Option<uuid::Uuid>,
        message_id: Option<uuid::Uuid>,
        task_id: Option<uuid::Uuid>,
        event: types::events::Event,
    ) -> Result<types::events::Event> {
        sqlx::query(
            r#"
            INSERT INTO events (
                id, trace_id, tenant_id, actor_id, chat_id, message_id, task_id,
                key, data, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
            "#,
        )
        .bind(event.id)
        .bind(event.trace_id)
        .bind(event.tenant_id)
        .bind(actor_id)
        .bind(chat_id)
        .bind(message_id)
        .bind(task_id)
        .bind(&event.key)
        .bind(Json(&event.data))
        .execute(self.pool)
        .await?;

        self.get_by_id(event.id)
            .await?
            .ok_or_else(|| error::Error::from(sqlx::Error::RowNotFound))
    }
}
