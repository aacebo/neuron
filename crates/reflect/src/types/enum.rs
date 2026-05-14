#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct EnumType {
    pub(crate) path: crate::Path,
    pub(crate) meta: crate::MetaData,
    pub(crate) vis: crate::Visibility,
    pub(crate) name: String,
    pub(crate) generics: crate::Generics,
    pub(crate) variants: Vec<crate::Variant>,
}

impl EnumType {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> EnumTypeBuilder {
        EnumTypeBuilder::new()
    }

    pub fn to_type(&self) -> crate::Type {
        crate::Type::Enum(std::rc::Rc::new(self.clone()))
    }

    pub fn id(&self) -> crate::TypeId {
        crate::TypeId::from_string(format!("{}::{}", &self.path, &self.name))
    }

    pub fn len(&self) -> usize {
        self.variants.len()
    }

    pub fn is_empty(&self) -> bool {
        self.variants.is_empty()
    }

    pub fn path(&self) -> &crate::Path {
        &self.path
    }

    pub fn meta(&self) -> &crate::MetaData {
        &self.meta
    }

    pub fn vis(&self) -> &crate::Visibility {
        &self.vis
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn generics(&self) -> &crate::Generics {
        &self.generics
    }

    pub fn assignable_to(&self, ty: crate::Type) -> bool {
        self.id() == ty.id()
    }

    pub fn convertable_to(&self, ty: crate::Type) -> bool {
        ty.is_enum()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, crate::Variant> {
        self.variants.iter()
    }

    pub fn has_variant(&self, name: &str) -> bool {
        self.variants.iter().any(|v| v.name() == name)
    }

    pub fn variant(&self, name: &str) -> &crate::Variant {
        self.variants.iter().find(|v| v.name() == name).unwrap()
    }

    pub fn variant_mut(&mut self, name: &str) -> &mut crate::Variant {
        self.variants.iter_mut().find(|v| v.name() == name).unwrap()
    }
}

impl crate::ToType for EnumType {
    fn to_type(&self) -> crate::Type {
        crate::Type::Enum(std::rc::Rc::new(self.clone()))
    }
}

impl std::fmt::Display for EnumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.vis != crate::Visibility::Private {
            write!(f, "{} ", &self.vis)?;
        }

        write!(f, "enum {}{} {{", &self.name, &self.generics)?;

        for variant in &self.variants {
            write!(f, "\n\t{},", variant)?;
        }

        if !self.variants.is_empty() {
            writeln!(f)?;
        }

        write!(f, "}}")
    }
}

///
/// Builder
///
#[derive(Debug, Clone)]
pub struct EnumTypeBuilder(crate::EnumType);

impl Default for EnumTypeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl EnumTypeBuilder {
    pub fn new() -> Self {
        Self(crate::EnumType {
            path: crate::Path::new(),
            meta: crate::MetaData::new(),
            vis: crate::Visibility::Private,
            name: String::new(),
            generics: crate::Generics::new(),
            variants: vec![],
        })
    }

    pub fn path(mut self, path: crate::Path) -> Self {
        self.0.path = path;
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.0.name = name.into();
        self
    }

    pub fn meta(mut self, meta: crate::MetaData) -> Self {
        self.0.meta = meta;
        self
    }

    pub fn visibility(mut self, vis: crate::Visibility) -> Self {
        self.0.vis = vis;
        self
    }

    pub fn generics(mut self, generics: crate::Generics) -> Self {
        self.0.generics = generics;
        self
    }

    pub fn variants(mut self, variants: impl IntoIterator<Item = crate::Variant>) -> Self {
        self.0.variants.extend(variants);
        self
    }

    pub fn variant(mut self, variant: crate::Variant) -> Self {
        self.0.variants.push(variant);
        self
    }

    pub fn build(self) -> crate::EnumType {
        self.0
    }
}
