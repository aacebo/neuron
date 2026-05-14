mod float;
mod int;

pub use float::*;
pub use int::*;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Number {
    Int(Int),
    Float(Float),
}

impl Number {
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

    pub fn to_int(&self) -> Int {
        match self {
            Self::Int(v) => *v,
            v => panic!("called 'to_int' on '{}'", v.to_type()),
        }
    }

    pub fn as_int(&self) -> &Int {
        match self {
            Self::Int(v) => v,
            v => panic!("called 'as_int' on '{}'", v.to_type()),
        }
    }

    pub fn to_float(&self) -> Float {
        match self {
            Self::Float(v) => *v,
            v => panic!("called 'to_float' on '{}'", v.to_type()),
        }
    }

    pub fn as_float(&self) -> &Float {
        match self {
            Self::Float(v) => v,
            v => panic!("called 'as_float' on '{}'", v.to_type()),
        }
    }
}

impl crate::ToValue for Number {
    fn to_value(&self) -> crate::Value<'static> {
        crate::Value::Number(*self)
    }
}

impl PartialEq<crate::Value<'_>> for Number {
    fn eq(&self, other: &crate::Value<'_>) -> bool {
        other.is_number() && other.as_number() == self
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(v) => write!(f, "{}", v),
            Self::Float(v) => write!(f, "{}", v),
        }
    }
}
