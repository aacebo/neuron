use crate::{Field, FieldName};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Fields {
    pub(crate) layout: crate::Layout,
    pub(crate) fields: Vec<Field>,
}

impl Fields {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> crate::FieldsBuilder {
        crate::FieldsBuilder::new()
    }

    pub fn layout(&self) -> &crate::Layout {
        &self.layout
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Field> {
        self.fields.iter()
    }

    pub fn has(&self, name: &FieldName) -> bool {
        self.fields.iter().any(|v| v.name() == name)
    }

    pub fn get(&self, name: &FieldName) -> Option<&Field> {
        self.fields.iter().find(|v| v.name() == name)
    }

    pub fn get_mut(&mut self, name: &FieldName) -> Option<&mut Field> {
        self.fields.iter_mut().find(|v| v.name() == name)
    }
}

impl From<&[crate::Field]> for Fields {
    fn from(value: &[crate::Field]) -> Self {
        Self::new().fields(value.iter().cloned()).build()
    }
}

impl<const N: usize> From<&[crate::Field; N]> for Fields {
    fn from(value: &[crate::Field; N]) -> Self {
        Self::new().fields(value.iter().cloned()).build()
    }
}

impl<const N: usize> From<[crate::Field; N]> for Fields {
    fn from(value: [crate::Field; N]) -> Self {
        Self::new().fields(value).build()
    }
}

impl AsRef<Fields> for Fields {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsMut<Fields> for Fields {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl std::ops::Index<usize> for Fields {
    type Output = Field;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(&FieldName::from(index)).unwrap()
    }
}

impl std::ops::IndexMut<usize> for Fields {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(&FieldName::from(index)).unwrap()
    }
}

impl std::ops::Index<&str> for Fields {
    type Output = Field;

    fn index(&self, index: &str) -> &Self::Output {
        self.get(&FieldName::from(index)).unwrap()
    }
}

impl std::ops::IndexMut<&str> for Fields {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        self.get_mut(&FieldName::from(index)).unwrap()
    }
}

impl std::fmt::Display for Fields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.layout.is_key() {
            write!(f, " {{")?;

            for field in &self.fields {
                write!(f, "\n\t{},", field)?;
            }

            if !self.fields.is_empty() {
                writeln!(f)?;
            }

            return write!(f, "}}");
        }

        if self.layout.is_index() {
            write!(f, "(")?;

            for (i, field) in self.fields.iter().enumerate() {
                if !field.vis.is_private() {
                    write!(f, "{} ", field.vis())?;
                }

                write!(f, "{}", field.ty())?;

                if i < self.fields.len() - 1 {
                    write!(f, ", ")?;
                }
            }

            return write!(f, ")");
        }

        Ok(())
    }
}

///
/// Builder
///
#[derive(Debug, Clone)]
pub struct FieldsBuilder(crate::Fields);

impl Default for FieldsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FieldsBuilder {
    pub fn new() -> Self {
        Self(crate::Fields {
            layout: crate::Layout::Unit,
            fields: vec![],
        })
    }

    pub fn layout(mut self, layout: crate::Layout) -> Self {
        self.0.layout = layout;
        self
    }

    pub fn fields(mut self, fields: impl IntoIterator<Item = crate::Field>) -> Self {
        self.0.fields.extend(fields);
        self
    }

    pub fn field(mut self, field: crate::Field) -> Self {
        self.0.fields.push(field);
        self
    }

    pub fn build(self) -> crate::Fields {
        self.0
    }
}
