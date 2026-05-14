#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Type {
    Any,
    Bool(crate::BoolType),
    Enum(std::rc::Rc<crate::EnumType>),
    Number(crate::NumberType),
    Str(crate::StrType),
    Ref(crate::RefType),
    Slice(crate::SliceType),
    Map(std::rc::Rc<crate::MapType>),
    Struct(std::rc::Rc<crate::StructType>),
    This(crate::ThisType),
    Tuple(std::rc::Rc<crate::TupleType>),
    Trait(std::rc::Rc<crate::TraitType>),
    Mut(crate::MutType),
    Mod(std::rc::Rc<crate::ModType>),
    Void,
}

impl Type {
    pub fn id(&self) -> crate::TypeId {
        match self {
            Self::Any => crate::TypeId::from_str("any"),
            Self::Bool(v) => v.id(),
            Self::Enum(v) => v.id(),
            Self::Number(v) => v.id(),
            Self::Str(v) => v.id(),
            Self::Ref(v) => v.id(),
            Self::Slice(v) => v.id(),
            Self::Map(v) => v.id(),
            Self::Struct(v) => v.id(),
            Self::This(v) => v.id(),
            Self::Tuple(v) => v.id(),
            Self::Trait(v) => v.id(),
            Self::Mut(v) => v.id(),
            Self::Mod(v) => v.id(),
            Self::Void => crate::TypeId::from_str("void"),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Enum(v) => v.len(),
            Self::Slice(v) => v.len(),
            Self::Struct(v) => v.len(),
            Self::Tuple(v) => v.len(),
            Self::Trait(v) => v.len(),
            Self::Mod(v) => v.len(),
            _ => panic!("called 'len' on '{}'", self.id()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn path(&self) -> &crate::Path {
        match self {
            Self::Map(v) => v.path(),
            Self::Struct(v) => v.path(),
            Self::Enum(v) => v.path(),
            Self::Trait(v) => v.path(),
            Self::Mod(v) => v.path(),
            _ => panic!("called 'path' on '{}'", self.id()),
        }
    }

    pub fn meta(&self) -> &crate::MetaData {
        match self {
            Self::Map(v) => v.meta(),
            Self::Struct(v) => v.meta(),
            Self::Enum(v) => v.meta(),
            Self::Trait(v) => v.meta(),
            Self::Mod(v) => v.meta(),
            _ => panic!("called 'meta' on '{}'", self.id()),
        }
    }

    pub fn to_item(&self) -> crate::Item {
        crate::Item::Type(self.clone())
    }

    pub fn is_any(&self) -> bool {
        matches!(self, Self::Any)
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    pub fn is_enum(&self) -> bool {
        matches!(self, Self::Enum(_))
    }

    pub fn is_ref(&self) -> bool {
        matches!(self, Self::Ref(_))
    }

    pub fn is_ref_of(&self, ty: Self) -> bool {
        match self {
            Self::Ref(v) => v.is_ref_of(ty),
            _ => false,
        }
    }

    pub fn is_ref_self(&self) -> bool {
        match self {
            Self::Ref(v) => v.ty().is_self(),
            _ => false,
        }
    }

    pub fn is_ref_mut(&self) -> bool {
        match self {
            Self::Ref(v) => v.ty().is_mut(),
            _ => false,
        }
    }

    pub fn is_ref_mut_self(&self) -> bool {
        match self {
            Self::Ref(v) => v.ty().is_mut_self(),
            _ => false,
        }
    }

    pub fn is_slice(&self) -> bool {
        matches!(self, Self::Slice(_))
    }

    pub fn is_slice_of(&self, ty: Self) -> bool {
        match self {
            Self::Slice(v) => v.is_slice_of(ty),
            _ => false,
        }
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, Self::Struct(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub fn is_int(&self) -> bool {
        match self {
            Self::Number(v) => v.is_int(),
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            Self::Number(v) => v.is_float(),
            _ => false,
        }
    }

    pub fn is_signed(&self) -> bool {
        match self {
            Self::Number(v) => v.is_signed(),
            _ => false,
        }
    }

    pub fn is_str(&self) -> bool {
        matches!(self, Self::Str(_))
    }

    pub fn is_self(&self) -> bool {
        matches!(self, Self::This(_))
    }

    pub fn is_tuple(&self) -> bool {
        matches!(self, Self::Tuple(_))
    }

    pub fn is_trait(&self) -> bool {
        matches!(self, Self::Trait(_))
    }

    pub fn is_mut(&self) -> bool {
        matches!(self, Self::Mut(_))
    }

    pub fn is_mut_of(&self, ty: crate::Type) -> bool {
        match self {
            Self::Mut(v) => v.is_mut_of(ty),
            _ => false,
        }
    }

    pub fn is_mut_self(&self) -> bool {
        match self {
            Self::Mut(v) => v.ty().is_self(),
            _ => false,
        }
    }

    pub fn is_mod(&self) -> bool {
        matches!(self, Self::Mod(_))
    }

    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map(_))
    }

    pub fn is_void(&self) -> bool {
        matches!(self, Self::Void)
    }

    pub fn to_bool(&self) -> crate::BoolType {
        match self {
            Self::Bool(v) => *v,
            _ => panic!("called 'to_bool' on '{}'", self.id()),
        }
    }

    pub fn as_bool(&self) -> &crate::BoolType {
        match self {
            Self::Bool(v) => v,
            _ => panic!("called 'as_bool' on '{}'", self.id()),
        }
    }

    pub fn to_enum(&self) -> crate::EnumType {
        match self {
            Self::Enum(v) => v.as_ref().clone(),
            _ => panic!("called 'to_enum' on '{}'", self.id()),
        }
    }

    pub fn as_enum(&self) -> &crate::EnumType {
        match self {
            Self::Enum(v) => v.as_ref(),
            _ => panic!("called 'as_enum' on '{}'", self.id()),
        }
    }

    pub fn to_ref(&self) -> crate::RefType {
        match self {
            Self::Ref(v) => v.clone(),
            _ => panic!("called 'to_ref' on '{}'", self.id()),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn as_ref(&self) -> &crate::RefType {
        match self {
            Self::Ref(v) => v,
            _ => panic!("called 'as_ref' on '{}'", self.id()),
        }
    }

    pub fn to_slice(&self) -> crate::SliceType {
        match self {
            Self::Slice(v) => v.clone(),
            _ => panic!("called 'to_slice' on '{}'", self.id()),
        }
    }

    pub fn as_slice(&self) -> &crate::SliceType {
        match self {
            Self::Slice(v) => v,
            _ => panic!("called 'as_slice' on '{}'", self.id()),
        }
    }

    pub fn to_struct(&self) -> crate::StructType {
        match self {
            Self::Struct(v) => v.as_ref().clone(),
            _ => panic!("called 'to_struct' on '{}'", self.id()),
        }
    }

    pub fn as_struct(&self) -> &crate::StructType {
        match self {
            Self::Struct(v) => v.as_ref(),
            _ => panic!("called 'as_struct' on '{}'", self.id()),
        }
    }

    pub fn to_number(&self) -> crate::NumberType {
        match self {
            Self::Number(v) => *v,
            _ => panic!("called 'to_number' on '{}'", self.id()),
        }
    }

    pub fn as_number(&self) -> &crate::NumberType {
        match self {
            Self::Number(v) => v,
            _ => panic!("called 'as_number' on '{}'", self.id()),
        }
    }

    pub fn to_int(&self) -> crate::IntType {
        match self {
            Self::Number(v) => v.to_int(),
            _ => panic!("called 'to_int' on '{}'", self.id()),
        }
    }

    pub fn as_int(&self) -> &crate::IntType {
        match self {
            Self::Number(v) => v.as_int(),
            _ => panic!("called 'as_int' on '{}'", self.id()),
        }
    }

    pub fn to_float(&self) -> crate::FloatType {
        match self {
            Self::Number(v) => v.to_float(),
            _ => panic!("called 'to_float' on '{}'", self.id()),
        }
    }

    pub fn as_float(&self) -> &crate::FloatType {
        match self {
            Self::Number(v) => v.as_float(),
            _ => panic!("called 'as_float' on '{}'", self.id()),
        }
    }

    pub fn to_str(&self) -> crate::StrType {
        match self {
            Self::Str(v) => *v,
            _ => panic!("called 'to_str' on '{}'", self.id()),
        }
    }

    pub fn as_str(&self) -> &crate::StrType {
        match self {
            Self::Str(v) => v,
            _ => panic!("called 'as_str' on '{}'", self.id()),
        }
    }

    pub fn to_self(&self) -> crate::ThisType {
        match self {
            Self::This(v) => v.clone(),
            _ => panic!("called 'to_self' on '{}'", self.id()),
        }
    }

    pub fn as_self(&self) -> &crate::ThisType {
        match self {
            Self::This(v) => v,
            _ => panic!("called 'as_self' on '{}'", self.id()),
        }
    }

    pub fn to_tuple(&self) -> crate::TupleType {
        match self {
            Self::Tuple(v) => v.as_ref().clone(),
            _ => panic!("called 'to_tuple' on '{}'", self.id()),
        }
    }

    pub fn as_tuple(&self) -> &crate::TupleType {
        match self {
            Self::Tuple(v) => v.as_ref(),
            _ => panic!("called 'as_tuple' on '{}'", self.id()),
        }
    }

    pub fn to_trait(&self) -> crate::TraitType {
        match self {
            Self::Trait(v) => v.as_ref().clone(),
            _ => panic!("called 'to_trait' on '{}'", self.id()),
        }
    }

    pub fn as_trait(&self) -> &crate::TraitType {
        match self {
            Self::Trait(v) => v.as_ref(),
            _ => panic!("called 'as_trait' on '{}'", self.id()),
        }
    }

    pub fn to_mut(&self) -> crate::MutType {
        match self {
            Self::Mut(v) => v.clone(),
            _ => panic!("called 'to_mut' on '{}'", self.id()),
        }
    }

    pub fn as_mut(&self) -> &crate::MutType {
        match self {
            Self::Mut(v) => v,
            _ => panic!("called 'as_mut' on '{}'", self.id()),
        }
    }

    pub fn to_mod(&self) -> crate::ModType {
        match self {
            Self::Mod(v) => v.as_ref().clone(),
            _ => panic!("called 'to_mod' on '{}'", self.id()),
        }
    }

    pub fn as_mod(&self) -> &crate::ModType {
        match self {
            Self::Mod(v) => v.as_ref(),
            _ => panic!("called 'as_mod' on '{}'", self.id()),
        }
    }

    pub fn to_map(&self) -> crate::MapType {
        match self {
            Self::Map(v) => v.as_ref().clone(),
            _ => panic!("called 'to_map' on '{}'", self.id()),
        }
    }

    pub fn as_map(&self) -> &crate::MapType {
        match self {
            Self::Map(v) => v.as_ref(),
            _ => panic!("called 'as_map' on '{}'", self.id()),
        }
    }

    pub fn assignable_to(&self, ty: Self) -> bool {
        match self {
            Self::Bool(v) => v.assignable_to(ty),
            Self::Enum(v) => v.assignable_to(ty),
            Self::Number(v) => v.assignable_to(ty),
            Self::Str(v) => v.assignable_to(ty),
            Self::Ref(v) => v.assignable_to(ty),
            Self::Slice(v) => v.assignable_to(ty),
            Self::Struct(v) => v.assignable_to(ty),
            Self::This(v) => v.assignable_to(ty),
            Self::Tuple(v) => v.assignable_to(ty),
            Self::Trait(v) => v.assignable_to(ty),
            Self::Mut(v) => v.assignable_to(ty),
            Self::Mod(v) => v.assignable_to(ty),
            Self::Map(v) => v.assignable_to(ty),
            _ => false,
        }
    }

    pub fn convertable_to(&self, ty: Self) -> bool {
        match self {
            Self::Bool(v) => v.convertable_to(ty),
            Self::Enum(v) => v.convertable_to(ty),
            Self::Number(v) => v.convertable_to(ty),
            Self::Str(v) => v.convertable_to(ty),
            Self::Ref(v) => v.convertable_to(ty),
            Self::Slice(v) => v.convertable_to(ty),
            Self::Struct(v) => v.convertable_to(ty),
            Self::This(v) => v.convertable_to(ty),
            Self::Tuple(v) => v.convertable_to(ty),
            Self::Trait(v) => v.convertable_to(ty),
            Self::Mut(v) => v.convertable_to(ty),
            Self::Mod(v) => v.convertable_to(ty),
            Self::Map(v) => v.convertable_to(ty),
            _ => false,
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Any => write!(f, "any"),
            Self::Bool(v) => write!(f, "{}", v),
            Self::Enum(v) => write!(f, "{}", v),
            Self::Number(v) => write!(f, "{}", v),
            Self::Str(v) => write!(f, "{}", v),
            Self::Ref(v) => write!(f, "{}", v),
            Self::Slice(v) => write!(f, "{}", v),
            Self::Struct(v) => write!(f, "{}", v),
            Self::This(v) => write!(f, "{}", v),
            Self::Tuple(v) => write!(f, "{}", v),
            Self::Trait(v) => write!(f, "{}", v),
            Self::Mut(v) => write!(f, "{}", v),
            Self::Mod(v) => write!(f, "{}", v),
            Self::Map(v) => write!(f, "{}", v),
            Self::Void => write!(f, "void"),
        }
    }
}
