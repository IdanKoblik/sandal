<div align="center">
  <h1>Sandal</h1>

  <img src="assets/banner.png" alt="banner" width="600">

  <p>Linux RPG shell / Refrence to <a href="https://en.wikipedia.org/wiki/Swords_and_Sandals">Swords and Sandals</a>.</p>
</div>

---

Sandal is a Linux shell that plays like an RPG. It runs your commands like any
other shell — external programs, builtins, pipelines, aliases, a configurable
prompt — but every command you run earns **XP**, levels up your character, and
trains five **attributes**. You pick a **class** when you start, and the way you
work decides how you grow.

## Contents

- [How it plays](#how-it-plays)
  - [Characters and classes](#characters-and-classes)
  - [Earning XP and leveling up](#earning-xp-and-leveling-up)
  - [Attributes](#attributes)
  - [The prompt](#the-prompt)
  - [Where your character is stored](#where-your-character-is-stored)
- [Download](#download)
  - [Build from source](#build-from-source)

## How it plays

On launch, sandal loads your saved character or helps you create a new one, then
drops you at a prompt. Run commands as usual; after each one it reports the XP
you earned and the attributes you trained.

### Characters and classes

The first time you play, sandal asks for a name and a class. Your class is fixed
for the life of the character and shapes how you grow through two buffs:

- **Affinity** — you train your class's primary *and* secondary attributes one
  point faster whenever a command exercises them.
- **Domain XP** — commands that train your primary attribute earn **+25% XP**,
  so you level up faster doing what your class is built for.

| Class       | Masters (primary) | Hones (secondary) | Domain               |
| ----------- | ----------------- | ----------------- | -------------------- |
| **Warrior** | Strength          | Agility           | feats of force       |
| **Mage**    | Intelligence      | Wisdom            | building things      |
| **Rogue**   | Agility           | Intelligence      | moving fast          |
| **Bard**    | Collaboration     | Wisdom            | working with others  |

### Earning XP and leveling up

Every command grants XP. A successful command is worth:

```
base (1)  +  name_length / 3  +  argument_bonus
```

Each positional argument is worth 1 and each flag (`-x` / `--long`) is worth 2,
but the argument bonus is capped at 6 — you can't farm XP by piling on arguments.
A failed command still teaches you something: a flat **2 XP**. A pipeline awards
the sum of its stages.

Reaching the next level costs `round(78 × level^1.5)` XP; any leftover rolls into
the new level. When you level up, sandal celebrates:

```
✨ Aria reached level 4!
```

### Attributes

Five attributes track the kind of work you do. Each command trains the one that
matches its nature:

| Attribute         | Trained by                                       | Examples                                     |
| ----------------- | ------------------------------------------------ | -------------------------------------------- |
| **Strength**      | heavy, destructive, or privileged operations     | `rm`, `mv`, `dd`, `kill`, `chmod`, `sudo`    |
| **Intelligence**  | building and programming                         | `cargo`, `rustc`, `gcc`, `make`, `python`    |
| **Agility**       | navigation and movement                          | `cd`, `ls`, `pushd`, `popd`                  |
| **Wisdom**        | reading, searching — and *every failed command*  | `man`, `grep`, `find`, `cat`, `less`, `tail` |
| **Collaboration** | version control, network, and sharing            | `git`, `ssh`, `scp`, `curl`, `gh`, `rsync`   |

Commands sandal doesn't recognise still earn XP but train no attribute. After
each command you see what you gained:

```
$ git commit -m "feat: add xp message"
earned +6 XP.
  +1 collaboration
```

(A Bard — whose primary is collaboration — would gain `+2` here instead of `+1`,
plus the +25% domain XP.)

### The prompt

The prompt is a format string of `{tokens}`. The default is:

```
[{user}@{host} {cwd}] {class} lvl{level}{prompt}
```

which renders as `[aria@host ~] Mage lvl4$`. Override it with the `PS1`
environment variable. Recognised tokens:

| Token      | Expands to                          |
| ---------- | ----------------------------------- |
| `{user}`   | username                            |
| `{host}`   | short hostname (up to the first dot)|
| `{cwd}`    | working directory, home as `~`      |
| `{dir}`    | basename of the working directory   |
| `{class}`  | your class name                     |
| `{level}`  | your current level                  |
| `{prompt}` | `#` when root, otherwise `$`        |

You can also use ANSI styles `{reset}` and `{bold}`, the base colors `{black}`
`{red}` `{green}` `{yellow}` `{blue}` `{magenta}` `{cyan}` `{white}`, and
`$(command)` substitution just like bash/zsh. Write a literal brace as `{{` or
`}}`.

### Where your character is stored

Characters are saved as JSON in your home directory, keyed by your username:

- `~/.sandal/users.json` — name, class, level, XP, and attributes
- `~/.sandal_history` — command history

## Download

Prebuilt `sandal` binary are published on the
[Releases](https://github.com/IdanKoblik/sandal/releases) page.

```sh
# Grab the latest release binary
curl -L -o sandal https://github.com/IdanKoblik/sandal/releases/latest/download/sandal
chmod +x sandal
./sandal
```

## Build from source

sandal is plain Rust with a `Makefile` — no build system to learn.

**Dependencies:**

- A recent stable [Rust toolchain](https://rustup.rs/) (`cargo`, `rustc`)
- `rustfmt` and `clippy` components (`rustup component add rustfmt clippy`)
- `make`

**Build:**

```sh
make build      # cargo build
make release    # cargo build --release
make run        # cargo run
```

Optional developer targets:

```sh
make test       # cargo test
make fmt        # cargo fmt
make fmt-check  # cargo fmt --check
make lint       # cargo clippy -- -D warnings
make check      # fmt-check + lint + test
make clean      # cargo clean
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for code conventions and how to add
tests.
