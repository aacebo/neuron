macro_rules! int {
    ($($name:ident $type_name:ident $is_type:ident $to_type:ident $set_value:ident $type:ty ,)*) => {
        ///
        /// Value: Implementations
        ///

        impl crate::Value<'_> {
            pub fn is_int(&self) -> bool {
                return match self {
                    Self::Number(v) => v.is_int(),
                    _ => false,
                };
            }

            $(
                pub fn $is_type(&self) -> bool {
                    return match self {
                        Self::Number(v) => v.$is_type(),
                        _ => false,
                    };
                }

                pub fn $to_type(&self) -> $type {
                    return match self {
                        Self::Number(v) => v.$to_type(),
                        Self::Ref(v) => v.value().$to_type(),
                        Self::Mut(v) => v.value().$to_type(),
                        v => panic!("called '{}' on type '{}'", stringify!($to_type), v.to_type()),
                    };
                }
            )*
        }

        $(
            impl crate::ToValue for $type {
                fn to_value(&self) -> crate::Value<'static> {
                    return crate::Value::Number(crate::Number::Int(crate::Int::$name(*self)));
                }
            }

            impl From<$type> for crate::Value<'static> {
                fn from(value: $type) -> Self {
                    return Self::Number(crate::Number::Int(crate::Int::$name(value)));
                }
            }

            impl From<crate::Value<'_>> for $type {
                fn from(value: crate::Value<'_>) -> Self {
                    return match value {
                        crate::Value::Number(v) => v.to_int().$to_type(),
                        v => panic!("called 'From<Value>::from' on type '{}'", v.to_type()),
                    };
                }
            }

            impl AsRef<$type> for crate::Value<'_> {
                fn as_ref(&self) -> &$type {
                    return match self {
                        Self::Number(v) => AsRef::<$type>::as_ref(v),
                        v => panic!("called 'AsRef<{}>::as_ref' on type '{}'", stringify!($type), v.to_type()),
                    };
                }
            }

            impl AsMut<$type> for crate::Value<'_> {
                fn as_mut(&mut self) -> &mut $type {
                    return match self {
                        Self::Number(v) => AsMut::<$type>::as_mut(v),
                        v => panic!("called 'AsMut<{}>::as_mut' on type '{}'", stringify!($type), v.to_type()),
                    };
                }
            }
        )*

        ///
        /// Number: Implementations
        ///

        impl crate::Number {
            $(
                pub fn $is_type(&self) -> bool {
                    return match self {
                        Self::Int(v) => v.$is_type(),
                        _ => false,
                    };
                }

                pub fn $to_type(&self) -> $type {
                    return match self {
                        Self::Int(v) => v.$to_type(),
                        v => panic!("called '{}' on type '{}'", stringify!($to_type), v.to_type()),
                    };
                }
            )*
        }

        $(
            impl From<$type> for crate::Number {
                fn from(value: $type) -> Self {
                    return Self::Int(Int::$name(value));
                }
            }

            impl From<crate::Number> for $type {
                fn from(value: crate::Number) -> Self {
                    return match value {
                        crate::Number::Int(v) => v.$to_type(),
                        v => panic!("called 'From<Number>::from' on '{}'", v.to_type()),
                    };
                }
            }

            impl AsRef<$type> for crate::Number {
                fn as_ref(&self) -> &$type {
                    return match self {
                        Self::Int(v) => match v {
                            Int::$name(v) => v,
                            v => panic!("called 'AsRef<{}>::as_ref' on '{}'", stringify!($type), v.to_type()),
                        },
                        v => panic!("called 'AsRef<{}>::as_ref' on '{}'", stringify!($type), v.to_type()),
                    };
                }
            }

            impl AsMut<$type> for crate::Number {
                fn as_mut(&mut self) -> &mut $type {
                    return match self {
                        Self::Int(v) => match v {
                            Int::$name(v) => v,
                            v => panic!("called 'AsMut<{}>::as_mut' on '{}'", stringify!($type), v.to_type()),
                        },
                        v => panic!("called 'AsMut<{}>::as_mut' on '{}'", stringify!($type), v.to_type()),
                    };
                }
            }
        )*

        ///
        /// Int: Value
        ///
        #[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub enum Int {
            $($name($type),)*
        }

        impl Int {
            pub fn to_type(&self) -> crate::Type {
                return match self {
                    $(Self::$name(_) => crate::Type::Number(crate::NumberType::Int(crate::IntType::$name(crate::$type_name))),)*
                };
            }

            $(
                pub fn $is_type(&self) -> bool {
                    return match self {
                        Self::$name(_) => true,
                        _ => false,
                    };
                }

                pub fn $to_type(&self) -> $type {
                    return match self {
                        Self::$name(v) => *v,
                        _ => panic!("called '{}' on '{}'", stringify!($to_type), stringify!($type)),
                    };
                }

                pub fn $set_value(&mut self, value: $type) {
                    *self = Self::$name(value);
                }
            )*
        }

        impl crate::ToValue for crate::Int {
            fn to_value(&self) -> crate::Value<'static> {
                return crate::Value::Number(crate::Number::Int(*self));
            }
        }

        $(
            impl From<$type> for crate::Int {
                fn from(value: $type) -> Self {
                    return Self::$name(value);
                }
            }

            impl From<crate::Int> for $type {
                fn from(value: crate::Int) -> Self {
                    return value.$to_type();
                }
            }

           impl AsRef<$type> for crate::Int {
                fn as_ref(&self) -> &$type {
                    return match self {
                        Self::$name(v) => v,
                        v => panic!("called 'AsRef<{}>::as_ref' on '{}'", stringify!($type), v.to_type()),
                    };
                }
            }

            impl AsMut<$type> for crate::Int {
                fn as_mut(&mut self) -> &mut $type {
                    return match self {
                        Self::$name(v) => v,
                        v => panic!("called 'AsMut<{}>::as_mut' on '{}'", stringify!($type), v.to_type()),
                    };
                }
            }
        )*

        impl PartialEq<crate::Value<'_>> for crate::Int {
            fn eq(&self, other: &crate::Value) -> bool {
                return other.is_int() && other.as_number().as_int() == self;
            }
        }

        impl std::fmt::Display for crate::Int {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                return match self {
                    $(Self::$name(v) => write!(f, "{}", v),)*
                };
            }
        }
    };
}

