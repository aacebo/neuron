#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LifetimeParam {
    pub(crate) name: String,
    pub(crate) bounds: Vec<crate::LifetimeBound>,
}

impl LifetimeParam {
    pub fn new(name: &str, bounds: &[crate::LifetimeBound]) -> Self {
        Self {
            name: name.to_string(),
            bounds: bounds.to_vec(),
        }
    }

    pub fn to_generic(&self) -> crate::Generic {
        crate::Generic::Lifetime(self.clone())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn bounds(&self) -> &[crate::LifetimeBound] {
        &self.bounds
    }
}

impl std::fmt::Display for LifetimeParam {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}", &self.name)?;

        if !self.bounds.is_empty() {
            write!(f, ": ")?;

            for (i, bound) in self.bounds.iter().enumerate() {
                write!(f, "{}", bound)?;

                if i < self.bounds.len() - 1 {
                    write!(f, " + ")?;
                }
            }
        }

        Ok(())
    }
}
