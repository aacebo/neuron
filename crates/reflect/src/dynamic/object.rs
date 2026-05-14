/// ## Object
///
/// implemented by types that can reflect their value/type
/// and the values of their individual fields
pub trait Object: std::fmt::Debug + crate::ToType {
    fn field(&self, name: &crate::FieldName) -> crate::Value<'_>;
}

#[cfg(feature = "serde")]
impl serde::Serialize for dyn Object {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let ty = self.to_type().to_struct();
        let mut ser = serializer.serialize_map(Some(ty.len()))?;

        for field in ty.fields().iter() {
            ser.serialize_entry(&field.name().to_string(), &self.field(field.name()))?;
        }

        ser.end()
    }
}

impl std::fmt::Display for dyn Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ty = self.to_type().to_struct();
        write!(f, "{{")?;

        for field in ty.fields().iter() {
            write!(f, "\n\t{}: {}", field.name(), self.field(field.name()))?;
        }
        write!(f, "\n}}")
    }
}
