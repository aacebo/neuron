macro_rules! float {
    ($($name:ident $type_name:ident $is_type:ident $to_type:ident $set_value:ident $type:ty)*) => {
        #[derive(Debug, Copy, Clone, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub enum FloatType {
            $($name($type_name),)*
        }

        impl crate::Type {
            $(
                pub fn $is_type(&self) -> bool {
                    return match self {
                        Self::Number(v) => v.$is_type(),
                        _ => false,
                    };
                }

                pub fn $to_type(&self) -> $type_name {
                    return match self {
                        Self::Number(v) => v.$to_type(),
                        v => panic!("called '{}' on type '{}'", stringify!($to_type), v),
                    };
                }
            )*
        }

        impl crate::NumberType {
            $(
                pub fn $is_type(&self) -> bool {
                    return match self {
                        Self::Float(v) => v.$is_type(),
                        _ => false,
                    };
                }

                pub fn $to_type(&self) -> $type_name {
                    return match self {
                        Self::Float(v) => v.$to_type(),
                        v => panic!("called '{}' on type '{}'", stringify!($to_type), v.to_type()),
                    };
                }
            )*
        }

        impl FloatType {
            pub fn to_type(&self) -> crate::Type {
                return crate::Type::Number(crate::NumberType::Float(self.clone()));
            }

            pub fn id(&self) -> crate::TypeId {
                return match self {
                    $(Self::$name(v) => v.id(),)*
                };
            }

            $(
                pub fn $is_type(&self) -> bool {
                    return match self {
                        Self::$name(_) => true,
                        _ => false,
                    };
                }

                pub fn $to_type(&self) -> $type_name {
                    return match self {
                        Self::$name(v) => v.clone(),
                        v => panic!("called '{}' on type '{}'", stringify!($to_type), v.to_type()),
                    };
                }
            )*

            pub fn assignable_to(&self, ty: crate::Type) -> bool {
                return match self {
                    $(Self::$name(v) => v.assignable_to(ty),)*
                };
            }

            pub fn convertable_to(&self, ty: crate::Type) -> bool {
                return match self {
                    $(Self::$name(v) => v.convertable_to(ty),)*
                };
            }
        }

        ///
        /// FloatType: Implementations
        ///

        impl PartialEq<crate::Type> for FloatType {
            fn eq(&self, other: &crate::Type) -> bool {
                return other.is_float() && other.as_number().as_float() == self;
            }
        }

        impl crate::ToType for FloatType {
            fn to_type(&self) -> crate::Type {
                return crate::Type::Number(crate::NumberType::Float(self.clone()));
            }
        }

        impl std::fmt::Display for FloatType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                return match self {
                    $(Self::$name(v) => write!(f, "{}", v),)*
                };
            }
        }

        ///
        /// Type: Definitions
        ///

        $(
            #[derive(Debug, Copy, Clone, PartialEq, Default)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            pub struct $type_name;

            impl $type_name {
                pub fn to_type(&self) -> crate::Type {
                    return crate::Type::Number(crate::NumberType::Float(crate::FloatType::$name($type_name)));
                }

                pub fn id(&self) -> crate::TypeId {
                    return crate::TypeId::from_str(&stringify!($type));
                }

                pub fn assignable_to(&self, ty: crate::Type) -> bool {
                    return self.id() == ty.id();
                }

                pub fn convertable_to(&self, ty: crate::Type) -> bool {
                    return ty.is_float();
                }
            }

            impl std::fmt::Display for $type_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    return write!(f, "{}", self.id());
                }
            }

            impl crate::TypeOf for $type {
                fn type_of() -> crate::Type {
                    return crate::Type::Number(crate::NumberType::Float(crate::FloatType::$name($type_name)));
                }
            }

            impl crate::ToType for $type {
                fn to_type(&self) -> crate::Type {
                    return crate::Type::Number(crate::NumberType::Float(crate::FloatType::$name($type_name)));
                }
            }

            impl PartialEq<crate::Type> for $type_name {
                fn eq(&self, other: &crate::Type) -> bool {
                    return match other {
                        crate::Type::Number(v) => v.$is_type() && (v.$to_type() == *self),
                        _ => false,
                    };
                }
            }
        )*
    };
}

float! {
    F32 F32Type is_f32 to_f32 set_f32 f32
    F64 F64Type is_f64 to_f64 set_f64 f64
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    pub fn f32() {
        let value = value_of!(300.26_f32);

        assert!(value.is_float());
        assert!(value.is_f32());
        assert_eq!(value.to_f32(), 300.26);
    }

    #[test]
    pub fn f64() {
        let value = value_of!(350.26_f64);

        assert!(value.is_float());
        assert!(value.is_f64());
        assert_eq!(value.to_f64(), 350.26);
    }
}
