use crate::TypeOf;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BoolType;

impl BoolType {
    pub fn to_type(&self) -> crate::Type {
        bool::type_of()
    }

    pub fn id(&self) -> crate::TypeId {
        crate::TypeId::from_str("bool")
    }

    pub fn assignable_to(&self, ty: crate::Type) -> bool {
        self.id() == ty.id()
    }

    pub fn convertable_to(&self, ty: crate::Type) -> bool {
        ty.is_bool()
    }
}

impl std::fmt::Display for BoolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}

impl crate::ToType for BoolType {
    fn to_type(&self) -> crate::Type {
        crate::Type::Bool(*self)
    }
}

impl PartialEq<crate::Type> for BoolType {
    fn eq(&self, other: &crate::Type) -> bool {
        other.is_bool() && other.as_bool() == self
    }
}

impl crate::TypeOf for bool {
    fn type_of() -> crate::Type {
        crate::Type::Bool(BoolType)
    }
}

impl crate::ToType for bool {
    fn to_type(&self) -> crate::Type {
        Self::type_of()
    }
}
#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    pub fn truthy() {
        let value = value_of!(true);

        assert!(value.is_bool());
        assert!(value.is_true());
        assert!(value.to_bool());
    }

    #[test]
    pub fn falsy() {
        let value = value_of!(false);

        assert!(value.is_bool());
        assert!(value.is_false());
        assert!(!value.to_bool());
    }
}
