#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), serde(transparent))]
pub struct MutType(pub(crate) std::rc::Rc<crate::Type>);

impl MutType {
    pub fn new(ty: crate::Type) -> Self {
        Self(std::rc::Rc::new(ty))
    }

    pub fn to_type(&self) -> crate::Type {
        crate::Type::Mut(self.clone())
    }

    pub fn id(&self) -> crate::TypeId {
        crate::TypeId::from_string(format!("mut {}", &self.0.id()))
    }

    pub fn ty(&self) -> &crate::Type {
        &self.0
    }

    pub fn is_mut_of(&self, ty: crate::Type) -> bool {
        ty.eq(&self.0)
    }

    pub fn assignable_to(&self, ty: crate::Type) -> bool {
        self.id() == ty.id()
    }

    pub fn convertable_to(&self, ty: crate::Type) -> bool {
        ty.is_mut()
    }
}

impl crate::ToType for MutType {
    fn to_type(&self) -> crate::Type {
        crate::Type::Mut(self.clone())
    }
}

impl std::fmt::Display for MutType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mut {}", &self.0)
    }
}

impl<T> crate::TypeOf for &mut T
where
    T: crate::TypeOf,
{
    fn type_of() -> crate::Type {
        crate::MutType::new(T::type_of()).to_type()
    }
}

impl<T> crate::ToType for &mut T
where
    T: crate::TypeOf,
{
    fn to_type(&self) -> crate::Type {
        crate::MutType::new(T::type_of()).to_type()
    }
}

#[cfg(test)]
mod test {
    use crate::{TypeOf, type_of};

    #[test]
    pub fn basic() {
        let ty = type_of!(&mut i8);
        assert!(ty.is_mut());
    }
}
