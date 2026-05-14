mod r#const;
mod lifetime;
mod r#type;

pub use r#const::*;
pub use lifetime::*;
pub use r#type::*;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Generics(pub(crate) Vec<Generic>);

impl Default for Generics {
    fn default() -> Self {
        Self::new()
    }
}

impl Generics {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Generic> {
        self.0.iter()
    }

    pub fn add(&mut self, param: &Generic) {
        self.0.push(param.clone());
    }
}

impl<const N: usize> From<[Generic; N]> for Generics {
    fn from(value: [Generic; N]) -> Self {
        Self(value.to_vec())
    }
}

impl std::ops::Index<usize> for Generics {
    type Output = Generic;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl std::ops::IndexMut<usize> for Generics {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl std::fmt::Display for Generics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.0.is_empty() {
            write!(f, "<")?;

            for (i, param) in self.0.iter().enumerate() {
                write!(f, "{}", param)?;

                if i < self.0.len() - 1 {
                    write!(f, ", ")?;
                }
            }

            write!(f, ">")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Generic {
    Const(ConstParam),
    Lifetime(LifetimeParam),
    Type(TypeParam),
}

impl Generic {
    pub fn name(&self) -> &str {
        match self {
            Self::Const(_) => "const",
            Self::Lifetime(_) => "lifetime",
            Self::Type(_) => "type",
        }
    }

    pub fn is_const(&self) -> bool {
        matches!(self, Self::Const(_))
    }

    pub fn is_lifetime(&self) -> bool {
        matches!(self, Self::Lifetime(_))
    }

    pub fn is_type(&self) -> bool {
        matches!(self, Self::Type(_))
    }

    pub fn to_const(&self) -> ConstParam {
        match self {
            Self::Const(v) => v.clone(),
            _ => panic!("called 'to_const' on '{}'", self.name()),
        }
    }

    pub fn to_lifetime(&self) -> LifetimeParam {
        match self {
            Self::Lifetime(v) => v.clone(),
            _ => panic!("called 'to_lifetime' on '{}'", self.name()),
        }
    }

    pub fn to_type(&self) -> TypeParam {
        match self {
            Self::Type(v) => v.clone(),
            _ => panic!("called 'to_type' on '{}'", self.name()),
        }
    }
}

impl std::fmt::Display for Generic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Const(v) => write!(f, "{}", v),
            Self::Lifetime(v) => write!(f, "{}", v),
            Self::Type(v) => write!(f, "{}", v),
        }
    }
}
