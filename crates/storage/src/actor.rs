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

    pub async fn get(&self, id: uuid::Uuid) -> Result<Option<types::actors::Actor>, sqlx::Error> {
        let query = format!("SELECT {} FROM actors actor WHERE actor.id = $1", project::actor("actor"));
        let actor = sqlx::query_scalar::<_, Json<types::actors::Actor>>(&query)
            .bind(id)
            .fetch_optional(self.pool)
            .await?;

        Ok(actor.map(|Json(actor)| actor))
    }

    pub async fn create(&self, actor: types::actors::Actor) -> Result<types::actors::Actor, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            INSERT INTO actors (
                id, tenant_id, external_id, role, name, display_name, metadata,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW())
            "#,
        )
        .bind(actor.id)
        .bind(actor.tenant_id)
        .bind(&actor.external_id)
        .bind(actor.role.as_str())
        .bind(&actor.name)
        .bind(&actor.display_name)
        .bind(Json(&actor.metadata))
        .execute(&mut *tx)
        .await?;

        if let Some(agent) = &actor.agent {
            sqlx::query(
                r#"
                INSERT INTO agents (actor_id, status, description)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(actor.id)
            .bind(agent.status.as_str())
            .bind(&agent.description)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        self.get(actor.id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn update(&self, actor: types::actors::Actor) -> Result<types::actors::Actor, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let result = sqlx::query(
            r#"
            UPDATE actors
            SET tenant_id = $2,
                external_id = $3,
                role = $4,
                name = $5,
                display_name = $6,
                metadata = $7,
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
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        if let Some(agent) = &actor.agent {
            sqlx::query(
                r#"
                INSERT INTO agents (actor_id, status, description)
                VALUES ($1, $2, $3)
                ON CONFLICT (actor_id) DO UPDATE
                SET status = EXCLUDED.status,
                    description = EXCLUDED.description
                "#,
            )
            .bind(actor.id)
            .bind(agent.status.as_str())
            .bind(&agent.description)
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query("DELETE FROM agents WHERE actor_id = $1")
                .bind(actor.id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        self.get(actor.id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn set_skill_versions(
        &self,
        actor_id: uuid::Uuid,
        skill_version_ids: &[uuid::Uuid],
    ) -> Result<types::actors::Actor, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query("DELETE FROM agent_skills WHERE agent_id = $1")
            .bind(actor_id)
            .execute(&mut *tx)
            .await?;

        sqlx::query(
            r#"
            INSERT INTO agent_skills (agent_id, skill_version_id, created_at)
            SELECT $1, skill_version_id, NOW()
            FROM (
                SELECT DISTINCT skill_version_id
                FROM UNNEST($2::UUID[]) AS version(skill_version_id)
            ) versions
            "#,
        )
        .bind(actor_id)
        .bind(skill_version_ids.to_vec())
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        self.get(actor_id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM actors WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
