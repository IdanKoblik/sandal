use crate::state::ShellState;

mod cd;
mod history;

#[derive(PartialEq)]
pub enum InternalCommand<'a> {
    Cd(&'a str),
    History,
    Exit,
}

impl InternalCommand<'_> {
    pub fn execute(self, state: &mut ShellState) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            InternalCommand::Cd(path) => cd::execute(path),
            InternalCommand::Exit => Ok(()),
            InternalCommand::History => history::execute(state),
        }
    }
}
