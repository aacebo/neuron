use sqlx::PgPool;

use crate::rows::Annotation;

pub struct AnnotationStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> AnnotationStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: uuid::Uuid) -> Result<Option<Annotation>, sqlx::Error> {
        sqlx::query_as::<_, Annotation>("SELECT * FROM annotations WHERE id = $1")
            .bind(id)
            .fetch_optional(self.pool)
            .await
    }

    pub async fn get_by_message(&self, message_id: uuid::Uuid) -> Result<Vec<Annotation>, sqlx::Error> {
        sqlx::query_as::<_, Annotation>(
            "SELECT * FROM annotations WHERE message_id = $1 ORDER BY score DESC, created_at ASC, id ASC",
        )
        .bind(message_id)
        .fetch_all(self.pool)
        .await
    }

    pub async fn create(&self, annotation: &Annotation) -> Result<Annotation, sqlx::Error> {
        sqlx::query_as::<_, Annotation>(
            r#"
            INSERT INTO annotations
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

    pub async fn update(&self, annotation: &Annotation) -> Result<Annotation, sqlx::Error> {
        sqlx::query_as::<_, Annotation>(
            r#"
            UPDATE annotations
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
        .fetch_one(self.pool)
        .await
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM annotations WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
