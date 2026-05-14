impl<'a> crate::ToType for &'a [crate::Value<'a>] {
    fn to_type(&self) -> crate::Type {
        let elem = self
            .first()
            .map(crate::ToType::to_type)
            .unwrap_or(crate::Type::Any);

        crate::Type::Slice(crate::SliceType {
            ty: std::rc::Rc::new(elem),
            capacity: None,
        })
    }
}

impl<'a> crate::ToValue for &'a [crate::Value<'a>] {
    fn to_value(&self) -> crate::Value<'_> {
        crate::Value::Dynamic(crate::Dynamic::from_sequence(self))
    }
}

impl<'a> crate::Sequence for &'a [crate::Value<'a>] {
    fn len(&self) -> usize {
        <[crate::Value<'a>]>::len(self)
    }

    fn index(&self, i: usize) -> crate::Value<'_> {
        match self.get(i) {
            None => crate::Value::Null,
            Some(v) => v.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    pub fn value_slice_to_value() {
        let values: [Value<'_>; 3] = [value_of!(1_i32), value_of!(2_i32), value_of!(3_i32)];
        let slice: &[Value<'_>] = &values;
        let value = slice.to_value();

        assert!(value.is_dynamic());
        assert_eq!(value.len(), 3);

        let seq = value.as_dynamic().as_sequence();
        for i in 0..seq.len() {
            let v = seq.index(i);
            assert!(v.is_i32());
            assert_eq!(i + 1, v.to_i32() as usize);
        }
    }
}
