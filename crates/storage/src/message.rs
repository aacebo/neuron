use pgvector::Vector;
use sqlx::PgPool;
use sqlx::types::Json;

use crate::project;

pub struct MessageStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> MessageStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(&self, id: uuid::Uuid) -> Result<Option<types::chats::Message>, sqlx::Error> {
        let query = format!(
            "SELECT {} FROM messages message WHERE message.id = $1",
            project::message("message")
        );

        let message = sqlx::query_scalar::<_, Json<types::chats::Message>>(&query)
            .bind(id)
            .fetch_optional(self.pool)
            .await?;

        Ok(message.map(|Json(message)| message))
    }

    pub async fn get_by_task(&self, task_id: uuid::Uuid) -> Result<Option<types::chats::Message>, sqlx::Error> {
        let query = format!(
            r#"
            SELECT {}
            FROM messages message
            JOIN tasks task ON task.message_id = message.id
            WHERE task.id = $1
            "#,
            project::message("message")
        );

        let message = sqlx::query_scalar::<_, Json<types::chats::Message>>(&query)
            .bind(task_id)
            .fetch_optional(self.pool)
            .await?;

        Ok(message.map(|Json(message)| message))
    }

    pub async fn create(&self, message: types::chats::Message) -> Result<types::chats::Message, sqlx::Error> {
        let embedding = message.embedding.clone().map(Vector::from);

        sqlx::query(
            r#"
            INSERT INTO messages (
                id, chat_id, content, metadata, embedding, created_by_id, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            "#,
        )
        .bind(message.id)
        .bind(message.chat.id)
        .bind(Json(&message.content))
        .bind(Json(&message.metadata))
        .bind(embedding)
        .bind(message.created_by.id)
        .execute(self.pool)
        .await?;

        self.get_by_id(message.id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn update(&self, message: types::chats::Message) -> Result<types::chats::Message, sqlx::Error> {
        let embedding = message.embedding.clone().map(Vector::from);
        let result = sqlx::query(
            r#"
            UPDATE messages
            SET content = $2,
                metadata = $3,
                embedding = $4,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(message.id)
        .bind(Json(&message.content))
        .bind(Json(&message.metadata))
        .bind(embedding)
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        self.get_by_id(message.id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn update_embedding(&self, id: uuid::Uuid, embedding: Vec<f32>) -> Result<types::chats::Message, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE messages
            SET embedding = $2,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(Vector::from(embedding))
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        self.get_by_id(id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM messages WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
