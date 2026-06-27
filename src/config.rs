use flash::interpreter::Interpreter;
use std::path::PathBuf;

const RC_FILE: &str = ".sandalrc";

pub fn expand_env(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '$' {
            out.push(c);
            continue;
        }

        let braced = chars.peek() == Some(&'{');
        if braced {
            chars.next();
        }

        let mut name = String::new();
        while let Some(&next) = chars.peek() {
            let valid = if braced {
                next != '}'
            } else {
                next.is_ascii_alphanumeric() || next == '_'
            };
            if !valid {
                break;
            }
            name.push(next);
            chars.next();
        }

        if braced && chars.peek() == Some(&'}') {
            chars.next();
        }

        if name.is_empty() {
            out.push('$');
            if braced {
                out.push_str("{}");
            }
            continue;
        }

        if let Ok(value) = std::env::var(&name) {
            out.push_str(&value);
        }
    }

    out
}

fn rc_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(RC_FILE))
}

pub fn source_rc() {
    let Some(path) = rc_path() else {
        return;
    };

    let script = match std::fs::read_to_string(&path) {
        Ok(script) => script,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return,
        Err(err) => {
            eprintln!("sandal: cannot read {}: {err}", path.display());
            return;
        }
    };

    let mut interpreter = Interpreter::new();
    if let Err(err) = interpreter.execute(&script) {
        eprintln!("sandal: {}: {err}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::expand_env;

    #[test]
    fn input_without_vars_is_unchanged() {
        assert_eq!(expand_env("echo hello"), "echo hello");
        assert_eq!(expand_env(""), "");
    }

    #[test]
    fn expands_plain_and_braced_vars() {
        unsafe { std::env::set_var("SANDAL_TEST_VAR", "world") };
        assert_eq!(expand_env("echo $SANDAL_TEST_VAR"), "echo world");
        assert_eq!(expand_env("echo ${SANDAL_TEST_VAR}!"), "echo world!");
    }

    #[test]
    fn unknown_var_expands_to_empty() {
        unsafe { std::env::remove_var("SANDAL_MISSING_VAR") };
        assert_eq!(expand_env("a${SANDAL_MISSING_VAR}b"), "ab");
    }

    #[test]
    fn lone_dollar_is_preserved() {
        assert_eq!(expand_env("cost is $ 5"), "cost is $ 5");
    }
}
