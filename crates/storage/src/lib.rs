use sqlx::PgPool;

mod actor;
mod annotation;
mod artifact;
mod chat;
mod event;
mod message;
mod projection;
mod skill;
mod task;

pub use actor::*;
pub use annotation::*;
pub use artifact::*;
pub use chat::*;
pub use event::*;
pub use message::*;
pub use skill::*;
pub use task::*;

pub struct Storage<'a> {
    _actors: ActorStorage<'a>,
    _chats: ChatStorage<'a>,
    _messages: MessageStorage<'a>,
    _annotations: AnnotationStorage<'a>,
    _artifacts: ArtifactStorage<'a>,
    _tasks: TaskStorage<'a>,
    _events: EventStorage<'a>,
    _skills: SkillStorage<'a>,
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
            _skills: SkillStorage::new(pool),
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

    pub fn skills(&self) -> &SkillStorage<'a> {
        &self._skills
    }
}
