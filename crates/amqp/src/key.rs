#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Key {
    entity: String,
    action: Action,
}

impl Key {
    pub fn new(entity: &str, action: Action) -> Self {
        Self {
            entity: entity.to_string(),
            action,
        }
    }

    pub fn exchange(&self) -> &str {
        &self.entity
    }

    pub fn queue(&self) -> &str {
        self.action.name()
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", &self.entity, &self.action)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Create,
    Update,
}

impl Action {
    pub fn name(&self) -> &str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
        }
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
