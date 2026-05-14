#[derive(Debug, Clone, PartialEq)]
pub struct Mut<'a> {
    pub(crate) ty: crate::MutType,
    pub(crate) value: &'a crate::Value<'a>,
}

impl<'a> Mut<'a> {
    pub fn to_type(&self) -> crate::Type {
        self.ty.to_type()
    }

    pub fn ty(&self) -> &crate::Type {
        &self.ty.0
    }

    pub fn value(&self) -> &crate::Value<'a> {
        self.value
    }
}

#[cfg(feature = "serde")]
impl<'a> serde::Serialize for Mut<'a> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(s)
    }
}

impl<'a> std::fmt::Display for Mut<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl<'a> crate::ToType for Mut<'a> {
    fn to_type(&self) -> crate::Type {
        self.ty.to_type()
    }
}

impl<'a> crate::ToValue for Mut<'a> {
    fn to_value(&self) -> crate::Value<'_> {
        crate::Value::Mut(self.clone())
    }
}

impl<'a> AsRef<crate::Value<'a>> for Mut<'a> {
    fn as_ref(&self) -> &crate::Value<'a> {
        self.value
    }
}

impl<'a> std::ops::Deref for Mut<'a> {
    type Target = crate::Value<'a>;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}
