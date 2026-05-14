#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct MapType {
    pub(crate) ty: std::rc::Rc<crate::Type>,
    pub(crate) key: std::rc::Rc<crate::Type>,
    pub(crate) value: std::rc::Rc<crate::Type>,
}

impl MapType {
    pub fn new(ty: crate::Type, key: crate::Type, value: crate::Type) -> Self {
        Self {
            ty: std::rc::Rc::new(ty),
            key: std::rc::Rc::new(key),
            value: std::rc::Rc::new(value),
        }
    }

    pub fn to_type(&self) -> crate::Type {
        crate::Type::Map(std::rc::Rc::new(self.clone()))
    }

    pub fn id(&self) -> crate::TypeId {
        self.ty.id()
    }

    pub fn assignable_to(&self, ty: crate::Type) -> bool {
        self.ty.assignable_to(ty)
    }

    pub fn convertable_to(&self, ty: crate::Type) -> bool {
        self.ty.convertable_to(ty)
    }
}

impl MapType {
    pub fn meta(&self) -> &crate::MetaData {
        self.ty.meta()
    }

    pub fn path(&self) -> &crate::Path {
        self.ty.path()
    }

    pub fn ty(&self) -> &crate::Type {
        &self.ty
    }

    pub fn key(&self) -> &crate::Type {
        &self.key
    }

    pub fn value(&self) -> &crate::Type {
        &self.value
    }
}

impl std::fmt::Display for MapType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.ty)
    }
}

impl crate::ToType for MapType {
    fn to_type(&self) -> crate::Type {
        crate::Type::Map(std::rc::Rc::new(self.clone()))
    }
}

impl<K, V> crate::TypeOf for std::collections::HashMap<K, V>
where
    K: crate::TypeOf,
    V: crate::TypeOf,
{
    fn type_of() -> crate::Type {
        let key = K::type_of();
        let value = V::type_of();
        let ty = crate::StructType::new()
            .path(crate::Path::from("std::collections"))
            .name("HashMap")
            .visibility(crate::Visibility::Public(crate::Public::Full))
            .generics(crate::Generics::from([
                crate::TypeParam::new().name("K").build().to_generic(),
                crate::TypeParam::new().name("V").build().to_generic(),
            ]))
            .build()
            .to_type();

        MapType::new(ty, key, value).to_type()
    }
}

impl<K, V> crate::ToType for std::collections::HashMap<K, V>
where
    K: crate::TypeOf,
    V: crate::TypeOf,
{
    fn to_type(&self) -> crate::Type {
        <Self as crate::TypeOf>::type_of()
    }
}

impl<K, V> crate::TypeOf for std::collections::BTreeMap<K, V>
where
    K: crate::TypeOf,
    V: crate::TypeOf,
{
    fn type_of() -> crate::Type {
        let key = K::type_of();
        let value = V::type_of();
        let ty = crate::StructType::new()
            .path(crate::Path::from("std::collections"))
            .name("BTreeMap")
            .visibility(crate::Visibility::Public(crate::Public::Full))
            .generics(crate::Generics::from([
                crate::TypeParam::new().name("K").build().to_generic(),
                crate::TypeParam::new().name("V").build().to_generic(),
            ]))
            .build()
            .to_type();

        MapType::new(ty, key, value).to_type()
    }
}

impl<K, V> crate::ToType for std::collections::BTreeMap<K, V>
where
    K: crate::TypeOf,
    V: crate::TypeOf,
{
    fn to_type(&self) -> crate::Type {
        <Self as crate::TypeOf>::type_of()
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::{TypeOf, type_of};

    #[test]
    pub fn type_of() {
        let ty = type_of!(HashMap<String, bool>);

        assert!(ty.is_map());
        assert_eq!(ty.as_map().key(), &type_of!(String));
        assert_eq!(ty.as_map().value(), &type_of!(bool));
    }
}
