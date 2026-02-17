# CLAUDE.md — linkbudget

## Overview

Rust crate for RF link budget analysis. Covers the full TX → path loss → RX → SNR → Eb/No → BER → margin chain for satellite and terrestrial communication systems. Published on crates.io (v0.5.0).

## Commands

```bash
cargo test                        # Run all 137 tests
cargo clippy -- -D warnings       # Lint
cargo fmt -- --check              # Format check
cargo run                         # Run built-in CLI example
cargo doc --open                  # Generate and view API docs
```

## Module Map

| Module | File | Description |
|--------|------|-------------|
| `budget` | `src/budget.rs` | `LinkBudget` struct — end-to-end link budget integration |
| `transmitter` | `src/transmitter.rs` | `Transmitter` — output power, gain, EIRP |
| `receiver` | `src/receiver.rs` | `Receiver` — gain, noise temperature, NF, G/T |
| `path_loss` | `src/path_loss.rs` | `PathLoss` — free space path loss from freq + distance |
| `modulation` | `src/modulation.rs` | `Modulation` enum — BPSK, QPSK, M-PSK, M-QAM, MSK |
| `ber` | `src/ber.rs` | Theoretical BER curves, required Eb/No, link margin |
| `energy` | `src/energy.rs` | Eb/No, Es/No, Ec/No, C/No conversions |
| `coding` | `src/coding.rs` | `CodedModulation`, `FecCode`, DVB-S2 presets |
| `sensitivity` | `src/sensitivity.rs` | Receiver MDS — matched filter and bandpass |
| `evm` | `src/evm.rs` | EVM ↔ SNR conversions and margin checking |
| `doppler` | `src/doppler.rs` | Doppler shift and received frequency |
| `pfd` | `src/pfd.rs` | Power flux density (dBW/m²) and PFD/MHz |
| `orbits` | `src/orbits.rs` | Slant range, circular orbit speed/period |
| `phy` | `src/phy.rs` | Shannon capacity from SNR and bandwidth |
| `quantization` | `src/quantization.rs` | ADC/DAC quantization SNR and ENOB |
| `constants` | `src/constants.rs` | Physical constants |
| `cli` | `src/cli.rs` | CLI entry point |
| `plot` | `src/plot.rs` | HTML plot generation |

## Where to Look

- **README.md** — Full API examples for every module with usage patterns
- **src/lib.rs** — Public re-exports; shows the full public API surface
- **src/budget.rs** — Core `LinkBudget` struct that ties everything together
- **src/coding.rs** — DVB-S2 presets and FEC integration
- **src/ber.rs** — BER curves and link margin calculations
- Tests are co-located in each module file
