use crate::game_engine::level::LevelState;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GamePlayer {
    pub name: String,
    #[serde(default)]
    pub class: PlayerClass,
    pub level: LevelState,
    pub attr: PlayerAttributes,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PlayerClass {
    #[default]
    Warrior,
    Mage,
    Rogue,
    Bard,
}

impl PlayerClass {
    pub const ALL: [PlayerClass; 4] = [
        PlayerClass::Warrior,
        PlayerClass::Mage,
        PlayerClass::Rogue,
        PlayerClass::Bard,
    ];

    pub fn name(self) -> &'static str {
        match self {
            PlayerClass::Warrior => "Warrior",
            PlayerClass::Mage => "Mage",
            PlayerClass::Rogue => "Rogue",
            PlayerClass::Bard => "Bard",
        }
    }

    pub fn primary(self) -> Attribute {
        match self {
            PlayerClass::Warrior => Attribute::Strength,
            PlayerClass::Mage => Attribute::Intelligence,
            PlayerClass::Rogue => Attribute::Agility,
            PlayerClass::Bard => Attribute::Collaboration,
        }
    }

    pub fn secondary(self) -> Attribute {
        match self {
            PlayerClass::Warrior => Attribute::Agility,
            PlayerClass::Mage => Attribute::Wisdom,
            PlayerClass::Rogue => Attribute::Intelligence,
            PlayerClass::Bard => Attribute::Wisdom,
        }
    }

    pub fn tagline(self) -> &'static str {
        match self {
            PlayerClass::Warrior => "feats of force",
            PlayerClass::Mage => "building things",
            PlayerClass::Rogue => "moving fast",
            PlayerClass::Bard => "working with others",
        }
    }

    pub fn affinity_bonus(self, attr: Attribute) -> u32 {
        if attr == self.primary() || attr == self.secondary() {
            1
        } else {
            0
        }
    }

    /// Bonus XP for a command that played to the class's strength: when it
    /// trains the primary attribute, you earn a quarter more XP — so you level
    /// up faster doing what your class is built for.
    pub fn xp_bonus(self, earned: u32, trained: &[(Attribute, u32)]) -> u32 {
        if trained.iter().any(|(attr, _)| *attr == self.primary()) {
            earned / 4
        } else {
            0
        }
    }

    pub fn from_choice(input: &str) -> Option<PlayerClass> {
        match input.trim().to_ascii_lowercase().as_str() {
            "1" | "warrior" => Some(PlayerClass::Warrior),
            "2" | "mage" => Some(PlayerClass::Mage),
            "3" | "rogue" => Some(PlayerClass::Rogue),
            "4" | "bard" => Some(PlayerClass::Bard),
            _ => None,
        }
    }
}

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
    pub fn new(name: impl Into<String>, class: PlayerClass) -> Self {
        Self {
            name: name.into(),
            class,
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

    #[test]
    fn each_class_owns_a_distinct_primary_attribute() {
        let primaries: Vec<_> = PlayerClass::ALL.iter().map(|c| c.primary()).collect();
        assert_eq!(
            primaries,
            vec![
                Attribute::Strength,
                Attribute::Intelligence,
                Attribute::Agility,
                Attribute::Collaboration,
            ]
        );
    }

    #[test]
    fn affinity_bonus_rewards_primary_and_secondary_only() {
        let mage = PlayerClass::Mage; // masters intelligence, hones wisdom
        assert_eq!(mage.affinity_bonus(Attribute::Intelligence), 1);
        assert_eq!(mage.affinity_bonus(Attribute::Wisdom), 1);
        assert_eq!(mage.affinity_bonus(Attribute::Strength), 0);
    }

    #[test]
    fn xp_bonus_applies_only_when_the_primary_attribute_is_trained() {
        let warrior = PlayerClass::Warrior; // masters strength
        // A strength command earns a quarter more XP.
        assert_eq!(warrior.xp_bonus(8, &[(Attribute::Strength, 1)]), 2);
        // Training only the secondary (agility) grants no XP bonus.
        assert_eq!(warrior.xp_bonus(8, &[(Attribute::Agility, 1)]), 0);
        // Nothing trained, nothing gained.
        assert_eq!(warrior.xp_bonus(8, &[]), 0);
    }

    #[test]
    fn class_is_chosen_by_number_or_name() {
        assert_eq!(PlayerClass::from_choice("1"), Some(PlayerClass::Warrior));
        assert_eq!(PlayerClass::from_choice("  rogue "), Some(PlayerClass::Rogue));
        assert_eq!(PlayerClass::from_choice("BARD"), Some(PlayerClass::Bard));
        assert_eq!(PlayerClass::from_choice("wizard"), None);
    }
}
