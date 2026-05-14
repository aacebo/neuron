#![allow(unused)]

use reflect::{ToType, TypeOf, type_of, value_of};
use reflect_macros::*;

#[reflect(a = "b")]
trait Hello<T = String> {
    fn world(&self, a: u8) -> bool;
}

#[test]
pub fn should_reflect_trait() {
    let ty = type_of!(dyn Hello);

    assert!(ty.is_trait());
    assert_eq!(ty.len(), 1);
    assert!(ty.to_trait().has("world"));
    assert_eq!(ty.to_trait().get("world").unwrap().params().len(), 2);
    assert!(ty.to_trait().get("world").unwrap().has_param("self"));
    assert!(ty.to_trait().get("world").unwrap().has_param("a"));
    assert!(ty.to_trait().meta().has("a"));
    assert_eq!(ty.to_trait().meta().get("a").unwrap(), &value_of!("b"));
    assert_eq!(ty.to_trait().generics().len(), 1);
    assert_eq!(ty.to_trait().generics()[0].to_type().name(), "T");
    assert_eq!(
        ty.to_trait().generics()[0].to_type().default().unwrap(),
        &type_of!(String)
    );
}

#[reflect]
trait Borrowing {
    fn shared(&self, x: &i32) -> bool;
    fn exclusive(&mut self, y: &mut i32);
}

#[test]
pub fn should_reflect_reference_params() {
    let ty = type_of!(dyn Borrowing).to_trait();

    let shared_x = ty.get("shared").unwrap().param("x");
    assert!(shared_x.ty().is_ref());
    assert!(shared_x.ty().is_ref_of(type_of!(i32)));

    let exclusive_y = ty.get("exclusive").unwrap().param("y");
    assert!(exclusive_y.ty().is_ref());
    assert!(exclusive_y.ty().is_ref_mut());
}
