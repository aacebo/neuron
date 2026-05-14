#[derive(Debug, Clone, PartialEq)]
pub struct Str<'a>(pub(crate) &'a str);

impl<'a> Str<'a> {
    pub fn to_type(&self) -> crate::Type {
        crate::Type::Str(crate::StrType)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> AsRef<str> for Str<'a> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl<'a> std::ops::Deref for Str<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> std::fmt::Display for Str<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "serde")]
impl<'a> serde::Serialize for Str<'a> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

impl<'a> PartialEq<std::string::String> for Str<'a> {
    fn eq(&self, other: &std::string::String) -> bool {
        self.0 == other.as_str()
    }
}

impl crate::ToValue for std::string::String {
    fn to_value(&self) -> crate::Value<'_> {
        crate::Value::Str(Str(self.as_str()))
    }
}

impl crate::ToValue for &'static str {
    fn to_value(&self) -> crate::Value<'static> {
        crate::Value::Str(Str(self))
    }
}

impl crate::ToValue for str {
    fn to_value(&self) -> crate::Value<'_> {
        crate::Value::Str(Str(self))
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    pub fn str() {
        let value = value_of!("test");

        assert!(value.is_str());
        assert_eq!(value.len(), 4);
        assert_eq!(value.to_string(), "test");
    }

    #[test]
    pub fn string() {
        let s = "test".to_string();
        let value = value_of!(s);

        assert!(value.is_str());
        assert_eq!(value.len(), 4);
        assert_eq!(value.to_string(), "test");
    }
}
