use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;

use crate::completion::{self, Completer};

struct Menu {
    candidates: Vec<String>,
    index: usize,
    /// Byte offset in the buffer where the word being completed starts.
    word_start: usize,
}

impl Menu {
    fn apply(&self, buf: &mut String) {
        buf.truncate(self.word_start);
        buf.push_str(&self.candidates[self.index]);
    }
}

/// The terminal is switched to raw mode only for the duration of the read, so
/// while a command runs afterwards the terminal behaves normally
pub fn read_line(
    prompt: &str,
    completer: &Completer,
    history: &[String],
) -> io::Result<Option<String>> {
    let _raw = RawMode::enable()?;
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut buf = String::new();
    write!(stdout, "{prompt}")?;
    stdout.flush()?;

    let mut hist_idx = history.len();
    let mut stash = String::new();

    let mut menu: Option<Menu> = None;

    let mut byte = [0u8; 1];
    loop {
        if stdin.read(&mut byte)? == 0 {
            return Ok(None);
        }

        if byte[0] != b'\t' && menu.take().is_some() {
            write!(stdout, "\x1b[J")?;
            stdout.flush()?;
        }

        match byte[0] {
            b'\r' | b'\n' => {
                write!(stdout, "\r\n")?;
                stdout.flush()?;
                return Ok(Some(buf));
            }
            // Backspace (DEL / BS).
            0x7f | 0x08 => {
                if buf.pop().is_some() {
                    // Move left, overwrite with a space, move left again.
                    write!(stdout, "\x08 \x08")?;
                    stdout.flush()?;
                }
            }
            b'\t' => {
                if let Some(m) = menu.as_mut() {
                    // Already in a menu: advance the selection and redraw.
                    m.index = (m.index + 1) % m.candidates.len();
                    m.apply(&mut buf);
                    render_menu(&mut stdout, prompt, &buf, m)?;
                } else {
                    menu = start_completion(&mut buf, prompt, completer, &mut stdout)?;
                }
            }
            // Ctrl-C: abandon the current line.
            0x03 => {
                buf.clear();
                write!(stdout, "^C\r\n{prompt}")?;
                stdout.flush()?;
                hist_idx = history.len();
            }
            // Ctrl-D: end of input only when the line is empty.
            0x04 => {
                if buf.is_empty() {
                    write!(stdout, "\r\n")?;
                    return Ok(None);
                }
            }
            // Escape sequence: arrow keys (`ESC [ A`/`B`) browse history.
            0x1b => {
                let mut next = [0u8; 1];
                if stdin.read(&mut next)? == 0 {
                    return Ok(None);
                }
                if next[0] == b'[' || next[0] == b'O' {
                    if stdin.read(&mut next)? == 0 {
                        return Ok(None);
                    }
                    match next[0] {
                        b'A' if !history.is_empty() => {
                            if hist_idx == history.len() {
                                stash = buf.clone();
                            }
                            if hist_idx > 0 {
                                hist_idx -= 1;
                                buf = history[hist_idx].clone();
                                redraw_line(&mut stdout, prompt, &buf)?;
                            }
                        }
                        b'B' if hist_idx < history.len() => {
                            hist_idx += 1;
                            buf = if hist_idx == history.len() {
                                stash.clone()
                            } else {
                                history[hist_idx].clone()
                            };
                            redraw_line(&mut stdout, prompt, &buf)?;
                        }
                        // Multi-byte sequences (Home/End/Delete: `ESC [ 3 ~`):
                        // swallow the trailing tilde so it can't leak.
                        b'0'..=b'9' => {
                            let _ = stdin.read(&mut next);
                        }
                        _ => {}
                    }
                }
            }
            b if b.is_ascii_graphic() || b == b' ' => {
                buf.push(b as char);
                stdout.write_all(&[b])?;
                stdout.flush()?;
            }
            _ => {}
        }
    }
}

