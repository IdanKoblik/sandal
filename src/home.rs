/// Resolve a leading `~` (`~` or `~/...`) to the home directory.
pub fn expand_tilde(path: &str) -> String {
    match path.strip_prefix('~') {
        Some(rest) if rest.is_empty() || rest.starts_with('/') => match dirs::home_dir() {
            Some(home) => path.replace("~", &home.to_string_lossy()),
            None => path.to_string(),
        },
        _ => path.to_string(),
    }
}
