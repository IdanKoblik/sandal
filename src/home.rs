/// Resolve a leading `~` (`~` or `~/...`) to the home directory.
pub fn expand_tilde(path: &str) -> String {
    match dirs::home_dir() {
        Some(home) => path.replace("~", &home.to_string_lossy()),
        None => path.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::expand_tilde;

    #[test]
    fn path_without_tilde_is_unchanged() {
        assert_eq!(expand_tilde("/usr/bin"), "/usr/bin");
        assert_eq!(expand_tilde(""), "");
    }

    #[test]
    fn leading_tilde_expands_to_home() {
        let Some(home) = dirs::home_dir() else {
            return;
        };
        let home = home.to_string_lossy();
        assert_eq!(expand_tilde("~"), home);
        assert_eq!(expand_tilde("~/projects"), format!("{home}/projects"));
    }
}
