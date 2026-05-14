mod consumer;
mod error;
mod event;
mod key;
mod producer;
mod socket;

pub use consumer::*;
pub use error::*;
pub use event::*;
pub use key::*;
pub use producer::*;
pub use socket::*;

pub fn new(uri: &str) -> SocketOptions {
    SocketOptions::new(uri)
}
