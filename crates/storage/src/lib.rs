use sqlx::PgPool;

pub mod types;

mod message;

pub use message::*;

pub struct Storage<'a> {
    pub messages: MessageStorage<'a>,
}

impl<'a> Storage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            messages: MessageStorage::new(pool),
        }
    }
}
