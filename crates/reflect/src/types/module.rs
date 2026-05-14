#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct ModType {
    pub(crate) path: crate::Path,
    pub(crate) meta: crate::MetaData,
    pub(crate) vis: crate::Visibility,
    pub(crate) items: Vec<crate::Item>,
}

impl ModType {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> crate::ModTypeBuilder {
        crate::ModTypeBuilder::new()
    }

    pub fn to_type(&self) -> crate::Type {
        crate::Type::Mod(std::rc::Rc::new(self.clone()))
    }

    pub fn id(&self) -> crate::TypeId {
        crate::TypeId::from_string(self.path.to_string())
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
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

    pub fn items(&self) -> &[crate::Item] {
        &self.items
    }

    pub fn assignable_to(&self, ty: crate::Type) -> bool {
        self.id() == ty.id()
    }

    pub fn convertable_to(&self, ty: crate::Type) -> bool {
        ty.is_mod()
    }
}

impl crate::ToType for ModType {
    fn to_type(&self) -> crate::Type {
        crate::Type::Mod(std::rc::Rc::new(self.clone()))
    }
}

impl std::ops::Index<usize> for ModType {
    type Output = crate::Item;

    fn index(&self, index: usize) -> &Self::Output {
        self.items.index(index)
    }
}

impl std::ops::IndexMut<usize> for ModType {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.items.index_mut(index)
    }
}

impl std::fmt::Display for ModType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.vis.is_private() {
            write!(f, "{} ", &self.vis)?;
        }

        write!(f, "{} {{", self.path.name())?;

        for item in &self.items {
            write!(f, "\n\t{}", item)?;
        }

        if !self.items.is_empty() {
            writeln!(f)?;
        }

        write!(f, "}}")
    }
}

///
/// Builder
///
#[derive(Debug, Clone)]
pub struct ModTypeBuilder(crate::ModType);

impl Default for ModTypeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ModTypeBuilder {
    pub fn new() -> Self {
        Self(crate::ModType {
            path: crate::Path::new(),
            meta: crate::MetaData::new(),
            vis: crate::Visibility::Private,
            items: vec![],
        })
    }

    pub fn path(mut self, path: crate::Path) -> Self {
        self.0.path = path;
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

    pub fn items(mut self, items: impl IntoIterator<Item = crate::Item>) -> Self {
        self.0.items.extend(items);
        self
    }

    pub fn build(self) -> crate::ModType {
        self.0
    }
}
