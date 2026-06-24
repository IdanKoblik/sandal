use std::io::{self, Write};

pub mod command;

fn main() {
    let mut input = String::new();
    loop {
        print!("$ ");
        io::stdout().flush().expect("err");

        input.clear();
        io::stdin().read_line(&mut input).expect("err");
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        let cmd = command::parse_command(input);
        if let Err(err) = cmd.execute() {
            println!("sandal: {err}");
        }
    }
}
