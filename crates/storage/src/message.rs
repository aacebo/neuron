use sqlx::PgPool;

use crate::rows::Message;

pub struct MessageStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> MessageStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_task(&self, task_id: uuid::Uuid) -> Result<Vec<Message>, sqlx::Error> {
        sqlx::query_as::<_, Message>(
            r#"
            SELECT messages.*
            FROM messages_tasks
            LEFT JOIN messages ON messages.id = messages_tasks.message_id
            WHERE messages_tasks.task_id = $1
            "#,
        )
        .bind(task_id)
        .fetch_all(self.pool)
        .await
    }

    pub async fn get(&self, id: uuid::Uuid) -> Result<Option<Message>, sqlx::Error> {
        sqlx::query_as::<_, Message>("SELECT * FROM messages WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool)
            .await
    }

    pub async fn create(&self, message: &Message) -> Result<Message, sqlx::Error> {
        sqlx::query_as::<_, Message>(
            r#"
            INSERT INTO messages (id, source, text, created_at, updated_at)
            VALUES ($1, $2, $3, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(message.id)
        .bind(message.source)
        .bind(&message.text)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(&self, message: &Message) -> Result<Message, sqlx::Error> {
        sqlx::query_as::<_, Message>(
            r#"
            UPDATE messages
            SET source = $2, text = $3, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(message.id)
        .bind(message.source)
        .bind(&message.text)
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM messages WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
