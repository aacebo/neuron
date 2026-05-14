use crate::TypeOf;

impl<T> crate::TypeOf for Option<T> {
    fn type_of() -> crate::Type {
        crate::EnumType::new()
            .path(crate::Path::from("core::option"))
            .name("Option")
            .visibility(crate::Visibility::Public(crate::Public::Full))
            .generics(crate::Generics::from([crate::TypeParam::new()
                .name("T")
                .build()
                .to_generic()]))
            .build()
            .to_type()
    }
}

impl<T> crate::ToType for Option<T> {
    fn to_type(&self) -> crate::Type {
        Option::<T>::type_of()
    }
}

impl<T: crate::ToValue> crate::ToValue for Option<T> {
    fn to_value(&self) -> crate::Value<'_> {
        match self {
            None => crate::Value::Null,
            Some(v) => v.to_value(),
        }
    }
}
