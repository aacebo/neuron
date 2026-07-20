#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Actor {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub external_id: String,
    pub role: Role,
    pub name: String,
    pub display_name: String,
    pub metadata: sqlx::types::Json<types::data::Metadata>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum Role {
    User,
    Agent,
}

impl Role {
    pub fn is_user(self) -> bool {
        matches!(self, Self::User)
    }

    pub fn is_agent(self) -> bool {
        matches!(self, Self::Agent)
    }
}

impl From<Role> for types::actors::Role {
    fn from(value: Role) -> Self {
        match value {
            Role::Agent => types::actors::Role::Agent,
            Role::User => types::actors::Role::User,
        }
    }
}
