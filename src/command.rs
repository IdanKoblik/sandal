use crate::internal::InternalCommand;
use crate::state::ShellState;
use std::process::{Command as ProcessCommand, Stdio};

pub struct Command<'a> {
    pub cmd: &'a str,
    pub kind: CommandKind<'a>,
}

pub enum CommandKind<'a> {
    Internal(InternalCommand<'a>),
    External(ExternalCommand<'a>),
    Pipeline(Vec<ExternalCommand<'a>>),
}

pub struct ExternalCommand<'a> {
    program: &'a str,
    args: Vec<&'a str>,
}

impl Command<'_> {
    pub fn execute(self, state: &mut ShellState) -> Result<(), Box<dyn std::error::Error>> {
        state.history.push_str(self.cmd);
        state.history.push('\n');

        match self.kind {
            CommandKind::Internal(cmd) => cmd.execute(state),
            CommandKind::External(cmd) => cmd.execute(),
            CommandKind::Pipeline(cmds) => execute_pipeline(cmds),
        }
    }
}

impl ExternalCommand<'_> {
    fn execute(self) -> Result<(), Box<dyn std::error::Error>> {
        match ProcessCommand::new(self.program).args(self.args).spawn() {
            Ok(mut child) => {
                child.wait()?;
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Err(format!("command not found: {}", self.program).into());
            }
            Err(err) => return Err(err.into()),
        }

        Ok(())
    }
}

fn execute_pipeline(cmds: Vec<ExternalCommand>) -> Result<(), Box<dyn std::error::Error>> {
    let last = cmds.len() - 1;
    let mut children = Vec::with_capacity(cmds.len());
    let mut prev_stdout = None;

    for (i, cmd) in cmds.into_iter().enumerate() {
        let stdin = match prev_stdout.take() {
            Some(out) => Stdio::from(out),
            None => Stdio::inherit(),
        };
        let stdout = if i == last {
            Stdio::inherit()
        } else {
            Stdio::piped()
        };

        let mut child = match ProcessCommand::new(cmd.program)
            .args(cmd.args)
            .stdin(stdin)
            .stdout(stdout)
            .spawn()
        {
            Ok(child) => child,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Err(format!("command not found: {}", cmd.program).into());
            }
            Err(err) => return Err(err.into()),
        };

        prev_stdout = child.stdout.take();
        children.push(child);
    }

    for mut child in children {
        child.wait()?;
    }

    Ok(())
}

fn tokenize(segment: &str) -> Vec<&str> {
    let bytes = segment.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }

        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let quote = bytes[i];
            i += 1;
            let start = i;
            while i < bytes.len() && bytes[i] != quote {
                i += 1;
            }
            tokens.push(&segment[start..i]);
            i += 1; // step past the closing quote (or the end)
        } else {
            let start = i;
            while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            tokens.push(&segment[start..i]);
        }
    }

    tokens
}

fn parse_external(segment: &str) -> ExternalCommand<'_> {
    let mut tokens = tokenize(segment).into_iter();
    let program = tokens.next().unwrap_or("");

    ExternalCommand {
        program,
        args: tokens.collect(),
    }
}

pub fn parse_command(input: &str) -> Command<'_> {
    if input.contains('|') {
        let segments = input.split('|').map(|seg| parse_external(seg.trim()));
        return Command {
            cmd: input,
            kind: CommandKind::Pipeline(segments.collect()),
        };
    }

    let mut tokens = tokenize(input).into_iter();
    let name = tokens.next().unwrap_or("");

    let kind = match name {
        "cd" => CommandKind::Internal(InternalCommand::Cd(tokens.next().unwrap_or(""))),
        "exit" => CommandKind::Internal(InternalCommand::Exit),
        "history" => CommandKind::Internal(InternalCommand::History),
        _ => CommandKind::External(ExternalCommand {
            program: name,
            args: tokens.collect(),
        }),
    };

    Command { cmd: input, kind }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cd_with_argument() {
        let cmd = parse_command("cd /tmp");
        assert!(
            matches!(cmd.kind, CommandKind::Internal(InternalCommand::Cd(path)) if path == "/tmp")
        );
    }

    #[test]
    fn parses_cd_with_quoted_argument() {
        let cmd = parse_command("cd \"my dir\"");
        assert!(
            matches!(cmd.kind, CommandKind::Internal(InternalCommand::Cd(path)) if path == "my dir")
        );
    }

    #[test]
    fn parses_bare_cd_with_empty_path() {
        let cmd = parse_command("cd");
        assert!(
            matches!(cmd.kind, CommandKind::Internal(InternalCommand::Cd(path)) if path.is_empty())
        );
    }

    #[test]
    fn parses_builtins() {
        assert!(matches!(
            parse_command("exit").kind,
            CommandKind::Internal(InternalCommand::Exit)
        ));
        assert!(matches!(
            parse_command("history").kind,
            CommandKind::Internal(InternalCommand::History)
        ));
    }

    #[test]
    fn parses_external_command_with_args() {
        let cmd = parse_command("ls -la /tmp");
        match cmd.kind {
            CommandKind::External(ext) => {
                assert_eq!(ext.program, "ls");
                assert_eq!(ext.args, vec!["-la", "/tmp"]);
            }
            _ => panic!("expected external command"),
        }
    }

    #[test]
    fn empty_input_is_external_with_empty_program() {
        match parse_command("").kind {
            CommandKind::External(ext) => assert_eq!(ext.program, ""),
            _ => panic!("expected external command"),
        }
    }

    #[test]
    fn parses_pipeline_segments_and_trims_whitespace() {
        let cmd = parse_command("ls -la | grep foo | wc -l");
        match cmd.kind {
            CommandKind::Pipeline(segments) => {
                let programs: Vec<_> = segments.iter().map(|s| s.program).collect();
                assert_eq!(programs, vec!["ls", "grep", "wc"]);
            }
            _ => panic!("expected pipeline"),
        }
    }

    #[test]
    fn command_retains_original_input() {
        assert_eq!(parse_command("cd /tmp").cmd, "cd /tmp");
    }
}
