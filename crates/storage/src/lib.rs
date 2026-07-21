use sqlx::PgPool;

mod actor;
mod annotation;
mod artifact;
mod chat;
mod error;
mod event;
mod message;
mod project;
mod task;

pub use actor::*;
pub use annotation::*;
pub use artifact::*;
pub use chat::*;
pub use error::*;
pub use event::*;
pub use message::*;
pub use task::*;

pub struct Storage<'a> {
    _actors: ActorStorage<'a>,
    _chats: ChatStorage<'a>,
    _messages: MessageStorage<'a>,
    _annotations: AnnotationStorage<'a>,
    _artifacts: ArtifactStorage<'a>,
    _tasks: TaskStorage<'a>,
    _events: EventStorage<'a>,
}

impl<'a> Storage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            _actors: ActorStorage::new(pool),
            _chats: ChatStorage::new(pool),
            _messages: MessageStorage::new(pool),
            _annotations: AnnotationStorage::new(pool),
            _artifacts: ArtifactStorage::new(pool),
            _tasks: TaskStorage::new(pool),
            _events: EventStorage::new(pool),
        }
    }

    pub fn actors(&self) -> &ActorStorage<'a> {
        &self._actors
    }

    pub fn chats(&self) -> &ChatStorage<'a> {
        &self._chats
    }

    pub fn messages(&self) -> &MessageStorage<'a> {
        &self._messages
    }

    pub fn annotations(&self) -> &AnnotationStorage<'a> {
        &self._annotations
    }

    pub fn artifacts(&self) -> &ArtifactStorage<'a> {
        &self._artifacts
    }

    pub fn tasks(&self) -> &TaskStorage<'a> {
        &self._tasks
    }

    pub fn events(&self) -> &EventStorage<'a> {
        &self._events
    }
}
