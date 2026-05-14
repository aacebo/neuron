#![allow(unused)]

use reflect::{ToType, ToValue, TypeOf, value_of};
use reflect_macros::*;

#[reflect(version = 2)]
mod models {
    use reflect::{ToType, TypeOf};
    use reflect_macros::*;

    #[derive(Debug, Clone, Reflect)]
    pub enum Kind {
        #[reflect(lowercase)]
        Admin(String),
        Moderator,
        Basic,
    }

    impl std::fmt::Display for Kind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            return match self {
                Self::Admin(v) => write!(f, "{}", v),
                Self::Moderator => write!(f, "mod"),
                Self::Basic => write!(f, "basic"),
            };
        }
    }

    #[derive(Debug, Clone, Reflect)]
    #[reflect(name = "alex")]
    pub struct User {
        #[reflect(hello = "world")]
        pub kind: Kind,
        pub name: String,
        pub password: String,
    }

    #[derive(Debug, Clone, Reflect)]
    pub struct Position(pub f64, pub f64);

    #[derive(Debug, Clone, Reflect)]
    pub(crate) struct Restricted {
        pub x: i32,
    }
}

#[test]
pub fn should_reflect_struct() {
    let user = models::User {
        kind: models::Kind::Basic,
        name: String::from("dev"),
        password: String::from("test"),
    };

    assert!(user.to_type().is_struct());
    assert_eq!(user.to_type().len(), 3);
    assert!(user.to_type().to_struct().fields()["kind"].ty().is_enum());
    assert_eq!(user.to_type().meta().len(), 1);
    assert!(user.to_type().meta().has("name"));
    assert_eq!(
        user.to_type().to_struct().meta().get("name").unwrap(),
        &"alex".to_value()
    );
}

#[test]
pub fn should_reflect_field() {
    let user = models::User {
        kind: models::Kind::Basic,
        name: String::from("dev"),
        password: String::from("test"),
    };

    assert!(user.to_type().is_struct());

    let field = user.to_type().to_struct().fields()["kind"].clone();

    assert!(field.ty().is_enum());
    assert_eq!(field.meta().len(), 1);
    assert!(field.meta().has("hello"));
    assert_eq!(field.meta().get("hello").unwrap(), &"world".to_value());
    assert!(value_of!(user.clone()).is_dynamic());
}

#[test]
pub fn should_reflect_enum() {
    let kind = models::Kind::Admin("test".to_string());

    assert!(kind.to_type().is_enum());
    assert_eq!(kind.to_type().len(), 3);
    assert!(kind.to_type().to_enum().has_variant("Admin"));
    assert_eq!(kind.to_type().to_enum().variant("Admin").len(), 1);
    assert!(
        kind.to_type().to_enum().variant("Admin").fields()[0]
            .ty()
            .is_str()
    );
    assert!(
        kind.to_type()
            .to_enum()
            .variant("Admin")
            .meta()
            .has("lowercase")
    );
    assert_eq!(
        kind.to_type()
            .to_enum()
            .variant("Admin")
            .meta()
            .get("lowercase")
            .unwrap(),
        &reflect::Value::Null
    );
}

#[test]
pub fn should_reflect_tuple_struct() {
    let pos = models::Position(-500.1, 1034.45);

    assert!(pos.to_type().is_struct());
    assert_eq!(pos.to_type().len(), 2);
    assert!(pos.to_type().to_struct().fields()[0].ty().is_f64());
}

#[test]
pub fn should_reflect_restricted_visibility() {
    let r = models::Restricted { x: 7 };

    assert!(r.to_type().is_struct());
    assert_eq!(r.to_type().to_struct().name(), "Restricted");
    assert!(r.to_type().to_struct().vis().is_public());
}

#[test]
pub fn should_reflect_path() {
    let user = models::User {
        kind: models::Kind::Basic,
        name: String::from("dev"),
        password: String::from("test"),
    };

    assert_eq!(user.to_type().path().to_string(), "struct_tests::models");
}

#[test]
pub fn should_reflect_mod() {
    let ty = models::type_of();

    assert!(ty.is_mod());
    assert_eq!(ty.to_mod().path().name(), "models");
    assert!(ty.to_mod().vis().is_private());
    assert_eq!(ty.to_mod().items().len(), 5);

    let module = ty.to_mod();

    assert!(module.meta().has("version"));
    assert_eq!(module.meta().get("version").unwrap(), &value_of!(2));

    let one = &module.items()[0].to_type();

    assert!(one.is_enum());
    assert_eq!(one.to_enum().name(), "Kind");

    let two = &module.items()[1];

    assert!(two.is_impl());
    assert_eq!(two.to_impl().self_ty().to_enum().name(), "Kind");
    assert_eq!(
        two.to_impl().of_trait().unwrap().to_string(),
        "std::fmt::Display"
    );

    let three = &module.items()[2].to_type();

    assert!(three.is_struct());
    assert_eq!(three.to_struct().name(), "User");

    let four = &module.items()[3].to_type();

    assert!(four.is_struct());
    assert_eq!(four.to_struct().name(), "Position");
}
