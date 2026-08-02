# AGENTS.md - linkbudget

Rust crate for RF link budget analysis across TX, path loss, RX, SNR, Eb/No,
BER, and margin workflows for satellite and terrestrial communication systems.

## Agent Usage

Use `linkbudget` when the task is an end-to-end communication link question:
EIRP, FSPL, receiver G/T, C/No, SNR, Eb/No, BER, fade margin, modulation/FEC,
PFD, Doppler, orbit geometry, receiver sensitivity, EVM, or quantization. Build
a `LinkBudget` from `Transmitter`, `PathLoss`, and `Receiver` when the whole
link matters; use the smaller modules directly for isolated formulas.

Do not use this crate for raw `.sNp` parsing or network-parameter conversion;
use `touchstone`. Do not use it for ordered hardware-block lineup analysis with
P1dB/IP3 per stage; use `gainlineup`. Use `rfconversions` for standalone dB,
dBm, wavelength, and noise-temperature conversions.

Keep units explicit in examples and fixes: powers are generally dBm or dBW as
named, antenna gains are dBi, frequencies are Hz, distances are meters,
bandwidths and bit/symbol rates are Hz or bps as named, C/No is dB-Hz, and
Eb/No/SNR/margins are dB. Avoid mixing occupied bandwidth, noise bandwidth, bit
rate, symbol rate, and FEC code rate without naming each one.

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
