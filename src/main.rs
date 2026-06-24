use std::io::{self, Write};
use std::process;

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().expect("err");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("err");
        input = input.trim().to_string();
        if input == "exit" {
            process::exit(1);
        }

        println!("sandal: command not found: {input}");
    }
}
