use sqlx::PgPool;

use crate::types::Message;

pub struct MessageStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> MessageStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
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
            INSERT INTO messages (id, text, source, artifacts, annotations, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            RETURNING *
            "#,
        )
        .bind(message.id)
        .bind(&message.text)
        .bind(message.source)
        .bind(&message.artifacts)
        .bind(&message.annotations)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(&self, message: &Message) -> Result<Option<Message>, sqlx::Error> {
        sqlx::query_as::<_, Message>(
            r#"
            UPDATE messages
            SET text = $2, source = $3, artifacts = $4, annotations = $5, updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(message.id)
        .bind(&message.text)
        .bind(message.source)
        .bind(&message.artifacts)
        .bind(&message.annotations)
        .fetch_optional(self.pool)
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
