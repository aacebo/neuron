#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct StructType {
    pub(crate) path: crate::Path,
    pub(crate) meta: crate::MetaData,
    pub(crate) vis: crate::Visibility,
    pub(crate) name: String,
    pub(crate) generics: crate::Generics,
    pub(crate) fields: crate::Fields,
}

impl StructType {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> crate::StructTypeBuilder {
        crate::StructTypeBuilder::new()
    }

    pub fn to_type(&self) -> crate::Type {
        crate::Type::Struct(std::rc::Rc::new(self.clone()))
    }

    pub fn id(&self) -> crate::TypeId {
        crate::TypeId::from_string(format!("{}::{}", &self.path, &self.name))
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
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

    pub fn fields(&self) -> &crate::Fields {
        &self.fields
    }

    pub fn assignable_to(&self, ty: crate::Type) -> bool {
        self.id() == ty.id()
    }

    pub fn convertable_to(&self, ty: crate::Type) -> bool {
        ty.is_struct()
    }
}

impl crate::ToType for StructType {
    fn to_type(&self) -> crate::Type {
        crate::Type::Struct(std::rc::Rc::new(self.clone()))
    }
}

impl std::fmt::Display for StructType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.vis != crate::Visibility::Private {
            write!(f, "{} ", &self.vis)?;
        }

        write!(f, "struct {}{}{}", &self.name, &self.generics, &self.fields)
    }
}

///
/// Builder
///
#[derive(Debug, Clone)]
pub struct StructTypeBuilder(crate::StructType);

impl Default for StructTypeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl StructTypeBuilder {
    pub fn new() -> Self {
        Self(crate::StructType {
            path: crate::Path::new(),
            meta: crate::MetaData::new(),
            vis: crate::Visibility::Private,
            name: String::from(""),
            generics: crate::Generics::new(),
            fields: crate::FieldsBuilder::new().build(),
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

    pub fn fields(mut self, fields: crate::Fields) -> Self {
        self.0.fields = fields;
        self
    }

    pub fn build(self) -> crate::StructType {
        self.0
    }
}
