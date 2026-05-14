#[derive(Debug, Copy, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StrType;

impl StrType {
    pub fn to_type(&self) -> crate::Type {
        crate::Type::Str(*self)
    }

    pub fn id(&self) -> crate::TypeId {
        crate::TypeId::from_str("string")
    }

    pub fn assignable_to(&self, ty: crate::Type) -> bool {
        self.id() == ty.id()
    }

    pub fn convertable_to(&self, ty: crate::Type) -> bool {
        ty.is_str()
    }
}

impl std::fmt::Display for StrType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}

impl crate::ToType for StrType {
    fn to_type(&self) -> crate::Type {
        crate::Type::Str(*self)
    }
}

impl crate::TypeOf for std::string::String {
    fn type_of() -> crate::Type {
        crate::Type::Str(StrType)
    }
}

impl crate::ToType for std::string::String {
    fn to_type(&self) -> crate::Type {
        crate::Type::Str(StrType)
    }
}

impl crate::TypeOf for str {
    fn type_of() -> crate::Type {
        crate::Type::Str(StrType)
    }
}

impl crate::ToType for str {
    fn to_type(&self) -> crate::Type {
        crate::Type::Str(StrType)
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    pub fn str() {
        let value = value_of!("test");

        assert!(value.is_str());
        assert_eq!(value.len(), 4);
        assert_eq!(value.to_string(), "test");
    }

    #[test]
    pub fn string() {
        let s = "test".to_string();
        let value = value_of!(s);

        assert!(value.is_str());
        assert_eq!(value.len(), 4);
        assert_eq!(value.to_string(), "test");
    }
}
