# list recipes
help:
    just --list

# format the code
fmt:
	cargo fmt --all

# alias for fmt
format: fmt

# lint the code
lint:
	cargo clippy --all-targets --all-features --fix -- -Dclippy::all

# run the crate
dev:
  cargo run

# build the crate
build:
  cargo build --release

# run tests
test:
  cargo test

# Lint and then test targets (like CI does)
ci: lint test build
