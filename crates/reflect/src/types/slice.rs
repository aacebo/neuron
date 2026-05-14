#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SliceType {
    pub(crate) ty: std::rc::Rc<crate::Type>,
    pub(crate) capacity: Option<usize>,
}

impl SliceType {
    pub fn to_type(&self) -> crate::Type {
        crate::Type::Slice(self.clone())
    }

    pub fn id(&self) -> crate::TypeId {
        match self.capacity {
            None => crate::TypeId::from_string(format!("[{}]", &self.ty.id())),
            Some(capacity) => {
                crate::TypeId::from_string(format!("[{}; {}]", &self.ty.id(), capacity))
            }
        }
    }

    pub fn len(&self) -> usize {
        match self.capacity {
            None => panic!("called 'len' on unbound slice type"),
            Some(capacity) => capacity,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.capacity == Some(0)
    }

    pub fn elem(&self) -> &crate::Type {
        &self.ty
    }

    pub fn capacity(&self) -> Option<usize> {
        self.capacity
    }

    pub fn is_slice_of(&self, ty: crate::Type) -> bool {
        ty.eq(&self.ty)
    }

    pub fn assignable_to(&self, ty: crate::Type) -> bool {
        self.id() == ty.id()
    }

    pub fn convertable_to(&self, ty: crate::Type) -> bool {
        ty.is_slice_of((*self.ty).clone())
    }
}

impl std::fmt::Display for SliceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id())
    }
}

impl crate::ToType for SliceType {
    fn to_type(&self) -> crate::Type {
        crate::Type::Slice(self.clone())
    }
}

impl<const N: usize, T> crate::TypeOf for [T; N]
where
    T: crate::TypeOf,
{
    fn type_of() -> crate::Type {
        let ty = T::type_of();

        crate::Type::Slice(SliceType {
            ty: std::rc::Rc::new(ty),
            capacity: Some(N),
        })
    }
}

impl<T> crate::TypeOf for [T]
where
    T: crate::TypeOf,
{
    fn type_of() -> crate::Type {
        let ty = T::type_of();

        crate::Type::Slice(SliceType {
            ty: std::rc::Rc::new(ty),
            capacity: None,
        })
    }
}

impl<const N: usize, T> crate::ToType for [T; N]
where
    T: crate::TypeOf,
{
    fn to_type(&self) -> crate::Type {
        let ty = T::type_of();

        crate::Type::Slice(SliceType {
            ty: std::rc::Rc::new(ty),
            capacity: Some(N),
        })
    }
}

impl<T> crate::ToType for [T]
where
    T: crate::TypeOf,
{
    fn to_type(&self) -> crate::Type {
        let ty = T::type_of();

        crate::Type::Slice(SliceType {
            ty: std::rc::Rc::new(ty),
            capacity: None,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    pub fn ok() {
        let value = value_of!([1, 2, 3]);

        assert!(value.is_dynamic());
        assert_eq!(value.to_type().len(), 3);
        assert_eq!(value.to_type().id(), "[i32; 3]");

        let seq = value.as_dynamic().as_sequence();
        assert_eq!(seq.len(), 3);

        for i in 0..seq.len() {
            let v = seq.index(i);
            assert!(v.is_i32());
            assert_eq!(i + 1, v.to_i32() as usize);
        }
    }
}
