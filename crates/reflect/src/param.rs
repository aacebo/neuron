#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Param {
    pub(crate) name: String,
    pub(crate) ty: std::rc::Rc<crate::Type>,
}

impl Param {
    pub fn new(name: &str, ty: crate::Type) -> Self {
        Self {
            name: name.to_string(),
            ty: std::rc::Rc::new(ty),
        }
    }

    pub fn is_selfish(&self) -> bool {
        self.name == "self"
            && (self.ty.is_self()
                || self.ty.is_mut_self()
                || self.ty.is_ref_self()
                || self.ty.is_ref_mut_self())
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ty(&self) -> &crate::Type {
        &self.ty
    }
}

impl std::fmt::Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_selfish() {
            let ty = self.ty.to_string();
            return write!(f, "{}self", &ty[0..ty.len() - 4]);
        }

        write!(f, "{}: {}", &self.name, &self.ty)
    }
}
