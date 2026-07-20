#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Key {
    queue: String,
    action: Action,
}

impl Key {
    pub fn exchange(&self) -> &str {
        "events"
    }
}

impl std::str::FromStr for Key {
    type Err = crate::AMQPError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (queue, action) = value
            .split_once('.')
            .ok_or_else(|| crate::AMQPError::custom("parse", value.to_owned()))?;

        if queue.is_empty() || action.is_empty() || action.contains('.') {
            return Err(crate::AMQPError::custom("parse", value.to_owned()));
        }

        Ok(Self {
            queue: queue.to_owned(),
            action: action.parse()?,
        })
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", &self.queue, &self.action)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Create,
    Update,
    Any,
}

impl Action {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Create => "create",
            Self::Update => "update",
            Self::Any => "*",
        }
    }
}

impl std::str::FromStr for Action {
    type Err = crate::AMQPError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "create" => Ok(Self::Create),
            "update" => Ok(Self::Update),
            "*" => Ok(Self::Any),
            _ => Err(crate::AMQPError::custom("parse", value.to_owned())),
        }
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
