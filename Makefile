.PHONY: all build run test fmt fmt-check lint check dist clean

BIN := sandal
DIST_DIR := dist

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

dist: release
	mkdir -p $(DIST_DIR)
	cp target/release/$(BIN) $(DIST_DIR)/$(BIN)

clean:
	cargo clean
