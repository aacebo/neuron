use sqlx::PgPool;

use crate::types::{Job, JobSource};

pub struct JobStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> JobStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: uuid::Uuid) -> Result<Option<Job>, sqlx::Error> {
        sqlx::query_as::<_, Job>("SELECT * FROM jobs WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool)
            .await
    }

    pub async fn get_by_message(&self, message_id: uuid::Uuid) -> Result<Vec<Job>, sqlx::Error> {
        sqlx::query_as::<_, Job>(
            r#"
            SELECT jobs.*
            FROM messages_jobs
            LEFT JOIN jobs
                ON jobs.id = messages_jobs.job_id
            WHERE message_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(message_id)
        .fetch_all(self.pool)
        .await
    }

    pub async fn create(&self, job: &Job, source: JobSource) -> Result<Job, sqlx::Error> {
        let res = sqlx::query_as::<_, Job>(
            r#"
            INSERT INTO jobs (
                id,
                name,
                status,
                error,
                attempts,
                max_attempts,
                started_at,
                ended_at,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(&job.id)
        .bind(&job.name)
        .bind(&job.status)
        .bind(&job.error)
        .bind(&job.attempts)
        .bind(&job.max_attempts)
        .bind(&job.started_at)
        .bind(&job.ended_at)
        .fetch_one(self.pool)
        .await?;

        #[allow(irrefutable_let_patterns)]
        if let JobSource::Message(message_id) = source {
            sqlx::query(
                r#"
                INSERT INTO messages_jobs (
                    message_id,
                    job_id,
                    created_at
                )
                VALUES ($1, $2, NOW())
                "#,
            )
            .bind(&job.id)
            .bind(&message_id)
            .execute(self.pool)
            .await?;
        }

        Ok(res)
    }

    pub async fn update(&self, job: &Job) -> Result<Option<Job>, sqlx::Error> {
        sqlx::query_as::<_, Job>(
            r#"
            UPDATE jobs
            SET status = $2,
                error = $3,
                attempts = $4,
                started_at = $5,
                ended_at = $6,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(&job.id)
        .bind(&job.status)
        .bind(&job.error)
        .bind(&job.attempts)
        .bind(&job.started_at)
        .bind(&job.ended_at)
        .fetch_optional(self.pool)
        .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM jobs WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
