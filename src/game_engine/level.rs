use serde::{Deserialize, Serialize};

const B: f64 = 78.0;

#[derive(Serialize, Deserialize, Default)]
pub struct LevelState {
    pub level: u8,
    pub current_xp: u32,
    pub target_xp: u32,
}

// x_{p}\left(l_{evel}\right)=B\cdot l_{evel}^{1.5}
impl LevelState {
    pub fn new() -> Self {
        Self {
            level: 1,
            current_xp: 0,
            target_xp: Self::calc_target_xp(1),
        }
    }

    pub fn add_xp(&mut self, amount: u32) {
        self.current_xp += amount;

        while self.current_xp >= self.target_xp {
            self.current_xp -= self.target_xp;
            self.level += 1;
            self.target_xp = Self::calc_target_xp(self.level as u32);
        }
    }

    fn calc_target_xp(level: u32) -> u32 {
        (B * (level as f64).powf(1.5)).round() as u32
    }
}
