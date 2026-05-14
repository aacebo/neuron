#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct TypeParam {
    pub(crate) name: String,
    pub(crate) default: Option<crate::Type>,
    pub(crate) bounds: Vec<crate::Bound>,
}

impl TypeParam {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> crate::TypeParamBuilder {
        crate::TypeParamBuilder::new()
    }

    pub fn to_generic(&self) -> crate::Generic {
        crate::Generic::Type(self.clone())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn bounds(&self) -> &[crate::Bound] {
        &self.bounds
    }

    pub fn default(&self) -> Option<&crate::Type> {
        match &self.default {
            None => None,
            Some(v) => Some(v),
        }
    }
}

impl std::fmt::Display for TypeParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)?;

        if !self.bounds.is_empty() {
            write!(f, ": ")?;
        }

        for (i, bound) in self.bounds.iter().enumerate() {
            write!(f, "{}", bound)?;

            if i < self.bounds.len() - 1 {
                write!(f, " + ")?;
            }
        }

        if let Some(default) = &self.default {
            write!(f, " = {}", default)?;
        }

        Ok(())
    }
}

///
/// Builder
///
#[derive(Debug, Clone)]
pub struct TypeParamBuilder(crate::TypeParam);

impl Default for TypeParamBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeParamBuilder {
    pub fn new() -> Self {
        Self(crate::TypeParam {
            name: String::from(""),
            default: None,
            bounds: vec![],
        })
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.0.name = name.into();
        self
    }

    pub fn default(mut self, default: crate::Type) -> Self {
        self.0.default = Some(default);
        self
    }

    pub fn bounds(mut self, bounds: impl IntoIterator<Item = crate::Bound>) -> Self {
        self.0.bounds.extend(bounds);
        self
    }

    pub fn bound(mut self, bound: crate::Bound) -> Self {
        self.0.bounds.push(bound);
        self
    }

    pub fn build(self) -> crate::TypeParam {
        self.0
    }
}
