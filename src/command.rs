use std::process::{Command as ProcessCommand};

pub enum Command {
    Internal(InternalCommand),
    External(ExternalCommand)
}

pub enum InternalCommand {
    Cd(String),
    Exit
}

pub struct ExternalCommand {
    program: String,
    args: Vec<String>
}

impl Command {
    pub fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            Command::Internal(cmd) => cmd.execute(),
            Command::External(cmd) => cmd.execute(),
        }
    }
}

impl InternalCommand {
    fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            InternalCommand::Cd(path) => {
                if path.is_empty() {
                    let home = dirs::home_dir().expect("Could not determine home directory");
                    std::env::set_current_dir(home)?;
                } else {
                    std::env::set_current_dir(path)?;
                }
            }
            InternalCommand::Exit => {
                std::process::exit(1);
            }
        };
        
        Ok(())
    }
}

impl ExternalCommand {
    fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        match ProcessCommand::new(&self.program).args(&self.args).spawn() {
            Ok(mut child) => {
                child.wait()?;
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Err(format!("command not found: {}", self.program).into());
            }
            Err(err) => return Err(err.into()),
        }

        Ok(())
    }
}


pub fn parse_command(input: &str) -> Command {
    let mut parts = input.split_whitespace();

    let name = parts.next().unwrap_or("");
    let args: Vec<String> = parts.map(String::from).collect();

    match name {
        "cd" => {
            Command::Internal(InternalCommand::Cd(args.first().cloned().unwrap_or_default()))
        }
        "exit" => Command::Internal(InternalCommand::Exit),
        _ => Command::External(
            ExternalCommand {
                program: name.to_string(),
                args,
            }
        )
    }
}
