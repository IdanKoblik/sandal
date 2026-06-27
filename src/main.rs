use crate::command::CommandKind;
use crate::home::expand_tilde;
use crate::internal::InternalCommand;

pub mod command;
pub mod completion;
pub mod editor;
pub mod home;
pub mod internal;
pub mod state;

fn main() {
    let mut shell_state = state::ShellState::new();
    let completer = completion::Completer::new();

    loop {
        match editor::read_line("$ ", &completer) {
            Ok(Some(line)) => {
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }
                let expanded = expand_tilde(input);
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
