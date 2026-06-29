use crate::game_engine::player::Attribute;

pub struct Experience {
    /// Flat reward for running any non-empty command.
    pub base: u32,
    /// XP per positional argument.
    pub per_positional: u32,
    /// XP per flag (`-x`/`--long`) — flags signal more deliberate use.
    pub per_flag: u32,
    /// Tool-name length is divided by this; longer names ~ more specialised
    /// tools, so they earn a little more.
    pub name_divisor: u32,
    /// Ceiling on the argument contribution, so a command can't be farmed by
    /// piling on arguments.
    pub max_arg_bonus: u32,
    /// XP for a failed command — you still learn from mistakes.
    pub learning: u32,
}

impl Default for Experience {
    fn default() -> Self {
        Self {
            base: 1,
            per_positional: 1,
            per_flag: 2,
            name_divisor: 3,
            max_arg_bonus: 6,
            learning: 2,
        }
    }
}

impl Experience {
    /// Total XP for an executed command, accounting for whether it succeeded.
    pub fn award(&self, program: &str, args: &[&str], success: bool) -> u32 {
        if success {
            self.command_xp(program, args)
        } else {
            self.learning
        }
    }

    /// formula for XP for a single successful command invocation:
    /// `base + name_len / name_divisor + capped(sum of argument values)`.
    pub fn command_xp(&self, program: &str, args: &[&str]) -> u32 {
        if program.is_empty() {
            return 0;
        }

        let name_score = program.len() as u32 / self.name_divisor.max(1);
        let arg_score: u32 = args
            .iter()
            .map(|arg| self.arg_value(arg))
            .sum::<u32>()
            .min(self.max_arg_bonus);

        self.base + name_score + arg_score
    }

    pub fn pipeline_xp(&self, stages: &[(&str, &[&str])]) -> u32 {
        stages
            .iter()
            .map(|(program, args)| self.command_xp(program, args))
            .sum()
    }

    pub fn attributes(
        &self,
        program: &str,
        _args: &[&str],
        success: bool,
    ) -> Vec<(Attribute, u32)> {
        if program.is_empty() {
            return Vec::new();
        }
        if !success {
            return vec![(Attribute::Wisdom, 1)];
        }

        match program {
            "rm" | "mv" | "dd" | "kill" | "chmod" | "chown" | "sudo" | "mount" => {
                vec![(Attribute::Strength, 1)]
            }
            "cargo" | "rustc" | "gcc" | "cc" | "make" | "python" | "python3" | "node" => {
                vec![(Attribute::Intelligence, 1)]
            }
            "git" | "ssh" | "scp" | "curl" | "wget" | "gh" | "rsync" => {
                vec![(Attribute::Collaboration, 1)]
            }
            "man" | "grep" | "find" | "cat" | "less" | "more" | "head" | "tail" => {
                vec![(Attribute::Wisdom, 1)]
            }
            "cd" | "ls" | "pushd" | "popd" => vec![(Attribute::Agility, 1)],
            _ => Vec::new(),
        }
    }

    pub fn pipeline_attributes(&self, stages: &[(&str, &[&str])]) -> Vec<(Attribute, u32)> {
        stages
            .iter()
            .flat_map(|(program, args)| self.attributes(program, args, true))
            .collect()
    }

    fn arg_value(&self, arg: &str) -> u32 {
        if arg.len() > 1 && arg.starts_with('-') {
            self.per_flag
        } else {
            self.per_positional
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_command_is_base_plus_name_length() {
        let xp = Experience::default();
        assert_eq!(xp.command_xp("ls", &[]), 1); // 1 + 2/3
        assert_eq!(xp.command_xp("cat", &[]), 2); // 1 + 3/3
        assert_eq!(xp.command_xp("grep", &[]), 2); // 1 + 4/3
        assert_eq!(xp.command_xp("python3", &[]), 3); // 1 + 7/3
    }

    #[test]
    fn empty_program_earns_nothing() {
        assert_eq!(Experience::default().command_xp("", &["whatever"]), 0);
    }

    #[test]
    fn positional_args_and_flags_are_valued_differently() {
        let xp = Experience::default();
        // "git" -> 1 + 3/3 = 2; positional "commit" -> +1
        assert_eq!(xp.command_xp("git", &["commit"]), 3);
        // flag "-m" worth 2, positional "msg" worth 1
        assert_eq!(
            xp.command_xp("git", &["commit", "-m", "msg"]),
            2 + 1 + 2 + 1
        );
        // a lone "-" is a positional (stdin), not a flag
        assert_eq!(xp.command_xp("cat", &["-"]), 3);
    }

    #[test]
    fn argument_bonus_is_capped() {
        let xp = Experience::default();
        let many = ["a", "b", "c", "d", "e", "f", "g", "h"];
        // name score 0 + base 1 + capped arg bonus 6
        assert_eq!(xp.command_xp("ls", &many), 1 + 6);
    }

    #[test]
    fn pipelines_sum_their_stages() {
        let xp = Experience::default();
        let stages: &[(&str, &[&str])] = &[("ls", &["-la"]), ("grep", &["foo"]), ("wc", &["-l"])];
        // (1+0+2) + (1+1+1) + (1+0+2) = 3 + 3 + 3
        assert_eq!(xp.pipeline_xp(stages), 9);
    }

    #[test]
    fn failures_grant_the_learning_reward() {
        let xp = Experience::default();
        assert_eq!(xp.award("gcc", &["main.c"], false), xp.learning);
        assert_eq!(
            xp.award("gcc", &["main.c"], true),
            xp.command_xp("gcc", &["main.c"])
        );
    }

    #[test]
    fn commands_train_their_themed_attribute() {
        let xp = Experience::default();
        assert_eq!(
            xp.attributes("rm", &["-rf", "x"], true),
            vec![(Attribute::Strength, 1)]
        );
        assert_eq!(
            xp.attributes("cargo", &["build"], true),
            vec![(Attribute::Intelligence, 1)]
        );
        assert_eq!(
            xp.attributes("ls", &[], true),
            vec![(Attribute::Agility, 1)]
        );
        assert_eq!(
            xp.attributes("grep", &["foo"], true),
            vec![(Attribute::Wisdom, 1)]
        );
        assert_eq!(
            xp.attributes("git", &["push"], true),
            vec![(Attribute::Collaboration, 1)]
        );
    }

    #[test]
    fn failures_only_train_wisdom() {
        let xp = Experience::default();
        assert_eq!(
            xp.attributes("cargo", &["build"], false),
            vec![(Attribute::Wisdom, 1)]
        );
    }

    #[test]
    fn unknown_and_empty_programs_train_nothing() {
        let xp = Experience::default();
        assert!(xp.attributes("frobnicate", &[], true).is_empty());
        assert!(xp.attributes("", &["x"], true).is_empty());
    }

    #[test]
    fn pipeline_attributes_accumulate_per_stage() {
        let xp = Experience::default();
        let stages: &[(&str, &[&str])] = &[("ls", &["-la"]), ("grep", &["foo"]), ("git", &["log"])];
        assert_eq!(
            xp.pipeline_attributes(stages),
            vec![
                (Attribute::Agility, 1),
                (Attribute::Wisdom, 1),
                (Attribute::Collaboration, 1),
            ]
        );
    }

    #[test]
    fn coefficients_are_tunable() {
        let xp = Experience {
            base: 5,
            per_positional: 3,
            ..Experience::default()
        };
        assert_eq!(xp.command_xp("ls", &["x"]), 5 + 3);
    }
}
