use crate::command::CommandKind;
use crate::home::expand_tilde;
use crate::internal::InternalCommand;

pub mod command;
pub mod completion;
pub mod config;
pub mod editor;
pub mod game_engine;
pub mod home;
pub mod internal;
pub mod prompt;
pub mod state;

fn main() {
    let cfg = config::source_rc();

    let player = game_engine::user::login();

    let mut shell_state = state::ShellState::new(player);
    if let Some(data) = cfg {
        shell_state.aliases = data.aliases;
    }

    let completer = completion::Completer::new();
    let format = std::env::var("PS1").unwrap_or_else(|_| prompt::DEFAULT_FORMAT.to_string());
    loop {
        let prompt = prompt::render(&format);
        let history: Vec<String> = shell_state.history.lines().map(str::to_string).collect();
        match editor::read_line(&prompt, &completer, &history) {
            Ok(Some(line)) => {
                let mut input = line.trim();
                if input.is_empty() {
                    continue;
                }

                if let Some(alias) = shell_state.aliases.get(input) {
                    input = alias.trim();
                }

                let expanded = expand_tilde(&config::expand_env(input));
                let cmd = command::parse_command(&expanded);
                if let CommandKind::Internal(InternalCommand::Exit) = cmd.kind {
                    break;
                }

                if let Err(err) = cmd.execute(&mut shell_state) {
                    println!("sandal: {err}");
                }
            }
            // Ctrl-D / end of input.
            Ok(None) => break,
            Err(err) => {
                eprintln!("sandal: {err}");
                break;
            }
        }
    }

    shell_state.save_state();
}
