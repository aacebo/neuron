#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Field {
    pub(crate) meta: crate::MetaData,
    pub(crate) vis: crate::Visibility,
    pub(crate) name: FieldName,
    pub(crate) ty: std::rc::Rc<crate::Type>,
}

impl Field {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> crate::FieldBuilder {
        crate::FieldBuilder::new()
    }

    pub fn meta(&self) -> &crate::MetaData {
        &self.meta
    }

    pub fn vis(&self) -> &crate::Visibility {
        &self.vis
    }

    pub fn name(&self) -> &FieldName {
        &self.name
    }

    pub fn ty(&self) -> &crate::Type {
        &self.ty
    }
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.vis.is_private() {
            write!(f, "{} ", &self.vis)?;
        }

        write!(f, "{}: {},", &self.name, &self.ty)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum FieldName {
    Key(String),
    Index(usize),
}

impl FieldName {
    pub fn is_key(&self) -> bool {
        matches!(self, Self::Key(_))
    }

    pub fn is_index(&self) -> bool {
        matches!(self, Self::Index(_))
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Key(v) => v,
            _ => panic!("called 'as_str' on 'FieldName::Index'"),
        }
    }

    pub fn as_index(&self) -> &usize {
        match self {
            Self::Index(v) => v,
            _ => panic!("called 'as_index' on 'FieldName::Key'"),
        }
    }
}

impl AsRef<Self> for FieldName {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<Self> for FieldName {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl From<&FieldName> for FieldName {
    fn from(value: &FieldName) -> Self {
        value.clone()
    }
}

impl From<&str> for FieldName {
    fn from(value: &str) -> Self {
        Self::Key(value.to_string())
    }
}

impl From<String> for FieldName {
    fn from(value: String) -> Self {
        Self::Key(value)
    }
}

impl From<&usize> for FieldName {
    fn from(value: &usize) -> Self {
        Self::Index(*value)
    }
}

impl From<usize> for FieldName {
    fn from(value: usize) -> Self {
        Self::Index(value)
    }
}

impl PartialEq<str> for FieldName {
    fn eq(&self, other: &str) -> bool {
        match self {
            Self::Key(v) => v == other,
            Self::Index(_) => false,
        }
    }
}

impl PartialEq<String> for FieldName {
    fn eq(&self, other: &String) -> bool {
        match self {
            Self::Key(v) => v == other,
            Self::Index(_) => false,
        }
    }
}

impl PartialEq<usize> for FieldName {
    fn eq(&self, other: &usize) -> bool {
        self.as_index() == other
    }
}

impl std::fmt::Display for FieldName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Key(v) => write!(f, "{}", v),
            Self::Index(v) => write!(f, "{}", v),
        }
    }
}

///
/// Builder
///
#[derive(Debug, Clone)]
pub struct FieldBuilder(crate::Field);

impl Default for FieldBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldBuilder {
    pub fn new() -> Self {
        Self(crate::Field {
            meta: crate::MetaData::new(),
            vis: crate::Visibility::Private,
            name: crate::FieldName::from(""),
            ty: std::rc::Rc::new(crate::Type::Any),
        })
    }

    pub fn name(mut self, name: crate::FieldName) -> Self {
        self.0.name = name;
        self
    }

    pub fn ty(mut self, ty: crate::Type) -> Self {
        self.0.ty = std::rc::Rc::new(ty);
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

    pub fn build(self) -> crate::Field {
        self.0
    }
}
