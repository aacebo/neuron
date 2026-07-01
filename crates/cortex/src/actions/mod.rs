mod embed;
mod entities;
mod keywords;
mod pii;
mod sentiment;
mod summarization;

pub use embed::*;
pub use entities::*;
pub use keywords::*;
pub use pii::*;
pub use sentiment::*;
pub use summarization::*;

use crate::Context;

pub trait Action {
    fn name(&self) -> &'static str;
    fn invoke(&self, ctx: &mut Context) -> Result<(), Box<dyn std::error::Error>>;
}
