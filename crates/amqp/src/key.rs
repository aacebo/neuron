#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Key {
    queue: String,
    action: String,
}

impl Key {
    pub fn exchange(&self) -> &str {
        "events"
    }
}

impl std::str::FromStr for Key {
    type Err = error::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (queue, action) = value
            .split_once('.')
            .ok_or_else(|| error::parse(format!("invalid amqp routing key {value}")))?;

        if queue.is_empty() || action.is_empty() || action.contains('.') {
            return Err(error::parse(value));
        }

        Ok(Self {
            queue: queue.to_string(),
            action: action.to_string(),
        })
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", &self.queue, &self.action)
    }
}
