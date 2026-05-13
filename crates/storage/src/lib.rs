use sqlx::PgPool;

pub mod types;

mod message;

pub use message::*;

pub struct Storage<'a> {
    _messages: MessageStorage<'a>,
}

impl<'a> Storage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            _messages: MessageStorage::new(pool),
        }
    }

    pub fn messages(&self) -> &MessageStorage<'a> {
        &self._messages
    }
}
