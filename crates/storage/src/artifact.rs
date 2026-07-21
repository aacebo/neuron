use pgvector::Vector;
use sqlx::PgPool;
use sqlx::types::Json;

use crate::project;

pub struct ArtifactStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> ArtifactStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(&self, id: uuid::Uuid) -> Result<Option<types::resources::Artifact>, sqlx::Error> {
        let query = format!(
            "SELECT {} FROM artifacts artifact WHERE artifact.id = $1",
            project::artifact("artifact")
        );
        let artifact = sqlx::query_scalar::<_, Json<types::resources::Artifact>>(&query)
            .bind(id)
            .fetch_optional(self.pool)
            .await?;

        Ok(artifact.map(|Json(artifact)| artifact))
    }

    pub async fn get_by_message(&self, message_id: uuid::Uuid) -> Result<Vec<types::resources::Artifact>, sqlx::Error> {
        let query = format!(
            r#"
            SELECT {}
            FROM artifacts artifact
            WHERE artifact.message_id = $1
            ORDER BY artifact.created_at, artifact.id
            "#,
            project::artifact("artifact")
        );
        let artifacts = sqlx::query_scalar::<_, Json<types::resources::Artifact>>(&query)
            .bind(message_id)
            .fetch_all(self.pool)
            .await?;

        Ok(artifacts.into_iter().map(|Json(artifact)| artifact).collect())
    }

    pub async fn create(
        &self,
        chat_id: uuid::Uuid,
        message_id: Option<uuid::Uuid>,
        task_id: Option<uuid::Uuid>,
        artifact: types::resources::Artifact,
    ) -> Result<types::resources::Artifact, sqlx::Error> {
        let embedding = artifact.embedding.clone().map(Vector::from);
        sqlx::query(
            r#"
            INSERT INTO artifacts (
                id, chat_id, message_id, task_id, name, content, embedding,
                metadata, created_by_id, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW(), NOW())
            "#,
        )
        .bind(artifact.id)
        .bind(chat_id)
        .bind(message_id)
        .bind(task_id)
        .bind(&artifact.name)
        .bind(Json(&artifact.content))
        .bind(embedding)
        .bind(Json(&artifact.metadata))
        .bind(artifact.created_by.id)
        .execute(self.pool)
        .await?;

        self.get_by_id(artifact.id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn update(&self, artifact: types::resources::Artifact) -> Result<types::resources::Artifact, sqlx::Error> {
        let embedding = artifact.embedding.clone().map(Vector::from);
        let result = sqlx::query(
            r#"
            UPDATE artifacts
            SET name = $2,
                content = $3,
                embedding = $4,
                metadata = $5,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(artifact.id)
        .bind(&artifact.name)
        .bind(Json(&artifact.content))
        .bind(embedding)
        .bind(Json(&artifact.metadata))
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        self.get_by_id(artifact.id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM artifacts WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