int! {
    I8 I8Type is_i8 to_i8 set_i8 i8,
    I16 I16Type is_i16 to_i16 set_i16 i16,
    I32 I32Type is_i32 to_i32 set_i32 i32,
    I64 I64Type is_i64 to_i64 set_i64 i64,

    U8 U8Type is_u8 to_u8 set_u8 u8,
    U16 U16Type is_u16 to_u16 set_u16 u16,
    U32 U32Type is_u32 to_u32 set_u32 u32,
    U64 U64Type is_u64 to_u64 set_u64 u64,
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    pub fn i8() {
        let value = value_of!(125_i8);

        assert!(value.is_int());
        assert!(value.to_type().is_signed());
        assert!(value.is_i8());
        assert_eq!(value.to_i8(), 125);
    }

    #[test]
    pub fn i16() {
        let value = value_of!(-15_i16);

        assert!(value.is_int());
        assert!(value.to_type().is_signed());
        assert!(value.is_i16());
        assert_eq!(value.to_i16(), -15);
    }

    #[test]
    pub fn i32() {
        let value = value_of!(-15_i32);

        assert!(value.is_int());
        assert!(value.to_type().is_signed());
        assert!(value.is_i32());
        assert_eq!(value.to_i32(), -15);
    }

    #[test]
    pub fn i64() {
        let value = value_of!(-15_i64);

        assert!(value.is_int());
        assert!(value.to_type().is_signed());
        assert!(value.is_i64());
        assert_eq!(value.to_i64(), -15);
    }

    #[test]
    pub fn u8() {
        let value = value_of!(15_u8);

        assert!(value.is_int());
        assert!(!value.to_type().is_signed());
        assert!(value.is_u8());
        assert_eq!(value.to_u8(), 15);
    }

    #[test]
    pub fn u16() {
        let value = value_of!(15_u16);

        assert!(value.is_int());
        assert!(!value.to_type().is_signed());
        assert!(value.is_u16());
        assert_eq!(value.to_u16(), 15);
    }

    #[test]
    pub fn u32() {
        let value = value_of!(15_u32);

        assert!(value.is_int());
        assert!(!value.to_type().is_signed());
        assert!(value.is_u32());
        assert_eq!(value.to_u32(), 15);
    }

    #[test]
    pub fn u64() {
        let value = value_of!(15_u64);

        assert!(value.is_int());
        assert!(!value.to_type().is_signed());
        assert!(value.is_u64());
        assert_eq!(value.to_u64(), 15);
    }
}
