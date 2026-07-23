#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct RoutingKey(String);

impl std::str::FromStr for RoutingKey {
    type Err = error::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        validate_routing_key(value)?;
        Ok(Self(value.to_string()))
    }
}

impl std::fmt::Display for RoutingKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for RoutingKey {
    type Error = error::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<RoutingKey> for String {
    fn from(value: RoutingKey) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct BindingKey(String);

impl std::str::FromStr for BindingKey {
    type Err = error::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        validate_binding_key(value)?;
        Ok(Self(value.to_string()))
    }
}

impl std::fmt::Display for BindingKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl TryFrom<String> for BindingKey {
    type Error = error::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl From<BindingKey> for String {
    fn from(value: BindingKey) -> Self {
        value.0
    }
}

fn validate_routing_key(value: &str) -> error::Result<()> {
    let segments = value.split('.').collect::<Vec<_>>();

    if segments.len() != 2 || segments.iter().any(|segment| !is_literal_segment(segment)) {
        return Err(error::parse(format!("invalid amqp routing key {value}")));
    }

    Ok(())
}

fn validate_binding_key(value: &str) -> error::Result<()> {
    let segments = value.split('.').collect::<Vec<_>>();

    if segments.is_empty()
        || segments
            .iter()
            .any(|segment| !matches!(*segment, "*" | "#") && !is_literal_segment(segment))
    {
        return Err(error::parse(format!("invalid amqp binding key {value}")));
    }

    Ok(())
}

fn is_literal_segment(value: &str) -> bool {
    !value.is_empty() && !value.contains(['*', '#'])
}

#[cfg(test)]
mod tests {
    use super::{BindingKey, RoutingKey};

    #[test]
    fn parses_concrete_routing_keys() {
        let key = "entity.create".parse::<RoutingKey>().unwrap();
        assert_eq!(key.to_string(), "entity.create");
    }

    #[test]
    fn routing_keys_require_two_literal_segments() {
        for value in ["entity", "entity.create.more", "entity.*", "#.create", ".create", "entity."] {
            assert!(value.parse::<RoutingKey>().is_err(), "{value} should be rejected");
        }
    }

    #[test]
    fn parses_topic_binding_keys() {
        for value in ["entity.*", "message.inbound", "#", "entity.#"] {
            let key = value.parse::<BindingKey>().unwrap();
            assert_eq!(key.to_string(), value);
        }
    }

    #[test]
    fn binding_keys_reject_empty_or_partial_wildcard_segments() {
        for value in ["", ".", "entity.", ".create", "ent*ity.create", "entity.cre#ate"] {
            assert!(value.parse::<BindingKey>().is_err(), "{value} should be rejected");
        }
    }

    #[test]
    fn deserialization_preserves_key_validation() {
        assert!(serde_json::from_str::<RoutingKey>(r#""entity.*""#).is_err());
        assert!(serde_json::from_str::<BindingKey>(r#""ent*ity.create""#).is_err());
    }
}
