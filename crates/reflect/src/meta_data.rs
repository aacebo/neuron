use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(transparent))]
pub struct MetaData(BTreeMap<String, Arc<dyn crate::ToValue>>);

impl MetaData {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> std::collections::btree_map::Iter<'_, String, Arc<dyn crate::ToValue>> {
        self.0.iter()
    }

    pub fn has(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    pub fn get(&self, key: &str) -> Option<&dyn crate::ToValue> {
        self.0.get(key).map(Arc::as_ref)
    }

    pub fn set<T: crate::ToValue + 'static>(&mut self, key: String, value: T) -> &mut Self {
        self.0.insert(key, Arc::new(value));
        self
    }

    pub fn merge(mut self, other: &Self) -> Self {
        for (key, value) in &other.0 {
            self.0.insert(key.clone(), value.clone());
        }

        self
    }
}

impl<const N: usize, V: crate::ToValue + 'static> From<[(&str, V); N]> for MetaData {
    fn from(items: [(&str, V); N]) -> Self {
        let mut data: BTreeMap<String, Arc<dyn crate::ToValue>> = BTreeMap::new();

        for (key, value) in items {
            data.insert(key.to_string(), Arc::new(value));
        }

        Self(data)
    }
}

impl std::fmt::Debug for MetaData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for MetaData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;

        for (key, value) in &self.0 {
            write!(f, "\n\t{}: {}", key, value.to_value())?;
        }

        if !self.0.is_empty() {
            writeln!(f)?;
        }

        write!(f, "}}")
    }
}

impl crate::TypeOf for MetaData {
    fn type_of() -> crate::Type {
        crate::StructType::new()
            .path(crate::Path::from("reflect"))
            .name("MetaData")
            .visibility(crate::Visibility::Public(crate::Public::Full))
            .build()
            .to_type()
    }
}

impl crate::ToType for MetaData {
    fn to_type(&self) -> crate::Type {
        <Self as crate::TypeOf>::type_of()
    }
}

impl crate::ToValue for MetaData {
    fn to_value(&self) -> crate::Value<'_> {
        crate::Value::Dynamic(crate::Dynamic::from_object(self))
    }
}

impl crate::Object for MetaData {
    fn field(&self, name: &crate::FieldName) -> crate::Value<'_> {
        self.get(&name.to_string()).unwrap().to_value()
    }
}

impl PartialEq for MetaData {
    fn eq(&self, other: &Self) -> bool {
        for (k, a) in &self.0 {
            let b = if let Some(v) = other.get(&k) {
                v
            } else {
                return false;
            };

            if a.to_value() != b.to_value() {
                return false;
            }
        }

        return true;
    }
}
