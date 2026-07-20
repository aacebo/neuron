use sqlx::PgPool;
use sqlx::types::Json;

use crate::project;

pub struct SkillStorage<'a> {
    pool: &'a PgPool,
}

impl<'a> SkillStorage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn get(&self, id: uuid::Uuid) -> Result<Option<types::skills::Skill>, sqlx::Error> {
        let query = format!("SELECT {} FROM skills skill WHERE skill.id = $1", project::skill("skill"));
        let skill = sqlx::query_scalar::<_, Json<types::skills::Skill>>(&query)
            .bind(id)
            .fetch_optional(self.pool)
            .await?;

        Ok(skill.map(|Json(skill)| skill))
    }

    pub async fn create(&self, skill: types::skills::Skill) -> Result<types::skills::Skill, sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO skills (id, tenant_id, name, display_name, created_at)
            VALUES ($1, $2, $3, $4, NOW())
            "#,
        )
        .bind(skill.id)
        .bind(skill.tenant_id)
        .bind(&skill.name)
        .bind(&skill.display_name)
        .execute(self.pool)
        .await?;

        self.get(skill.id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn update(&self, skill: types::skills::Skill) -> Result<types::skills::Skill, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE skills
            SET name = $2,
                display_name = $3
            WHERE id = $1
            "#,
        )
        .bind(skill.id)
        .bind(&skill.name)
        .bind(&skill.display_name)
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        self.get(skill.id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn get_version(&self, id: uuid::Uuid) -> Result<Option<types::skills::Version>, sqlx::Error> {
        let query = format!(
            "SELECT {} FROM skill_versions skill_version WHERE skill_version.id = $1",
            project::version("skill_version")
        );
        let version = sqlx::query_scalar::<_, Json<types::skills::Version>>(&query)
            .bind(id)
            .fetch_optional(self.pool)
            .await?;

        Ok(version.map(|Json(version)| version))
    }

    pub async fn get_versions(&self, skill_id: uuid::Uuid) -> Result<Vec<types::skills::Version>, sqlx::Error> {
        let query = format!(
            r#"
            SELECT {}
            FROM skill_versions skill_version
            WHERE skill_version.skill_id = $1
            ORDER BY skill_version.major DESC, skill_version.minor DESC, skill_version.patch DESC,
                     skill_version.prerelease DESC NULLS FIRST, skill_version.id
            "#,
            project::version("skill_version")
        );
        let versions = sqlx::query_scalar::<_, Json<types::skills::Version>>(&query)
            .bind(skill_id)
            .fetch_all(self.pool)
            .await?;

        Ok(versions.into_iter().map(|Json(version)| version).collect())
    }

    pub async fn create_version(
        &self,
        skill_id: uuid::Uuid,
        version: types::skills::Version,
    ) -> Result<types::skills::Version, sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO skill_versions (
                id, skill_id, major, minor, patch, prerelease, status,
                description, tags, input, output, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW())
            "#,
        )
        .bind(version.id)
        .bind(skill_id)
        .bind(version.major)
        .bind(version.minor)
        .bind(version.patch)
        .bind(&version.prerelease)
        .bind(version.status.as_str())
        .bind(&version.description)
        .bind(&version.tags)
        .bind(&version.input)
        .bind(&version.output)
        .execute(self.pool)
        .await?;

        self.get_version(version.id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn update_version(&self, version: types::skills::Version) -> Result<types::skills::Version, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE skill_versions
            SET major = $2,
                minor = $3,
                patch = $4,
                prerelease = $5,
                status = $6,
                description = $7,
                tags = $8,
                input = $9,
                output = $10,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(version.id)
        .bind(version.major)
        .bind(version.minor)
        .bind(version.patch)
        .bind(&version.prerelease)
        .bind(version.status.as_str())
        .bind(&version.description)
        .bind(&version.tags)
        .bind(&version.input)
        .bind(&version.output)
        .execute(self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(sqlx::Error::RowNotFound);
        }

        self.get_version(version.id).await?.ok_or(sqlx::Error::RowNotFound)
    }

    pub async fn delete_version(&self, id: uuid::Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM skill_versions WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn delete(&self, id: uuid::Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM skills WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
