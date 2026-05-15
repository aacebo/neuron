use sqlx::PgPool;

use crate::types::MessageAnnotation;

pub struct AnnotationStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> AnnotationStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: uuid::Uuid) -> Result<Option<MessageAnnotation>, sqlx::Error> {
        sqlx::query_as::<_, MessageAnnotation>("SELECT * FROM message_annotations WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool)
            .await
    }

    pub async fn get_by_message(
        &self,
        message_id: uuid::Uuid,
    ) -> Result<Vec<MessageAnnotation>, sqlx::Error> {
        sqlx::query_as::<_, MessageAnnotation>(
            "SELECT * FROM message_annotations WHERE message_id = $1 ORDER BY created_at",
        )
        .bind(message_id)
        .fetch_all(self.pool)
        .await
    }

    pub async fn create(
        &self,
        annotation: &MessageAnnotation,
    ) -> Result<MessageAnnotation, sqlx::Error> {
        sqlx::query_as::<_, MessageAnnotation>(
            r#"
            INSERT INTO message_annotations
                (id, message_id, type, label, text, score, spans, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
            RETURNING *
            "#,
        )
        .bind(annotation.id)
        .bind(annotation.message_id)
        .bind(&annotation.r#type)
        .bind(&annotation.label)
        .bind(&annotation.text)
        .bind(annotation.score)
        .bind(&annotation.spans)
        .fetch_one(self.pool)
        .await
    }

    pub async fn update(
        &self,
        annotation: &MessageAnnotation,
    ) -> Result<Option<MessageAnnotation>, sqlx::Error> {
        sqlx::query_as::<_, MessageAnnotation>(
            r#"
            UPDATE message_annotations
            SET type = $2, label = $3, text = $4, score = $5, spans = $6
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(annotation.id)
        .bind(&annotation.r#type)
        .bind(&annotation.label)
        .bind(&annotation.text)
        .bind(annotation.score)
        .bind(&annotation.spans)
        .fetch_optional(self.pool)
        .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM message_annotations WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
