use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::game_engine::player::GamePlayer;

const STORE_DIR: &str = ".sandal";
const STORE_FILE: &str = "users.json";

#[derive(Default, Serialize, Deserialize)]
pub struct UserStore {
    users: HashMap<String, GamePlayer>,
}

impl UserStore {
    pub fn load() -> Self {
        let Some(path) = store_path() else {
            return Self::default();
        };
        match std::fs::read_to_string(&path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) {
        let Some(path) = store_path() else {
            return;
        };
        if let Some(dir) = path.parent() && let Err(err) = std::fs::create_dir_all(dir) {
                eprintln!("sandal: could not create {}: {err}", dir.display());
                return;
        }

        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(err) = std::fs::write(&path, json) {
                    eprintln!("sandal: failed to save users: {err}");
                }
            }
            Err(err) => eprintln!("sandal: failed to serialise users: {err}"),
        }
    }
}

pub fn login() -> GamePlayer {
    let mut store = UserStore::load();
    let id = current_identity();

    if let Some(player) = store.users.remove(&id) {
        println!("Welcome back, {} (level {}).", player.name, player.level.level);
        return player;
    }

    let name = prompt_name(&id);
    let player = GamePlayer::new(name);
    println!(
        "Welcome, {}! Your adventure begins at level {}.",
        player.name, player.level.level
    );
    player
}

pub fn save(player: GamePlayer) {
    let mut store = UserStore::load();
    store.users.insert(current_identity(), player);
    store.save();
}

fn store_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(STORE_DIR).join(STORE_FILE))
}

fn current_identity() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "player".to_string())
}

fn prompt_name(default: &str) -> String {
    println!("No saved character found — let's create one.");
    print!("Choose a name [{default}]: ");
    let _ = io::stdout().flush();

    let mut line = String::new();
    if io::stdin().read_line(&mut line).is_err() {
        return default.to_string();
    }

    let name = line.trim();
    if name.is_empty() {
        default.to_string()
    } else {
        name.to_string()
    }
}
