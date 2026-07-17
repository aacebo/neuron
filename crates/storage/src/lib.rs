use sqlx::PgPool;

pub mod types;

mod annotation;
mod artifact;
mod log;
mod message;
mod task;

pub use annotation::*;
pub use artifact::*;
pub use log::*;
pub use message::*;
pub use task::*;

pub struct Storage<'a> {
    _messages: MessageStorage<'a>,
    _annotations: AnnotationStorage<'a>,
    _artifacts: ArtifactStorage<'a>,
    _tasks: TaskStorage<'a>,
    _logs: LogStorage<'a>,
}

impl<'a> Storage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            _messages: MessageStorage::new(pool),
            _annotations: AnnotationStorage::new(pool),
            _artifacts: ArtifactStorage::new(pool),
            _tasks: TaskStorage::new(pool),
            _logs: LogStorage::new(pool),
        }
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

    pub fn logs(&self) -> &LogStorage<'a> {
        &self._logs
    }
}
