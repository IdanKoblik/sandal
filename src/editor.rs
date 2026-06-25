use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;

use crate::completion::{self, Completer};

/// The terminal is switched to raw mode only for the duration of the read, so
/// while a command runs afterwards the terminal behaves normally
pub fn read_line(prompt: &str, completer: &Completer) -> io::Result<Option<String>> {
    let _raw = RawMode::enable()?;
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut buf = String::new();
    write!(stdout, "{prompt}")?;
    stdout.flush()?;

    let mut byte = [0u8; 1];
    loop {
        if stdin.read(&mut byte)? == 0 {
            return Ok(None);
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
            b'\t' => complete(&mut buf, prompt, completer, &mut stdout)?,
            // Ctrl-C: abandon the current line.
            0x03 => {
                buf.clear();
                write!(stdout, "^C\r\n{prompt}")?;
                stdout.flush()?;
            }
            // Ctrl-D: end of input only when the line is empty.
            0x04 => {
                if buf.is_empty() {
                    write!(stdout, "\r\n")?;
                    return Ok(None);
                }
            }
            // Escape sequence (arrow keys etc.): consume and ignore so the
            // bytes don't leak into the buffer.
            0x1b => {
                let mut seq = [0u8; 2];
                let _ = stdin.read(&mut seq);
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

fn complete(
    buf: &mut String,
    prompt: &str,
    completer: &Completer,
    stdout: &mut io::Stdout,
) -> io::Result<()> {
    let start = buf.rfind(char::is_whitespace).map(|i| i + 1).unwrap_or(0);
    let word = &buf[start..];
    let is_command = buf[..start].trim().is_empty();

    // An empty command word would dump the entire $PATH; skip it.
    if word.is_empty() && is_command {
        return Ok(());
    }

    let candidates = completer.candidates(word, is_command);
    match candidates.as_slice() {
        [] => write!(stdout, "\x07")?, // bell
        [only] => {
            let suffix = &only[word.len()..];
            buf.push_str(suffix);
            write!(stdout, "{suffix}")?;
            // Directories end in '/' and stay open for the next component;
            if !only.ends_with('/') {
                buf.push(' ');
                write!(stdout, " ")?;
            }
        }
        _ => {
            let lcp = completion::longest_common_prefix(&candidates);
            if lcp.len() > word.len() {
                // Extend the line up to the shared prefix.
                let suffix = &lcp[word.len()..];
                buf.push_str(suffix);
                write!(stdout, "{suffix}")?;
            } else {
                write!(stdout, "\r\n")?;
                for candidate in &candidates {
                    write!(stdout, "{}  ", basename(candidate))?;
                }
                write!(stdout, "\r\n{prompt}{buf}")?;
            }
        }
    }

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
