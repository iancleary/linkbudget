# list recipes
help:
    just --list

# lint the code
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# dev server
dev:
  cargo run

# build the app
build:
  cargo build --release

test:
  cargo test

# Lint and then test targets (like CI does)
ci: lint test build
