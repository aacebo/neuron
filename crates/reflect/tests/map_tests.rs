#![allow(unused)]

use std::collections::HashMap;

use reflect::{ToType, TypeOf, type_of, value_of};
use reflect_macros::*;

#[derive(Debug, Clone, Reflect)]
pub struct Tester(HashMap<u8, i8>);

#[test]
pub fn should_reflect_map() {
    let ty = type_of!(Tester).to_struct();
    let inner = &ty.fields()[0];

    assert_eq!(inner.ty(), &type_of!(HashMap<u8, i8>));
}
