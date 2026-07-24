use error::Result;
use sqlx::PgPool;
use sqlx::types::Json;

use crate::project;

pub struct LogStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> LogStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(&self, id: uuid::Uuid) -> Result<Option<types::logs::Log>> {
        let query = format!("SELECT {} FROM logs WHERE id = $1", project::log("logs"));
        let log = sqlx::query_scalar::<_, Json<types::logs::Log>>(&query)
            .bind(id)
            .fetch_optional(self.pool)
            .await?;

        Ok(log.map(|Json(log)| log))
    }

    pub async fn get_by_trace(&self, trace_id: uuid::Uuid) -> Result<Vec<types::logs::Log>> {
        let query = format!(
            r#"
            SELECT {}
            FROM logs
            WHERE logs.trace_id = $1
            ORDER BY logs.created_at DESC, logs.id
            "#,
            project::log("logs")
        );

        let logs = sqlx::query_scalar::<_, Json<types::logs::Log>>(&query)
            .bind(trace_id)
            .fetch_all(self.pool)
            .await?;

        Ok(logs.into_iter().map(|Json(log)| log).collect())
    }

    pub async fn get_by_tenant(&self, tenant_id: uuid::Uuid) -> Result<Vec<types::logs::Log>> {
        let query = format!(
            r#"
            SELECT {}
            FROM logs
            WHERE logs.tenant_id = $1
            ORDER BY logs.created_at DESC, logs.id
            "#,
            project::log("logs")
        );

        let logs = sqlx::query_scalar::<_, Json<types::logs::Log>>(&query)
            .bind(tenant_id)
            .fetch_all(self.pool)
            .await?;

        Ok(logs.into_iter().map(|Json(log)| log).collect())
    }

    pub async fn get_by_task(&self, task_id: uuid::Uuid) -> Result<Vec<types::logs::Log>> {
        let query = format!(
            r#"
            SELECT {}
            FROM logs
            WHERE logs.task_id = $1
            ORDER BY logs.created_at DESC, logs.id
            "#,
            project::log("logs")
        );

        let logs = sqlx::query_scalar::<_, Json<types::logs::Log>>(&query)
            .bind(task_id)
            .fetch_all(self.pool)
            .await?;

        Ok(logs.into_iter().map(|Json(log)| log).collect())
    }

    pub async fn create(&self, log: types::logs::Log) -> Result<types::logs::Log> {
        sqlx::query(
            r#"
            INSERT INTO logs (
                id, trace_id, tenant_id, task_id, level,
                source, message, fields, created_by_id, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
            "#,
        )
        .bind(log.id)
        .bind(log.trace_id)
        .bind(log.tenant_id)
        .bind(log.task_id)
        .bind(log.level.as_str())
        .bind(log.source)
        .bind(log.message)
        .bind(Json(log.fields))
        .bind(log.created_by.id)
        .execute(self.pool)
        .await?;

        self.get_by_id(log.id)
            .await?
            .ok_or_else(|| error::Error::from(sqlx::Error::RowNotFound))
    }
}
