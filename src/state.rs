const HISTORY_FILE: &str = ".sandal_history";

#[derive(Default)]
pub struct ShellState {
    pub history: String
}

fn history_path() -> String {
    let home = match dirs::home_dir() {
        Some(home) => home,
        None => return String::new(),
    };

    format!("{}/{HISTORY_FILE}", home.to_string_lossy())
}

fn load_history() -> String {
    let path = history_path();
    println!("path: {path}");
    std::fs::read_to_string(path).unwrap_or_default()
}

impl ShellState {
    pub fn new() -> Self {
        Self {
            history: load_history(),
        }
    }

    pub fn save_state(self) {
        let path = history_path();
        match std::fs::write(path, self.history) {
            Ok(_) => (),
            Err(err) => println!("failed to write into shell command history, {err}"),
        }
    }
}
