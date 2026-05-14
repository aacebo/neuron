#[macro_export]
macro_rules! type_of {
    [&$value:expr] => {
        $crate::ToType::to_type(&$value)
    };
    [$($type:ty)*] => {
        $(<$type>::type_of())*
    };
    [$value:expr] => {
        $crate::ToType::to_type(&$value)
    };
}

pub trait TypeOf {
    fn type_of() -> crate::Type;
}

pub trait ToType {
    fn to_type(&self) -> crate::Type;
}

#[cfg(test)]
mod test {
    use crate::TypeOf;

    #[test]
    pub fn from_expr() {
        assert!(type_of!(1_i8).is_i8());
    }

    #[test]
    pub fn from_expr_ref() {
        assert!(type_of!(&1_i8).is_i8());
    }

    #[test]
    pub fn from_type() {
        assert!(type_of!(i8).is_i8());
    }
}
