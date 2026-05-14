#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Impl {
    pub(crate) path: crate::Path,
    pub(crate) meta: crate::MetaData,
    pub(crate) of_trait: Option<crate::Path>,
    pub(crate) self_ty: crate::Type,
    pub(crate) generics: crate::Generics,
    pub(crate) methods: Vec<crate::Method>,
}

impl Impl {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> crate::ImplBuilder {
        crate::ImplBuilder::new()
    }

    pub fn to_item(&self) -> crate::Item {
        crate::Item::Impl(self.clone())
    }

    pub fn id(&self) -> crate::TypeId {
        let mut path = self.path.clone() + self.self_ty.path();

        if let Some(of) = &self.of_trait {
            path = path + of;
        }

        crate::TypeId::from_string(path.to_string())
    }

    pub fn len(&self) -> usize {
        self.methods.len()
    }

    pub fn is_empty(&self) -> bool {
        self.methods.is_empty()
    }

    pub fn meta(&self) -> &crate::MetaData {
        &self.meta
    }

    pub fn of_trait(&self) -> Option<&crate::Path> {
        match &self.of_trait {
            None => None,
            Some(v) => Some(v),
        }
    }

    pub fn self_ty(&self) -> &crate::Type {
        &self.self_ty
    }

    pub fn generics(&self) -> &crate::Generics {
        &self.generics
    }

    pub fn methods(&self) -> &[crate::Method] {
        &self.methods
    }

    pub fn has_method(&self, name: &str) -> bool {
        self.methods.iter().any(|v| v.name() == name)
    }

    pub fn method(&self, name: &str) -> &crate::Method {
        self.methods.iter().find(|v| v.name() == name).unwrap()
    }

    pub fn method_mut(&mut self, name: &str) -> &mut crate::Method {
        self.methods.iter_mut().find(|v| v.name() == name).unwrap()
    }
}

impl std::fmt::Display for Impl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "impl{}", &self.generics)?;

        if let Some(of) = &self.of_trait {
            write!(f, " {} for ", of.name())?;
        }

        write!(f, " {} {{", &self.self_ty)?;

        for method in &self.methods {
            write!(f, "\n\t{}", method)?;
        }

        if !self.methods.is_empty() {
            writeln!(f)?;
        }

        write!(f, "}}")
    }
}

///
/// Builder
///
#[derive(Debug, Clone)]
pub struct ImplBuilder(crate::Impl);

impl Default for ImplBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ImplBuilder {
    pub fn new() -> Self {
        Self(crate::Impl {
            path: crate::Path::new(),
            meta: crate::MetaData::new(),
            of_trait: None,
            self_ty: crate::Type::Any,
            generics: crate::Generics::new(),
            methods: vec![],
        })
    }

    pub fn path(mut self, path: crate::Path) -> Self {
        self.0.path = path;
        self
    }

    pub fn ty(mut self, ty: crate::Type) -> Self {
        self.0.self_ty = ty;
        self
    }

    pub fn meta(mut self, meta: crate::MetaData) -> Self {
        self.0.meta = meta;
        self
    }

    pub fn of(mut self, _trait: crate::Path) -> Self {
        self.0.of_trait = Some(_trait);
        self
    }

    pub fn generics(mut self, generics: crate::Generics) -> Self {
        self.0.generics = generics;
        self
    }

    pub fn methods(mut self, methods: impl IntoIterator<Item = crate::Method>) -> Self {
        self.0.methods.extend(methods);
        self
    }

    pub fn method(mut self, method: crate::Method) -> Self {
        self.0.methods.push(method);
        self
    }

    pub fn build(self) -> crate::Impl {
        self.0
    }
}
