macro_rules! int {
    ($($name:ident $type_name:ident $is_type:ident $to_type:ident $set_value:ident $type:ty ,)*) => {
        #[derive(Debug, Copy, Clone, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub enum IntType {
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
                        Self::Int(v) => v.$is_type(),
                        _ => false,
                    };
                }

                pub fn $to_type(&self) -> $type_name {
                    return match self {
                        Self::Int(v) => v.$to_type(),
                        v => panic!("called '{}' on type '{}'", stringify!($to_type), v.to_type()),
                    };
                }
            )*
        }

        impl IntType {
            pub fn to_type(&self) -> crate::Type {
                return crate::Type::Number(crate::NumberType::Int(self.clone()));
            }

            pub fn id(&self) -> crate::TypeId {
                return match self {
                    $(Self::$name(v) => v.id(),)*
                };
            }

            pub fn is_signed(&self) -> bool {
                return match self {
                    $(Self::$name(v) => v.is_signed(),)*
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
                        v => panic!("called '{}' on type '{}'", stringify!($to_type), v),
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
        /// IntType: Implementations
        ///

        impl PartialEq<crate::Type> for IntType {
            fn eq(&self, other: &crate::Type) -> bool {
                return other.is_int() && other.as_number().as_int() == self;
            }
        }

        impl crate::ToType for IntType {
            fn to_type(&self) -> crate::Type {
                return crate::Type::Number(crate::NumberType::Int(self.clone()));
            }
        }

        impl std::fmt::Display for IntType {
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
                pub fn id(&self) -> crate::TypeId {
                    return crate::TypeId::from_str(stringify!($type));
                }

                pub fn is_signed(&self) -> bool {
                    return stringify!($name).starts_with("I");
                }

                pub fn assignable_to(&self, ty: crate::Type) -> bool {
                    return self.id() == ty.id();
                }

                pub fn convertable_to(&self, ty: crate::Type) -> bool {
                    return ty.is_int() && self.is_signed() == ty.is_signed();
                }
            }

            impl std::fmt::Display for $type_name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    return write!(f, "{}", self.id());
                }
            }

            impl crate::TypeOf for $type {
                fn type_of() -> crate::Type {
                    return crate::Type::Number(crate::NumberType::Int(crate::IntType::$name($type_name)));
                }
            }

            impl crate::ToType for $type {
                fn to_type(&self) -> crate::Type {
                    return crate::Type::Number(crate::NumberType::Int(IntType::$name($type_name)));
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
        let ty = type_of!(125_i8);

        assert!(ty.is_int());
        assert!(ty.is_signed());
        assert!(ty.is_i8());
    }

    #[test]
    pub fn i16() {
        let ty = type_of!(-15_i16);

        assert!(ty.is_int());
        assert!(ty.is_signed());
        assert!(ty.is_i16());
    }

    #[test]
    pub fn i32() {
        let ty = type_of!(-15_i32);

        assert!(ty.is_int());
        assert!(ty.is_signed());
        assert!(ty.is_i32());
    }

    #[test]
    pub fn i64() {
        let ty = type_of!(-15_i64);

        assert!(ty.is_int());
        assert!(ty.is_signed());
        assert!(ty.is_i64());
    }

    #[test]
    pub fn u8() {
        let ty = type_of!(15_u8);

        assert!(ty.is_int());
        assert!(!ty.is_signed());
        assert!(ty.is_u8());
    }

    #[test]
    pub fn u16() {
        let ty = type_of!(15_u16);

        assert!(ty.is_int());
        assert!(!ty.is_signed());
        assert!(ty.is_u16());
    }

    #[test]
    pub fn u32() {
        let ty = type_of!(15_u32);

        assert!(ty.is_int());
        assert!(!ty.is_signed());
        assert!(ty.is_u32());
    }

    #[test]
    pub fn u64() {
        let ty = type_of!(15_u64);

        assert!(ty.is_int());
        assert!(!ty.is_signed());
        assert!(ty.is_u64());
    }
}