fn start_completion(
    buf: &mut String,
    prompt: &str,
    completer: &Completer,
    stdout: &mut io::Stdout,
) -> io::Result<Option<Menu>> {
    let start = buf.rfind(char::is_whitespace).map(|i| i + 1).unwrap_or(0);
    let word = &buf[start..];
    let prefix = buf[..start].trim();
    let is_command = prefix.is_empty();

    if word.is_empty() && is_command {
        return Ok(None);
    }

    // `cd` only ever takes a directory, so don't clutter the menu with files.
    let dirs_only = prefix.split_whitespace().next() == Some("cd");

    let candidates = completer.candidates(word, is_command, dirs_only);
    match candidates.as_slice() {
        [] => {
            write!(stdout, "\x07")?; // bell
            stdout.flush()?;
            Ok(None)
        }
        [only] => {
            let suffix = &only[word.len()..];
            buf.push_str(suffix);
            write!(stdout, "{suffix}")?;
            // Directories end in '/' and stay open for the next component;
            if !only.ends_with('/') {
                buf.push(' ');
                write!(stdout, " ")?;
            }
            stdout.flush()?;
            Ok(None)
        }
        _ => {
            let lcp = completion::longest_common_prefix(&candidates);
            if lcp.len() > word.len() {
                // Extend the line up to the shared prefix; a second Tab opens
                // the menu once there is nothing left in common.
                let suffix = &lcp[word.len()..];
                buf.push_str(suffix);
                write!(stdout, "{suffix}")?;
                stdout.flush()?;
                Ok(None)
            } else {
                let menu = Menu {
                    candidates,
                    index: 0,
                    word_start: start,
                };
                menu.apply(buf);
                render_menu(stdout, prompt, buf, &menu)?;
                Ok(Some(menu))
            }
        }
    }
}

fn redraw_line(stdout: &mut io::Stdout, prompt: &str, buf: &str) -> io::Result<()> {
    write!(stdout, "\r{prompt}{buf}\x1b[K")?;
    stdout.flush()
}

fn render_menu(stdout: &mut io::Stdout, prompt: &str, buf: &str, menu: &Menu) -> io::Result<()> {
    // Redraw the input line and erase whatever the previous menu drew below it.
    write!(stdout, "\r{prompt}{buf}\x1b[J")?;
    // Remember the end-of-input position so we can return to it (DECSC).
    write!(stdout, "\x1b7")?;
    write!(stdout, "\r\n")?;
    for (i, candidate) in menu.candidates.iter().enumerate() {
        let name = basename(candidate);
        if i == menu.index {
            write!(stdout, "\x1b[7m {name} \x1b[0m ")?;
        } else {
            write!(stdout, " {name}  ")?;
        }
    }
    // Restore the cursor to the input line (DECRC).
    write!(stdout, "\x1b8")?;
    stdout.flush()
}

fn basename(candidate: &str) -> &str {
    let trimmed = candidate.strip_suffix('/').unwrap_or(candidate);
    match trimmed.rfind('/') {
        Some(idx) => &candidate[idx + 1..],
        None => candidate,
    }
}

struct RawMode {
    fd: i32,
    original: libc::termios,
}

impl RawMode {
    fn enable() -> io::Result<Self> {
        let fd = io::stdin().as_raw_fd();
        unsafe {
            let mut original: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(fd, &mut original) != 0 {
                return Err(io::Error::last_os_error());
            }

            let mut raw = original;
            // Disable canonical mode (read byte-by-byte), echo (we draw the
            // line ourselves), and signal generation (Ctrl-C/D handled here).
            raw.c_lflag &= !(libc::ICANON | libc::ECHO | libc::ISIG);
            raw.c_cc[libc::VMIN] = 1;
            raw.c_cc[libc::VTIME] = 0;

            if libc::tcsetattr(fd, libc::TCSAFLUSH, &raw) != 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(RawMode { fd, original })
        }
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        unsafe {
            libc::tcsetattr(self.fd, libc::TCSAFLUSH, &self.original);
        }
    }
}
