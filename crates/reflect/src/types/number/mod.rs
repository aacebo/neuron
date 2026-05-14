mod float;
mod int;

pub use float::*;
pub use int::*;

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NumberType {
    Int(IntType),
    Float(FloatType),
}

impl NumberType {
    pub fn id(&self) -> crate::TypeId {
        match self {
            Self::Int(v) => v.id(),
            Self::Float(v) => v.id(),
        }
    }

    pub fn to_type(&self) -> crate::Type {
        match self {
            Self::Int(v) => v.to_type(),
            Self::Float(v) => v.to_type(),
        }
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    pub fn is_signed(&self) -> bool {
        match self {
            Self::Int(v) => v.is_signed(),
            Self::Float(_) => true,
        }
    }

    pub fn to_int(&self) -> IntType {
        match self {
            Self::Int(v) => *v,
            _ => panic!("called 'to_int' on type '{}'", self.id()),
        }
    }

    pub fn as_int(&self) -> &IntType {
        match self {
            Self::Int(v) => v,
            _ => panic!("called 'as_int' on type '{}'", self.id()),
        }
    }

    pub fn to_float(&self) -> FloatType {
        match self {
            Self::Float(v) => *v,
            _ => panic!("called 'to_float' on type '{}'", self.id()),
        }
    }

    pub fn as_float(&self) -> &FloatType {
        match self {
            Self::Float(v) => v,
            _ => panic!("called 'as_float' on type '{}'", self.id()),
        }
    }

    pub fn assignable_to(&self, ty: crate::Type) -> bool {
        match self {
            Self::Int(v) => v.assignable_to(ty),
            Self::Float(v) => v.assignable_to(ty),
        }
    }

    pub fn convertable_to(&self, ty: crate::Type) -> bool {
        match self {
            Self::Int(v) => v.convertable_to(ty),
            Self::Float(v) => v.convertable_to(ty),
        }
    }
}

impl PartialEq<crate::Type> for NumberType {
    fn eq(&self, other: &crate::Type) -> bool {
        match other {
            crate::Type::Number(v) => v == self,
            _ => false,
        }
    }
}

impl std::fmt::Display for NumberType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{}", v),
            Self::Float(v) => write!(f, "{}", v),
        }
    }
}

impl crate::ToType for NumberType {
    fn to_type(&self) -> crate::Type {
        match self {
            Self::Int(v) => v.to_type(),
            Self::Float(v) => v.to_type(),
        }
    }
}
