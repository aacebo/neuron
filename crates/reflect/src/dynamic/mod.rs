mod r#dyn;
mod object;
mod sequence;

pub use r#dyn::*;
pub use object::*;
pub use sequence::*;

#[derive(Debug, Clone)]
pub enum Dynamic<'a> {
    Dyn(&'a (dyn crate::Dyn + 'a)),
    Object(&'a (dyn crate::Object + 'a)),
    Sequence(&'a (dyn crate::Sequence + 'a)),
}

impl<'a> Dynamic<'a> {
    pub fn from_dyn<T: crate::Dyn + 'a>(value: &'a T) -> Self {
        Self::Dyn(value)
    }

    pub fn from_object<T: crate::Object + 'a>(value: &'a T) -> Self {
        Self::Object(value)
    }

    pub fn from_sequence<T: crate::Sequence + 'a>(value: &'a T) -> Self {
        Self::Sequence(value)
    }

    pub fn to_type(&self) -> crate::Type {
        match self {
            Self::Dyn(v) => v.to_type(),
            Self::Object(v) => v.to_type(),
            Self::Sequence(v) => v.to_type(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Sequence(v) => v.len(),
            v => panic!("called 'len' on '{}'", v.to_type()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    pub fn is_sequence(&self) -> bool {
        matches!(self, Self::Sequence(_))
    }

    pub fn as_object(&self) -> &(dyn crate::Object + 'a) {
        match self {
            Self::Object(v) => *v,
            v => panic!("called 'as_object' on '{}'", v.to_type()),
        }
    }

    pub fn as_sequence(&self) -> &(dyn crate::Sequence + 'a) {
        match self {
            Self::Sequence(v) => *v,
            v => panic!("called 'as_sequence' on '{}'", v.to_type()),
        }
    }
}

impl<'a> crate::TypeOf for Dynamic<'a> {
    fn type_of() -> crate::Type {
        crate::Type::Any
    }
}

impl<'a> crate::ToType for Dynamic<'a> {
    fn to_type(&self) -> crate::Type {
        match self {
            Self::Dyn(v) => v.to_type(),
            Self::Object(v) => v.to_type(),
            Self::Sequence(v) => v.to_type(),
        }
    }
}

impl<'a> crate::ToValue for Dynamic<'a> {
    fn to_value(&self) -> crate::Value<'_> {
        crate::Value::Dynamic(self.clone())
    }
}

impl<'a> crate::Object for Dynamic<'a> {
    fn field(&self, name: &crate::FieldName) -> crate::Value<'_> {
        match self {
            Self::Object(v) => v.field(name),
            v => panic!("called 'field' on '{}'", v.to_type()),
        }
    }
}

impl<'a> crate::Sequence for Dynamic<'a> {
    fn len(&self) -> usize {
        match self {
            Self::Sequence(v) => v.len(),
            v => panic!("called 'len' on '{}'", v.to_type()),
        }
    }

    fn index(&self, i: usize) -> crate::Value<'_> {
        match self {
            Self::Sequence(v) => v.index(i),
            v => panic!("called 'index' on '{}'", v.to_type()),
        }
    }
}

impl<'a> std::fmt::Display for Dynamic<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ty = self.to_type();
        write!(f, "<{}>", ty)
    }
}

#[cfg(feature = "serde")]
impl<'a> serde::Serialize for Dynamic<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        match self {
            Self::Dyn(_) => serializer.serialize_none(),
            Self::Object(v) => {
                let ty = v.to_type().to_struct();
                let mut ser = serializer.serialize_map(Some(ty.len()))?;

                for field in ty.fields().iter() {
                    ser.serialize_entry(&field.name().to_string(), &v.field(field.name()))?;
                }
                ser.end()
            }
            Self::Sequence(v) => {
                use serde::ser::SerializeSeq;
                let ty = v.to_type().to_slice();
                let mut ser = serializer.serialize_seq(ty.capacity())?;

                for i in 0..v.len() {
                    ser.serialize_element(&v.index(i))?;
                }
                ser.end()
            }
        }
    }
}
