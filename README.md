<div align="center">
  <h1>Sandal</h1>

  <img src="assets/banner.png" alt="banner" width="600">

  <p>Linux RPG shell / Refrence to <a href="https://en.wikipedia.org/wiki/Swords_and_Sandals">Swords and Sandals</a>.</p>
</div>

---

**TODO DESCRIPTION**

> ⚠️ Currently this project is in active developing
>

## Contents

- [Build from source](#build-from-source)

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
