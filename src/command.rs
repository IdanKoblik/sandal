use crate::internal::InternalCommand;
use crate::state::ShellState;
use std::process::{Command as ProcessCommand, Stdio};

pub struct Command<'a> {
    pub cmd: &'a str,
    pub kind: CommandKind<'a>
}

pub enum CommandKind<'a> {
    Internal(InternalCommand<'a>),
    External(ExternalCommand<'a>),
    Pipeline(Vec<ExternalCommand<'a>>),
}

pub struct ExternalCommand<'a> {
    program: &'a str,
    args: std::str::SplitWhitespace<'a>,
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

fn parse_external(segment: &str) -> ExternalCommand<'_> {
    let mut parts = segment.split_whitespace();
    let program = parts.next().unwrap_or("");

    ExternalCommand {
        program,
        args: parts,
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

    let mut parts = input.split_whitespace();
    let name = parts.next().unwrap_or("");

    let kind = match name {
        "cd" => CommandKind::Internal(InternalCommand::Cd(parts.next().unwrap_or_default())),
        "exit" => CommandKind::Internal(InternalCommand::Exit),
        "history" => CommandKind::Internal(InternalCommand::History),
        _ => CommandKind::External(ExternalCommand {
            program: name,
            args: parts,
        }),
    };

    Command {
        cmd: input,
        kind,
    }
}
