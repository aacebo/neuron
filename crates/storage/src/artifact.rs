use sqlx::PgPool;

use crate::types::MessageArtifact;

pub struct ArtifactStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> ArtifactStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: uuid::Uuid) -> Result<Option<MessageArtifact>, sqlx::Error> {
        sqlx::query_as::<_, MessageArtifact>("SELECT * FROM message_artifacts WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool)
            .await
    }

    pub async fn get_by_message(
        &self,
        message_id: uuid::Uuid,
    ) -> Result<Vec<MessageArtifact>, sqlx::Error> {
        sqlx::query_as::<_, MessageArtifact>(
            "SELECT * FROM message_artifacts WHERE message_id = $1 ORDER BY created_at",
        )
        .bind(message_id)
        .fetch_all(self.pool)
        .await
    }

    pub async fn create(&self, artifact: &MessageArtifact) -> Result<MessageArtifact, sqlx::Error> {
        sqlx::query_as::<_, MessageArtifact>(
            r#"
            INSERT INTO message_artifacts
                (id, message_id, type, content, embedding, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            RETURNING *
            "#,
        )
        .bind(artifact.id)
        .bind(artifact.message_id)
        .bind(&artifact.r#type)
        .bind(&artifact.content)
        .bind(&artifact.embedding)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(
        &self,
        artifact: &MessageArtifact,
    ) -> Result<Option<MessageArtifact>, sqlx::Error> {
        sqlx::query_as::<_, MessageArtifact>(
            r#"
            UPDATE message_artifacts
            SET type = $2, content = $3, embedding = $4
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(artifact.id)
        .bind(&artifact.r#type)
        .bind(&artifact.content)
        .bind(&artifact.embedding)
        .fetch_optional(self.pool)
        .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM message_artifacts WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
