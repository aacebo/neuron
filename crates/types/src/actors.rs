use crate::data;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Actor {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub external_id: Option<String>,
    pub role: Role,
    pub name: String,
    pub display_name: String,
    #[serde(flatten)]
    pub agent: Option<Agent>,
    pub metadata: data::Metadata,
    pub embedding: Option<Vec<f32>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ActorPartial {
    pub id: uuid::Uuid,
    pub role: Role,
    pub name: String,
    pub display_name: String,
    #[serde(flatten)]
    pub agent: Option<Agent>,
}

impl From<Actor> for ActorPartial {
    fn from(value: Actor) -> Self {
        Self {
            id: value.id,
            role: value.role,
            name: value.name,
            display_name: value.display_name,
            agent: value.agent,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Agent,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Agent => "agent",
        }
    }

    pub fn is_user(self) -> bool {
        matches!(self, Self::User)
    }

    pub fn is_agent(self) -> bool {
        matches!(self, Self::Agent)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Agent {
    pub status: AgentStatus,
    pub description: String,
    pub skills: Vec<Skill>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Online,
    Offline,
}

impl AgentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Offline => "offline",
        }
    }

    pub fn is_online(self) -> bool {
        matches!(self, Self::Online)
    }

    pub fn is_offline(self) -> bool {
        matches!(self, Self::Offline)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Skill {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
}
