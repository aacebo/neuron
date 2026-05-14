use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(transparent))]
pub struct RefType(pub(crate) std::rc::Rc<crate::Type>);

impl RefType {
    pub fn new(ty: crate::Type) -> Self {
        Self(std::rc::Rc::new(ty))
    }

    pub fn to_type(&self) -> crate::Type {
        crate::Type::Ref(self.clone())
    }

    pub fn id(&self) -> crate::TypeId {
        crate::TypeId::from_string(format!("&{}", self.0.id()))
    }

    pub fn ty(&self) -> &crate::Type {
        &self.0
    }

    pub fn is_ref_of(&self, ty: crate::Type) -> bool {
        ty.eq(&self.0)
    }

    pub fn assignable_to(&self, ty: crate::Type) -> bool {
        self.id() == ty.id()
    }

    pub fn convertable_to(&self, ty: crate::Type) -> bool {
        ty.is_ref_of((*self.0).clone())
    }
}

impl std::fmt::Display for RefType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "&{}", &self.0)
    }
}

impl crate::ToType for RefType {
    fn to_type(&self) -> crate::Type {
        crate::Type::Ref(self.clone())
    }
}

impl PartialEq<crate::Type> for RefType {
    fn eq(&self, other: &crate::Type) -> bool {
        match other {
            crate::Type::Ref(v) => v == self,
            _ => false,
        }
    }
}

impl<T> crate::TypeOf for &T
where
    T: crate::TypeOf,
{
    fn type_of() -> crate::Type {
        crate::RefType::new(T::type_of()).to_type()
    }
}

impl<T> crate::ToType for &T
where
    T: crate::TypeOf,
{
    fn to_type(&self) -> crate::Type {
        crate::RefType::new(T::type_of()).to_type()
    }
}

impl<T> crate::ToType for Arc<T>
where
    T: Clone + crate::ToType,
{
    fn to_type(&self) -> crate::Type {
        crate::RefType::new(self.as_ref().to_type()).to_type()
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    pub fn int() {
        let ty = RefType::new(type_of!(i32));
        assert_eq!(ty.id(), ty.id());
        assert!(ty.to_type().is_ref());
    }

    #[test]
    pub fn bool() {
        let ty = RefType::new(type_of!(bool));
        assert!(ty.to_type().is_ref());
    }
}
