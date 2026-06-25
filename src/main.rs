pub mod command;
pub mod completion;
pub mod editor;
pub mod home;

fn main() {
    let completer = completion::Completer::new();

    loop {
        match editor::read_line("$ ", &completer) {
            Ok(Some(line)) => {
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }

                let cmd = command::parse_command(input);
                if let Err(err) = cmd.execute() {
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
}
