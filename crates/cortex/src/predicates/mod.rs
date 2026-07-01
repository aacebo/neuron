mod equal;

pub use equal::*;

use crate::Context;

pub trait Predicate {
    fn name(&self) -> &'static str;
    fn invoke(&self, ctx: &Context) -> Result<bool, Box<dyn std::error::Error>>;
}
