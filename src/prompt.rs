use std::ffi::CStr;
use std::process::Command;

pub const DEFAULT_FORMAT: &str = "[{user}@{host} {cwd}] {class} lvl{level}{prompt} ";

#[derive(Default)]
pub struct PromptContext {
    pub class: String,
    pub level: u8,
}

/// Render a prompt format string into the text shown to the user.
///
/// Recognized tokens, written in braces:
///   `{user}`   - current username
///   `{host}`   - short hostname (up to the first dot)
///   `{cwd}`    - working directory, home collapsed to `~`
///   `{dir}`    - basename of the working directory (like zsh `%c`)
///   `{prompt}` - `#` when root, otherwise `$`
///   `{class}`  - the player's class name
///   `{level}`  - the player's current level
///
/// `$(command)` is substituted with the command's output on every redraw, just
/// like in bash/zsh — so a git branch segment is written the usual way.
///
/// Colors/styles (ANSI SGR): `{reset}`, `{bold}`, and the eight base colors
/// `{black} {red} {green} {yellow} {blue} {magenta} {cyan} {white}`.
///
/// A literal brace is written `{{` or `}}`. An unknown token such as `{foo}`
/// is left verbatim so the mistake is visible rather than silently dropped.
pub fn render(format: &str, ctx: &PromptContext) -> String {
    let mut out = String::with_capacity(format.len());
    let mut chars = format.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '{' => {
                // `{{` is an escaped literal brace.
                if chars.peek() == Some(&'{') {
                    chars.next();
                    out.push('{');
                    continue;
                }

                let mut name = String::new();
                let mut closed = false;
                for next in chars.by_ref() {
                    if next == '}' {
                        closed = true;
                        break;
                    }
                    name.push(next);
                }

                if !closed {
                    out.push('{');
                    out.push_str(&name);
                    continue;
                }

                match name.as_str() {
                    "user" => out.push_str(&username()),
                    "host" => out.push_str(&hostname()),
                    "cwd" => out.push_str(&cwd()),
                    "dir" => out.push_str(&dir()),
                    "prompt" => out.push_str(prompt_char()),
                    "class" => out.push_str(&ctx.class),
                    "level" => out.push_str(&ctx.level.to_string()),
                    _ => match style_code(&name) {
                        Some(code) => out.push_str(code),
                        None => {
                            out.push('{');
                            out.push_str(&name);
                            out.push('}');
                        }
                    },
                }
            }
            '}' => {
                if chars.peek() == Some(&'}') {
                    chars.next();
                }
                out.push('}');
            }
            '$' if chars.peek() == Some(&'(') => {
                chars.next(); // consume '('

                let mut body = String::new();
                let mut depth = 1;
                let mut closed = false;

                for ch in chars.by_ref() {
                    if ch == '(' {
                        depth += 1;
                    } else if ch == ')' {
                        depth -= 1;
                        if depth == 0 {
                            closed = true;
                            break;
                        }
                    }
                    body.push(ch);
                }

                if closed {
                    out.push_str(&command_substitution(&body));
                } else {
                    out.push_str("$(");
                    out.push_str(&body);
                }
            }
            _ => out.push(c),
        }
    }

    out
}

fn username() -> String {
    if let Ok(user) = std::env::var("USER")
        && !user.is_empty()
    {
        return user;
    }

    // Fall back to the passwd database.
    unsafe {
        let pw = libc::getpwuid(libc::getuid());
        if !pw.is_null() && !(*pw).pw_name.is_null() {
            return CStr::from_ptr((*pw).pw_name).to_string_lossy().into_owned();
        }
    }

    String::new()
}

fn hostname() -> String {
    const LEN: usize = 256;
    let mut buf = [0u8; LEN];

    let ret = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, LEN) };
    if ret != 0 {
        return String::new();
    }

    let end = buf.iter().position(|&b| b == 0).unwrap_or(LEN);
    let full = String::from_utf8_lossy(&buf[..end]);
    full.split('.').next().unwrap_or("").to_string()
}

fn cwd() -> String {
    let cwd = std::env::current_dir().unwrap_or_default();
    let cwd = cwd.to_string_lossy();

    if let Some(home) = dirs::home_dir() {
        let home = home.to_string_lossy();
        if cwd == home {
            return "~".to_string();
        }
        if let Some(rest) = cwd.strip_prefix(&format!("{home}/")) {
            return format!("~/{rest}");
        }
    }

    cwd.into_owned()
}

fn dir() -> String {
    let cwd = std::env::current_dir().unwrap_or_default();
    match cwd.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        // The filesystem root has no trailing component.
        None => "/".to_string(),
    }
}

fn command_substitution(body: &str) -> String {
    let output = match Command::new("sh").arg("-c").arg(body).output() {
        Ok(output) => output,
        Err(_) => return String::new(),
    };

    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    while text.ends_with('\n') {
        text.pop();
    }
    text
}

fn style_code(name: &str) -> Option<&'static str> {
    Some(match name {
        "reset" => "\x1b[0m",
        "bold" => "\x1b[1m",
        "black" => "\x1b[30m",
        "red" => "\x1b[31m",
        "green" => "\x1b[32m",
        "yellow" => "\x1b[33m",
        "blue" => "\x1b[34m",
        "magenta" => "\x1b[35m",
        "cyan" => "\x1b[36m",
        "white" => "\x1b[37m",
        _ => return None,
    })
}

fn prompt_char() -> &'static str {
    if unsafe { libc::getuid() } == 0 {
        "#"
    } else {
        "$"
    }
}

#[cfg(test)]
mod tests {
    use super::PromptContext;

    /// Render with an empty context — for tests that don't exercise player tokens.
    fn render(format: &str) -> String {
        super::render(format, &PromptContext::default())
    }

    #[test]
    fn plain_text_is_unchanged() {
        assert_eq!(render("$ "), "$ ");
        assert_eq!(render(""), "");
    }

    #[test]
    fn class_and_level_tokens_reflect_the_player() {
        let ctx = PromptContext {
            class: "Mage".to_string(),
            level: 7,
        };
        assert_eq!(super::render("{class} lvl{level}", &ctx), "Mage lvl7");
    }

    #[test]
    fn unknown_token_is_left_verbatim() {
        assert_eq!(render("a{nope}b"), "a{nope}b");
    }

    #[test]
    fn escaped_braces_collapse_to_literals() {
        assert_eq!(render("{{user}}"), "{user}");
    }

    #[test]
    fn unterminated_token_is_left_verbatim() {
        assert_eq!(render("x{user"), "x{user");
    }

    #[test]
    fn color_tokens_expand_to_ansi() {
        assert_eq!(render("{green}x{reset}"), "\x1b[32mx\x1b[0m");
        assert_eq!(render("{bold}{cyan}"), "\x1b[1m\x1b[36m");
    }

    #[test]
    fn command_substitution_inlines_output() {
        assert_eq!(render("[$(printf hi)]"), "[hi]");
        // Trailing newlines are stripped, like shell `$(...)`.
        assert_eq!(render("$(printf 'a\\nb\\n')"), "a\nb");
    }

    #[test]
    fn unterminated_substitution_is_left_verbatim() {
        assert_eq!(render("x$(echo"), "x$(echo");
    }

    #[test]
    fn prompt_token_reflects_privilege() {
        let rendered = render("{prompt}");
        assert!(rendered == "$" || rendered == "#");
    }
}
