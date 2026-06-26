pub fn execute(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if path.is_empty() {
        let home = dirs::home_dir().expect("Could not determine home directory");
        std::env::set_current_dir(home)?;
    } else {
        std::env::set_current_dir(path)?;
    }

    Ok(())
}
