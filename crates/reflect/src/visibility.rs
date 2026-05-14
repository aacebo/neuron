#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Default)]
pub enum Visibility {
    Public(Public),
    #[default]
    Private,
}

impl Visibility {
    pub fn is_public(&self) -> bool {
        matches!(self, Self::Public(_))
    }

    pub fn is_private(&self) -> bool {
        matches!(self, Self::Private)
    }
}

impl std::fmt::Display for Visibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Public(v) => write!(f, "{}", v),
            Self::Private => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Public {
    Full,
    Type,
    Super,
    Crate,
    Mod(String),
}

impl Public {
    pub fn is_full(&self) -> bool {
        matches!(self, Self::Full)
    }

    pub fn is_type(&self) -> bool {
        matches!(self, Self::Type)
    }

    pub fn is_super(&self) -> bool {
        matches!(self, Self::Super)
    }

    pub fn is_crate(&self) -> bool {
        matches!(self, Self::Crate)
    }

    pub fn is_mod(&self) -> bool {
        matches!(self, Self::Mod(_))
    }
}

impl std::fmt::Display for Public {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Full => write!(f, "pub"),
            Self::Type => write!(f, "pub(self)"),
            Self::Super => write!(f, "pub(super)"),
            Self::Crate => write!(f, "pub(crate)"),
            Self::Mod(path) => write!(f, "pub(in {})", path),
        }
    }
}
