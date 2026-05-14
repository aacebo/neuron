#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ConstParam {
    pub(crate) name: String,
    pub(crate) ty: crate::Type,
    pub(crate) default: Option<crate::Value<'static>>,
}

impl ConstParam {
    pub fn new(name: &str, ty: &crate::Type) -> Self {
        Self {
            name: name.to_string(),
            ty: ty.clone(),
            default: None,
        }
    }

    pub fn with_default(self, default: &crate::Value<'static>) -> Self {
        let mut next = self.clone();
        next.default = Some(default.clone());
        next
    }

    pub fn to_generic(&self) -> crate::Generic {
        crate::Generic::Const(self.clone())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ty(&self) -> &crate::Type {
        &self.ty
    }

    pub fn default(&self) -> Option<&crate::Value<'static>> {
        match &self.default {
            None => None,
            Some(v) => Some(v),
        }
    }
}

impl std::fmt::Display for ConstParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "const {}: {}", &self.name, &self.ty)?;

        if let Some(default) = &self.default {
            write!(f, " = {}", default)?;
        }

        Ok(())
    }
}
