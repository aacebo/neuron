use sqlx::PgPool;

use crate::types::Log;

pub struct LogStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> LogStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, log: &Log) -> Result<Log, sqlx::Error> {
        let res = sqlx::query_as::<_, Log>(
            r#"
            INSERT INTO logs (
                id,
                trace_id,
                level,
                source,
                message,
                context,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            RETURNING *
            "#,
        )
        .bind(log.id)
        .bind(log.trace_id)
        .bind(log.level)
        .bind(&log.source)
        .bind(&log.message)
        .bind(&log.context)
        .fetch_one(self.pool)
        .await?;

        Ok(res)
    }

    pub async fn trace(
        &self,
        trace_id: uuid::Uuid,
        source: impl std::fmt::Display,
        message: impl std::fmt::Display,
    ) -> Result<Log, sqlx::Error> {
        let log = Log::trace(trace_id, source, message);
        self.create(&log).await
    }

    pub async fn info(
        &self,
        trace_id: uuid::Uuid,
        source: impl std::fmt::Display,
        message: impl std::fmt::Display,
    ) -> Result<Log, sqlx::Error> {
        let log = Log::info(trace_id, source, message);
        self.create(&log).await
    }

    pub async fn warn(
        &self,
        trace_id: uuid::Uuid,
        source: impl std::fmt::Display,
        message: impl std::fmt::Display,
    ) -> Result<Log, sqlx::Error> {
        let log = Log::warn(trace_id, source, message);
        self.create(&log).await
    }

    pub async fn error(
        &self,
        trace_id: uuid::Uuid,
        source: impl std::fmt::Display,
        message: impl std::fmt::Display,
    ) -> Result<Log, sqlx::Error> {
        let log = Log::error(trace_id, source, message);
        self.create(&log).await
    }
}
