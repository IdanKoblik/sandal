use std::io::{self, Write};

pub mod command;

fn main() {
    let mut input = String::new();
    loop {
        print!("$ ");
        io::stdout().flush().expect("err");

        input.clear();
        io::stdin().read_line(&mut input).expect("err");
        input = input.trim().to_string();

        if input.is_empty() {
            continue;
        }

        let cmd = command::parse_command(input.as_str());
        if let Err(err) = cmd.execute() {
            println!("sandal: {err}");
        }
    }
}
