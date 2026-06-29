use crate::game_engine::level::LevelState;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GamePlayer {
    pub name: String,
    #[serde(skip)] // TODO
    pub class: Option<PlayerClass>,
    pub level: LevelState,
    pub attr: PlayerAttributes,
}

pub enum PlayerClass {}

#[derive(Serialize, Deserialize, Default)]
pub struct PlayerAttributes {
    pub strength: u32,
    pub intelligence: u32,
    pub agility: u32,
    pub wisdom: u32,
    pub collaboration: u32,
}

impl GamePlayer {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            class: None,
            level: LevelState::new(),
            attr: PlayerAttributes::default(),
        }
    }
}
