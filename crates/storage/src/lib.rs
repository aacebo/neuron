use sqlx::PgPool;

pub mod types;

mod annotation;
mod artifact;
mod job;
mod message;

pub use annotation::*;
pub use artifact::*;
pub use job::*;
pub use message::*;

pub struct Storage<'a> {
    _messages: MessageStorage<'a>,
    _annotations: AnnotationStorage<'a>,
    _artifacts: ArtifactStorage<'a>,
    _jobs: JobStorage<'a>,
}

impl<'a> Storage<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self {
            _messages: MessageStorage::new(pool),
            _annotations: AnnotationStorage::new(pool),
            _artifacts: ArtifactStorage::new(pool),
            _jobs: JobStorage::new(pool),
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

    pub fn jobs(&self) -> &JobStorage<'a> {
        &self._jobs
    }
}
