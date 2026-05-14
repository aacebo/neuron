#[derive(Debug, Clone, PartialEq)]
pub struct Ref<'a> {
    pub(crate) ty: crate::RefType,
    pub(crate) value: &'a crate::Value<'a>,
}

impl<'a> Ref<'a> {
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
impl<'a> serde::Serialize for Ref<'a> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.value.serialize(s)
    }
}

impl<'a> crate::ToValue for Ref<'a> {
    fn to_value(&self) -> crate::Value<'_> {
        crate::Value::Ref(self.clone())
    }
}

impl<'a> AsRef<crate::Value<'a>> for Ref<'a> {
    fn as_ref(&self) -> &crate::Value<'a> {
        self.value
    }
}

impl<'a> std::ops::Deref for Ref<'a> {
    type Target = crate::Value<'a>;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<'a> std::fmt::Display for Ref<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<'a> PartialEq<crate::Value<'a>> for Ref<'a> {
    fn eq(&self, other: &crate::Value<'a>) -> bool {
        other.is_ref() && other.to_ref() == *self
    }
}
