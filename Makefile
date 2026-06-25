.PHONY: all build run test fmt fmt-check lint check clean

all: build

build:
	cargo build

release:
	cargo build --release

run:
	cargo run

test:
	cargo test

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

lint:
	cargo clippy -- -D warnings

check: fmt-check lint test

clean:
	cargo clean
