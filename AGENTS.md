# AGENTS.md - linkbudget

Rust crate for RF link budget analysis across TX, path loss, RX, SNR, Eb/No,
BER, and margin workflows for satellite and terrestrial communication systems.

## Commands

```bash
cargo test
cargo clippy -- -D warnings
cargo fmt -- --check
cargo run
cargo doc --open
just cut-release --dry-run --version <semver>
```

## Releases

Maintain the deterministic release workflow with `create-release-process`.
Execute ordinary releases with `cut-release` via `just cut-release`; see
`docs/release.md` for the repo-local contract. The runner requires an explicit
SemVer `--version`, supports read-only version queries, and creates the GitHub
release as the final public step of a real release.

## Notes

- Keep changes minimal and aligned to the crate's RF link-budget purpose.
- Run `cargo fmt -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` for behavior changes.
- Claude Code guidance lives in `CLAUDE.md`; keep both files consistent when changing repo workflows.
