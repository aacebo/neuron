use sqlx::PgPool;
use sqlx::types::Json;

use crate::{Error, Result, project};

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
            FROM events stored_event
            WHERE stored_event.task_id = $1
            ORDER BY stored_event.created_at, stored_event.id
            "#,
            project::event("stored_event")
        );

        let events = sqlx::query_scalar::<_, Json<types::events::Event>>(&query)
            .bind(task_id)
            .fetch_all(self.pool)
            .await?;

        Ok(events.into_iter().map(|Json(event)| event).collect())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        tenant_id: uuid::Uuid,
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
        .bind(tenant_id)
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
            .ok_or_else(|| Error::from(sqlx::Error::RowNotFound))
    }
}
