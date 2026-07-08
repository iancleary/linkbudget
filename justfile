# list recipes
help:
    just --list

# format the code
fmt:
	cargo fmt --all

# alias for fmt
format: fmt

# check formatting without writing changes
fmt-check:
	cargo fmt --all -- --check

# lint the code without writing changes
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# apply automatic clippy fixes
lint-fix:
	cargo clippy --all-targets --all-features --fix -- -Dclippy::all

# run the crate
dev:
  cargo run

# build the crate
build:
  cargo build --release

# run tests
test:
  cargo test --all-features

# check documentation with rustdoc warnings denied
doc-check:
  RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps

# verify package contents without publishing
package:
  cargo package

# format, lint, test, document, and package like CI
check: fmt-check lint test doc-check package

# run the same checks and build mirrored by CI
ci: check build

# Cut a GitHub release for an explicit SemVer version.
cut-release *args:
  ./scripts/cut-release.sh {{args}}
