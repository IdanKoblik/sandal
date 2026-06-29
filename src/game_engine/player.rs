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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attribute {
    Strength,
    Intelligence,
    Agility,
    Wisdom,
    Collaboration,
}

impl Attribute {
    pub fn name(self) -> &'static str {
        match self {
            Attribute::Strength => "strength",
            Attribute::Intelligence => "intelligence",
            Attribute::Agility => "agility",
            Attribute::Wisdom => "wisdom",
            Attribute::Collaboration => "collaboration",
        }
    }
}

impl PlayerAttributes {
    pub fn increment(&mut self, attr: Attribute, by: u32) {
        let field = match attr {
            Attribute::Strength => &mut self.strength,
            Attribute::Intelligence => &mut self.intelligence,
            Attribute::Agility => &mut self.agility,
            Attribute::Wisdom => &mut self.wisdom,
            Attribute::Collaboration => &mut self.collaboration,
        };
        *field = field.saturating_add(by);
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increment_adds_to_the_named_field_only() {
        let mut attr = PlayerAttributes::default();
        attr.increment(Attribute::Strength, 3);
        attr.increment(Attribute::Wisdom, 1);
        assert_eq!(attr.strength, 3);
        assert_eq!(attr.wisdom, 1);
        assert_eq!(attr.agility, 0);
    }

    #[test]
    fn increment_saturates_instead_of_overflowing() {
        let mut attr = PlayerAttributes {
            strength: u32::MAX,
            ..PlayerAttributes::default()
        };
        attr.increment(Attribute::Strength, 5);
        assert_eq!(attr.strength, u32::MAX);
    }
}
