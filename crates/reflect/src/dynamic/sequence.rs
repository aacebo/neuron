/// ## Sequence
///
/// implemented by types that can reflect their value/type
/// and the values of their individual elements
pub trait Sequence: std::fmt::Debug + crate::ToType {
    fn len(&self) -> usize;
    fn index(&self, i: usize) -> crate::Value<'_>;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for dyn Sequence {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        let ty = self.to_type().to_slice();
        let mut ser = serializer.serialize_seq(ty.capacity())?;

        for i in 0..self.len() {
            ser.serialize_element(&self.index(i))?;
        }

        ser.end()
    }
}

impl std::fmt::Display for dyn Sequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;

        for i in 0..self.len() {
            write!(f, "\n\t{}", self.index(i))?;
        }

        write!(f, "\n]")
    }
}

impl<T> crate::TypeOf for Vec<T>
where
    T: crate::TypeOf,
{
    fn type_of() -> crate::Type {
        crate::Type::Slice(crate::SliceType {
            ty: std::rc::Rc::new(T::type_of()),
            capacity: None,
        })
    }
}

impl<T> crate::ToType for Vec<T>
where
    T: crate::TypeOf,
{
    fn to_type(&self) -> crate::Type {
        <Vec<T> as crate::TypeOf>::type_of()
    }
}

impl<T> crate::ToValue for Vec<T>
where
    T: std::fmt::Debug + crate::TypeOf + crate::ToValue + 'static,
{
    fn to_value(&self) -> crate::Value<'_> {
        crate::Value::Dynamic(crate::Dynamic::from_sequence(self))
    }
}

impl<T> crate::Sequence for Vec<T>
where
    T: std::fmt::Debug + crate::TypeOf + crate::ToValue + 'static,
{
    fn len(&self) -> usize {
        self.len()
    }

    fn index(&self, i: usize) -> crate::Value<'_> {
        match self.get(i) {
            None => crate::Value::Null,
            Some(v) => v.to_value(),
        }
    }
}

impl<const N: usize, T> crate::Sequence for [T; N]
where
    T: std::fmt::Debug + crate::TypeOf + crate::ToValue + 'static,
{
    fn len(&self) -> usize {
        N
    }

    fn index(&self, i: usize) -> crate::Value<'_> {
        match self.get(i) {
            None => crate::Value::Null,
            Some(v) => v.to_value(),
        }
    }
}

impl<const N: usize, T> crate::ToValue for [T; N]
where
    T: std::fmt::Debug + crate::TypeOf + crate::ToValue + 'static,
{
    fn to_value(&self) -> crate::Value<'_> {
        crate::Value::Dynamic(crate::Dynamic::from_sequence(self))
    }
}

#[cfg(test)]
mod test {
    use crate::{Dynamic, ToValue};

    #[test]
    pub fn vec_sequence_index_returns_element() {
        let vec = vec![10_i32, 20, 30];
        let dynamic = Dynamic::from_sequence(&vec);

        assert_eq!(dynamic.len(), 3);
        assert_eq!(dynamic.as_sequence().index(1).to_i32(), 20);
    }

    #[test]
    pub fn vec_to_value_routes_through_dynamic_sequence() {
        let vec = vec![10_i32, 20, 30];
        let value = vec.to_value();

        assert!(value.is_dynamic());
        let seq = value.as_dynamic().as_sequence();
        assert_eq!(seq.len(), 3);
        assert_eq!(seq.index(2).to_i32(), 30);
    }

    #[test]
    pub fn array_to_value_routes_through_dynamic_sequence() {
        let arr: [i32; 3] = [1, 2, 3];
        let value = arr.to_value();

        assert!(value.is_dynamic());
        let seq = value.as_dynamic().as_sequence();
        assert_eq!(seq.len(), 3);
        assert_eq!(seq.index(0).to_i32(), 1);
    }
}
