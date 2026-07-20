pub mod actors;
pub mod chats;
pub mod data;
pub mod events;
pub mod resources;
pub mod skills;
pub mod tasks;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Page<T> {
    pub index: usize,
    pub size: usize,
    pub items: Vec<T>,
}
