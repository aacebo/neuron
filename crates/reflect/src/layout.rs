#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Layout {
    Unit,
    Key,
    Index,
}

impl Layout {
    pub fn is_unit(&self) -> bool {
        matches!(self, Self::Unit)
    }

    pub fn is_key(&self) -> bool {
        matches!(self, Self::Key)
    }

    pub fn is_index(&self) -> bool {
        matches!(self, Self::Index)
    }
}

impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unit => write!(f, "unit"),
            Self::Key => write!(f, "key"),
            Self::Index => write!(f, "index"),
        }
    }
}
