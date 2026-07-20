use sqlx::PgPool;

use crate::rows;

pub struct ActorStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> ActorStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: uuid::Uuid) -> Result<types::actors::Actor, sqlx::Error> {
        let actor = sqlx::query_as::<_, rows::Actor>(
            r#"
            SELECT *
            FROM actors
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_one(self.pool)
        .await?;

        let agent = sqlx::query_as::<_, rows::Agent>(
            r#"
            SELECT *
            FROM agents
            WHERE actor_id = $1
            "#,
        )
        .bind(actor.id)
        .fetch_optional(self.pool)
        .await?;

        Ok(types::actors::Actor {
            id: actor.id,
            tenant_id: actor.tenant_id,
            external_id: actor.external_id,
            role: actor.role.into(),
            name: actor.name,
            display_name: actor.display_name,
            agent: if let Some(a) = agent {
                let skills = sqlx::query_as::<_, rows::Skill>(
                    r#"
                    SELECT *
                    FROM skills
                    "#
                )
                .fetch_all(self.pool)
                .await?;

                Some(types::actors::Agent {
                    status: a.status.into(),
                    description: a.description,
                    skills:
                })
            } else {
                None
            },
        })
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
