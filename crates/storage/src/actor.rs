use pgvector::Vector;
use sqlx::PgPool;
use sqlx::types::Json;

use crate::project;

pub struct ActorStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> ActorStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(&self, id: uuid::Uuid) -> Result<Option<types::actors::Actor>, sqlx::Error> {
        let query = format!("SELECT {} FROM actors actor WHERE actor.id = $1", project::actor("actor"));
        let actor = sqlx::query_scalar::<_, Json<types::actors::Actor>>(&query)
            .bind(id)
            .fetch_optional(self.pool)
            .await?;

        Ok(actor.map(|Json(actor)| actor))
    }

    pub async fn create(&self, actor: types::actors::Actor) -> Result<types::actors::Actor, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let embedding = actor.embedding.clone().map(Vector::from);

        sqlx::query(
            r#"
            INSERT INTO actors (
                id, tenant_id, external_id, role, name, display_name, metadata,
                embedding, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
            "#,
        )
        .bind(actor.id)
        .bind(actor.tenant_id)
        .bind(&actor.external_id)
        .bind(actor.role.as_str())
        .bind(&actor.name)
        .bind(&actor.display_name)
        .bind(Json(&actor.metadata))
        .bind(embedding)
        .execute(&mut *tx)
        .await?;

        if let Some(agent) = &actor.agent {
            sqlx::query(
                r#"
                INSERT INTO agents (actor_id, status, description, skills)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(actor.id)
            .bind(agent.status.as_str())
            .bind(&agent.description)
            .bind(Json(&agent.skills))
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        self.get_by_id(actor.id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn update(&self, actor: types::actors::Actor) -> Result<types::actors::Actor, sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let embedding = actor.embedding.clone().map(Vector::from);
        let result = sqlx::query(
            r#"
            UPDATE actors
            SET tenant_id = $2,
                external_id = $3,
                role = $4,
                name = $5,
                display_name = $6,
                metadata = $7,
                embedding = $8,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(actor.id)
        .bind(actor.tenant_id)
        .bind(&actor.external_id)
        .bind(actor.role.as_str())
        .bind(&actor.name)
        .bind(&actor.display_name)
        .bind(Json(&actor.metadata))
        .bind(embedding)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        if let Some(agent) = &actor.agent {
            sqlx::query(
                r#"
                INSERT INTO agents (actor_id, status, description, skills)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (actor_id) DO UPDATE
                SET status = EXCLUDED.status,
                    description = EXCLUDED.description,
                    skills = EXCLUDED.skills
                "#,
            )
            .bind(actor.id)
            .bind(agent.status.as_str())
            .bind(&agent.description)
            .bind(Json(&agent.skills))
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query("DELETE FROM agents WHERE actor_id = $1")
                .bind(actor.id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        self.get_by_id(actor.id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn update_embedding(&self, id: uuid::Uuid, embedding: Vec<f32>) -> Result<types::actors::Actor, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE actors
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
        let result = sqlx::query("DELETE FROM actors WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
