use crate::ToType;

#[macro_export]
macro_rules! map {
    () => {{ ::std::collections::HashMap::new() }};
    ( $( { $key:expr, $value:expr } ),+ $(,)? ) => {{
        ::std::collections::HashMap::from([ $(($key, $value),)+ ])
    }};
    ( $( $key:expr => $value:expr ),+ $(,)? ) => {{
        ::std::collections::HashMap::from([ $(($key, $value),)+ ])
    }};
}

#[macro_export]
macro_rules! btree_map {
    () => {{ ::std::collections::BTreeMap::new() }};
    ( $( { $key:expr, $value:expr } ),+ $(,)? ) => {{
        ::std::collections::BTreeMap::from([ $(($key, $value),)+ ])
    }};
    ( $( $key:expr => $value:expr ),+ $(,)? ) => {{
        ::std::collections::BTreeMap::from([ $(($key, $value),)+ ])
    }};
}

#[derive(Debug, Clone, PartialEq)]
pub struct Map<'a> {
    pub(crate) ty: crate::MapType,
    pub(crate) data: std::collections::BTreeMap<crate::Value<'a>, crate::Value<'a>>,
}

impl<'a> Map<'a> {
    pub fn new(ty: &crate::MapType) -> Self {
        Self {
            ty: ty.clone(),
            data: std::collections::BTreeMap::new(),
        }
    }

    pub fn to_type(&self) -> crate::Type {
        crate::Type::Map(std::rc::Rc::new(self.ty.clone()))
    }

    pub fn iter(
        &self,
    ) -> std::collections::btree_map::Iter<'_, crate::Value<'a>, crate::Value<'a>> {
        self.data.iter()
    }

    pub fn keys(
        &self,
    ) -> std::collections::btree_map::Keys<'_, crate::Value<'a>, crate::Value<'a>> {
        self.data.keys()
    }

    pub fn values(
        &self,
    ) -> std::collections::btree_map::Values<'_, crate::Value<'a>, crate::Value<'a>> {
        self.data.values()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn data(&self) -> &std::collections::BTreeMap<crate::Value<'a>, crate::Value<'a>> {
        &self.data
    }

    pub fn has(&self, key: &crate::Value<'a>) -> bool {
        self.data.contains_key(key)
    }

    pub fn get(&self, key: &crate::Value<'a>) -> Option<&crate::Value<'a>> {
        self.data.get(key)
    }

    pub fn insert(&mut self, key: crate::Value<'a>, value: crate::Value<'a>) {
        self.data.insert(key, value);
    }
}

#[cfg(feature = "serde")]
impl<'a> serde::Serialize for Map<'a> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut ser = s.serialize_map(Some(self.data.len()))?;

        for (k, v) in &self.data {
            ser.serialize_entry(k, v)?;
        }
        ser.end()
    }
}

impl<'a> crate::ToType for Map<'a> {
    fn to_type(&self) -> crate::Type {
        crate::Type::Map(std::rc::Rc::new(self.ty.clone()))
    }
}

impl<'a> crate::ToValue for Map<'a> {
    fn to_value(&self) -> crate::Value<'_> {
        crate::Value::Map(self.clone())
    }
}

impl<'a> std::ops::Index<&crate::Value<'a>> for Map<'a> {
    type Output = crate::Value<'a>;

    fn index(&self, index: &crate::Value<'a>) -> &Self::Output {
        self.data.index(index)
    }
}

impl<'a> std::fmt::Display for Map<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;

        for (key, value) in &self.data {
            write!(f, "\n\t{}: {}", key, value)?;
        }
        if !self.data.is_empty() {
            writeln!(f)?;
        }
        write!(f, "}}")
    }
}

impl<K, V> crate::ToValue for std::collections::HashMap<K, V>
where
    K: crate::TypeOf + crate::ToValue,
    V: crate::TypeOf + crate::ToValue,
{
    fn to_value(&self) -> crate::Value<'_> {
        let ty = self.to_type();
        let mut map = Map::new(ty.as_map());

        for (k, v) in self {
            map.insert(k.to_value(), v.to_value());
        }
        crate::Value::Map(map)
    }
}

impl<K, V> crate::ToValue for std::collections::BTreeMap<K, V>
where
    K: crate::TypeOf + crate::ToValue,
    V: crate::TypeOf + crate::ToValue,
{
    fn to_value(&self) -> crate::Value<'_> {
        let ty = self.to_type();
        let mut map = Map::new(ty.as_map());

        for (k, v) in self {
            map.insert(k.to_value(), v.to_value());
        }
        crate::Value::Map(map)
    }
}

#[cfg(test)]
mod test {
    use crate::value_of;

    #[test]
    pub fn to_value() {
        let map = btree_map! {
            "hello".to_string() => 123_i32,
            "world".to_string() => 111_i32
        };
        let value = value_of!(map);

        assert!(value.is_map());
        assert_eq!(value.len(), 2);
        assert_eq!(value["hello"], value_of!(123_i32));
    }
}
