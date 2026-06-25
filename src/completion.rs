use std::fs::DirEntry;
use std::os::unix::fs::PermissionsExt;

use trie_rs::{Trie, TrieBuilder};

use crate::home::expand_tilde;

const BUILTINS: [&str; 2] = ["cd", "exit"];

pub struct Completer {
    commands: Trie<u8>,
}

impl Default for Completer {
    fn default() -> Self {
        Self::new()
    }
}

impl Completer {
    pub fn new() -> Self {
        let mut builder = TrieBuilder::new();
        for builtin in BUILTINS {
            builder.push(builtin);
        }
        load_path_commands(&mut builder);

        Completer {
            commands: builder.build(),
        }
    }

    pub fn candidates(&self, word: &str, is_command: bool) -> Vec<String> {
        if is_command && !word.contains('/') {
            self.commands.predictive_search(word).collect()
        } else {
            path_candidates(word)
        }
    }
}

fn load_path_commands(builder: &mut TrieBuilder<u8>) {
    let Some(path) = std::env::var_os("PATH") else {
        return;
    };

    for dir in std::env::split_paths(&path) {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            if !is_executable(&entry) {
                continue;
            }
            if let Ok(name) = entry.file_name().into_string() {
                builder.push(name);
            }
        }
    }
}

fn is_executable(entry: &DirEntry) -> bool {
    std::fs::metadata(entry.path())
        .map(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn path_candidates(word: &str) -> Vec<String> {
    if word == "~" {
        return vec!["~/".to_string()];
    }

    let (dir, prefix) = match word.rfind('/') {
        Some(idx) => (&word[..=idx], &word[idx + 1..]),
        None => ("", word),
    };

    let read_from = expand_tilde(if dir.is_empty() { "." } else { dir });
    let Ok(entries) = std::fs::read_dir(&read_from) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for entry in entries.flatten() {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };

        // Hidden entries only surface when the prefix asks for them.
        if name.starts_with('.') && !prefix.starts_with('.') {
            continue;
        }

        if !name.starts_with(prefix) {
            continue;
        }

        let mut candidate = String::with_capacity(dir.len() + name.len() + 1);
        candidate.push_str(dir);
        candidate.push_str(&name);
        if entry.path().is_dir() {
            candidate.push('/');
        }
        out.push(candidate);
    }
    out
}

pub fn longest_common_prefix(words: &[String]) -> String {
    let Some(first) = words.first() else {
        return String::new();
    };

    let mut end = first.len();
    for word in &words[1..] {
        let common = first
            .bytes()
            .zip(word.bytes())
            .take_while(|(a, b)| a == b)
            .count();
        end = end.min(common);
    }

    while !first.is_char_boundary(end) {
        end -= 1;
    }
    first[..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::{Completer, longest_common_prefix};

    #[test]
    fn completes_builtins() {
        let completer = Completer::new();
        let exit = completer.candidates("exi", true);
        assert!(exit.contains(&"exit".to_string()), "got {exit:?}");
    }

    #[test]
    fn path_candidates_match_prefix() {
        let got = completer_candidates_for("src/comp");
        assert!(got.iter().any(|c| c == "src/completion.rs"), "got {got:?}");
    }

    #[test]
    fn bare_tilde_completes_to_home_root() {
        assert_eq!(completer_candidates_for("~"), vec!["~/".to_string()]);
    }

    #[test]
    fn tilde_candidates_keep_the_tilde_prefix() {
        // Every candidate must still start with the typed word, so the editor
        // can splice in the completed suffix. None should be expanded paths.
        for candidate in completer_candidates_for("~/") {
            assert!(candidate.starts_with("~/"), "leaked expansion: {candidate}");
        }
    }

    fn completer_candidates_for(word: &str) -> Vec<String> {
        Completer::new().candidates(word, false)
    }

    #[test]
    fn common_prefix_of_shared_stem() {
        let words = ["cargo".to_string(), "cargo-fmt".to_string()];
        assert_eq!(longest_common_prefix(&words), "cargo");
    }

    #[test]
    fn no_common_prefix() {
        let words = ["abc".to_string(), "xyz".to_string()];
        assert_eq!(longest_common_prefix(&words), "");
    }

    #[test]
    fn empty_input() {
        assert_eq!(longest_common_prefix(&[]), "");
    }
}
