use crate::state::ShellState;

pub fn execute(state: &mut ShellState) -> Result<(), Box<dyn std::error::Error>> {
    for (i, val) in state.history.lines().enumerate() {
        println!("{i}: {val}");
    }

    Ok(())
}
