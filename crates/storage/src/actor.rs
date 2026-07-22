use error::Result;
use pgvector::Vector;
use sqlx::PgPool;
use sqlx::types::Json;

use crate::project;

fn postgres_instances(instances: u32) -> Result<i32> {
    i32::try_from(instances).map_err(|_| error::sql(format!("agent instance count {instances} exceeds PostgreSQL INT")))
}

pub struct ActorStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> ActorStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(&self, id: uuid::Uuid) -> Result<Option<types::actors::Actor>> {
        let query = format!("SELECT {} FROM actors actor WHERE actor.id = $1", project::actor("actor"));
        let actor = sqlx::query_scalar::<_, Json<types::actors::Actor>>(&query)
            .bind(id)
            .fetch_optional(self.pool)
            .await?;

        Ok(actor.map(|Json(actor)| actor))
    }

    pub async fn create(&self, actor: types::actors::Actor) -> Result<types::actors::Actor> {
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
            let instances = postgres_instances(agent.instances)?;
            sqlx::query(
                r#"
                INSERT INTO agents (actor_id, status, description, secret, instances, skills)
                VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(actor.id)
            .bind(agent.status.as_str())
            .bind(&agent.description)
            .bind(&agent.secret)
            .bind(instances)
            .bind(Json(&agent.skills))
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(self.get_by_id(actor.id).await?.ok_or(sqlx::Error::RowNotFound)?)
    }

    pub async fn update(&self, actor: types::actors::Actor) -> Result<types::actors::Actor> {
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
            return Err(sqlx::Error::RowNotFound.into());
        }

        if let Some(agent) = &actor.agent {
            let instances = postgres_instances(agent.instances)?;
            sqlx::query(
                r#"
                INSERT INTO agents (actor_id, status, description, secret, instances, skills)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (actor_id) DO UPDATE
                SET status = EXCLUDED.status,
                    description = EXCLUDED.description,
                    secret = EXCLUDED.secret,
                    instances = EXCLUDED.instances,
                    skills = EXCLUDED.skills
                "#,
            )
            .bind(actor.id)
            .bind(agent.status.as_str())
            .bind(&agent.description)
            .bind(&agent.secret)
            .bind(instances)
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
        Ok(self.get_by_id(actor.id).await?.ok_or(sqlx::Error::RowNotFound)?)
    }

    pub async fn update_embedding(&self, id: uuid::Uuid, embedding: Vec<f32>) -> Result<types::actors::Actor> {
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
            return Err(sqlx::Error::RowNotFound.into());
        }

        Ok(self.get_by_id(id).await?.ok_or(sqlx::Error::RowNotFound)?)
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM actors WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
