#[macro_export]
macro_rules! value_of {
    [$value:expr] => {
        $crate::ToValue::to_value(&$value)
    };
    ($($anything:tt)*) => {
        $crate::value_of!($($anything)*)
    };
}

/// ## ToValue
///
/// implemented by types that can reflect their value.
/// The lifetime `'a` ties the returned [`Value`] to the borrow of `self`,
/// allowing string/slice data to be borrowed rather than cloned.
pub trait ToValue {
    fn to_value(&self) -> crate::Value<'_>;
}

#[cfg(feature = "serde")]
impl serde::Serialize for dyn ToValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_value().serialize(serializer)
    }
}

#[cfg(test)]
mod test {
    #[test]
    pub fn from_expr() {
        let value = value_of!(1_i8);
        assert!(value.is_i8());
        assert_eq!(value.to_i8(), 1);
    }
}
