# Agent operating loop

This crate is small enough that future changes should stay explainable from the
public RF model outward. Treat the system as a link-budget toolbox, not a bag of
numeric helpers: additions should either improve an end-to-end link question or
make one of the supporting RF calculations clearer, safer, or easier to verify.

## Start here

Before editing, read these surfaces in order:

1. `README.md` for the public story, examples, and crate boundaries.
2. `AGENTS.md` or `CLAUDE.md` for command and release expectations.
3. `src/lib.rs` for the exported API surface.
4. The module being changed, plus its inline unit tests.
5. `tests/readme_examples.rs` when examples or public API usage changes.
6. `tests/scenarios.rs` when behavior affects end-to-end link closure.

If a change only touches release mechanics, start with `docs/release.md`,
`justfile`, and `scripts/cut-release.sh` instead.

## Preserve the RF contract

The most important design constraint is unit legibility. Public structs and
functions use plain `f64`, so names, docs, examples, and tests carry the unit
contract:

- powers are dBm or dBW as named;
- gains are dBi;
- frequencies and bandwidths are Hz;
- distances are meters;
- temperatures are kelvin;
- rates are bps or symbols/s as named;
- C/No is dB-Hz;
- SNR, Eb/No, Es/No, Ec/No, losses, gains, and margins are dB.

When changing formulas, keep occupied bandwidth, receiver noise bandwidth, bit
rate, symbol rate, and FEC code rate distinct. If a calculation combines them,
name each term in code or tests so a later agent can audit the link equation
without reverse-engineering intent from constants.

## Make changes accretive

Prefer changes that leave a future agent with another reliable foothold:

- Add a README example only when it represents a public workflow a crate user
  would plausibly copy. Mirror it in `tests/readme_examples.rs`.
- Add a scenario test when a formula change affects realistic link closure,
  margin, sensitivity, BER, PFD, Doppler, or orbit behavior.
- Keep module-level unit tests near the formula they protect when the behavior is
  local and does not need a full `LinkBudget`.
- Re-export from `src/lib.rs` only when the item is meant to be part of the
  stable crate surface.
- Update `AGENTS.md` and `CLAUDE.md` together when command, release, or agent
  routing guidance changes.

Avoid adding generic process docs, issue templates, or planning files unless
they encode a real repo-specific workflow. This repository's durable operating
knowledge belongs in `AGENTS.md`, `CLAUDE.md`, `docs/release.md`, and this file.

## Verification ladder

Match checks to the blast radius:

- Docs-only changes: run `git diff --check`.
- README example changes: run `cargo test --test readme_examples`.
- Formula, API, or scenario changes: run `cargo test --all-features`.
- Public API or docs.rs-facing changes: run
  `RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps`.
- Before a release or broad refactor: run `just check` or `just ci`.

Use `cargo fmt --all -- --check` when editing Rust. Use
`cargo clippy --all-targets --all-features -- -D warnings` when changing code
paths, public APIs, or tests.

## Release posture

This is a public crates.io crate. Keep the release path deterministic and avoid
manual version bumps outside `just cut-release --version <semver>`. Release
workflow changes should preserve the contract in `docs/release.md`: explicit
SemVer input, dry-run restoration, validation before tagging, and GitHub release
creation as the final public step.
