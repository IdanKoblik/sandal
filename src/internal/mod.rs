mod cd;
mod exit;

pub enum InternalCommand<'a> {
    Cd(&'a str),
    Exit,
}

impl InternalCommand<'_> {
    pub fn execute(self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            InternalCommand::Cd(path) => cd::execute(path),
            InternalCommand::Exit => exit::execute(),
        }
    }
}
