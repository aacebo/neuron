#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TraitBound {
    pub(crate) path: crate::Path,
    pub(crate) modifier: TraitBoundModifier,
}

impl TraitBound {
    pub fn new(path: &crate::Path, modifier: TraitBoundModifier) -> Self {
        Self {
            path: path.clone(),
            modifier,
        }
    }

    pub fn to_bound(&self) -> crate::Bound {
        crate::Bound::Trait(self.clone())
    }

    pub fn path(&self) -> &crate::Path {
        &self.path
    }

    pub fn modifier(&self) -> &TraitBoundModifier {
        &self.modifier
    }
}

impl std::fmt::Display for TraitBound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", &self.modifier, &self.path)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TraitBoundModifier {
    None,

    /// `?`
    Maybe,
}

impl TraitBoundModifier {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_maybe(&self) -> bool {
        matches!(self, Self::Maybe)
    }
}

impl std::fmt::Display for TraitBoundModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => Ok(()),
            Self::Maybe => write!(f, "?"),
        }
    }
}
