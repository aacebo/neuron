use sqlx::PgPool;

use crate::rows::{JobSource, Task};

pub struct TaskStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> TaskStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: uuid::Uuid) -> Result<Option<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool)
            .await
    }

    pub async fn get_by_message(&self, message_id: uuid::Uuid) -> Result<Vec<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>(
            r#"
            SELECT tasks.*
            FROM messages_tasks
            LEFT JOIN tasks
                ON tasks.id = messages_tasks.task_id
            WHERE message_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(message_id)
        .fetch_all(self.pool)
        .await
    }

    pub async fn create(&self, task: &Task, source: JobSource) -> Result<Task, sqlx::Error> {
        let res = sqlx::query_as::<_, Task>(
            r#"
            INSERT INTO tasks (
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
        .bind(task.id)
        .bind(&task.name)
        .bind(task.status)
        .bind(&task.error)
        .bind(task.attempts)
        .bind(task.max_attempts)
        .bind(task.started_at)
        .bind(task.ended_at)
        .fetch_one(self.pool)
        .await?;

        #[allow(irrefutable_let_patterns)]
        if let JobSource::Message(message_id) = source {
            sqlx::query(
                r#"
                INSERT INTO messages_tasks (
                    message_id,
                    task_id,
                    created_at
                )
                VALUES ($1, $2, NOW())
                "#,
            )
            .bind(message_id)
            .bind(task.id)
            .execute(self.pool)
            .await?;
        }

        Ok(res)
    }

    pub async fn update(&self, task: &Task) -> Result<Task, sqlx::Error> {
        sqlx::query_as::<_, Task>(
            r#"
            UPDATE tasks
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
        .bind(task.id)
        .bind(task.status)
        .bind(&task.error)
        .bind(task.attempts)
        .bind(task.started_at)
        .bind(task.ended_at)
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
