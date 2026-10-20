# Published reference models in production

These APIs separate a published model's assumptions from a hardware prediction.
They complement, rather than replace, the derivations in the [calculation guides](README.md).
The [research shortlist](literature-opportunities.md) records the original proposals.

## Rollout and verification

This work is delivered as six stacked changes related to issue #84. Each change
must pass `just fmt-check lint test doc-check` before the next one is built.
The baseline passed those checks before implementation. New observable workflows
live in `tests/published_models.rs`; these use external table entries or analytic
values rather than proving only that a forward/inverse pair agrees.

| Step | Slice | Verification/result |
| --- | --- | --- |
| 1 | DVB-S2 reference profiles | Table 13 rows, allocation → selection → rate; baseline API retained |
| 2 | Atmospheric sky noise | Planned |
| 3 | Rain attenuation/exceedance | Planned |
| 4 | Monte Carlo uncertainty | Planned |
| 5 | Gaussian-tail numerics | Planned |
| 6 | Finite-blocklength estimates | Planned |

The stack uses no automatic climate-data downloads, production writes, modem
simulations, or new runtime dependencies. Validation of formulas is not field
validation of their inputs or performance guarantees for a particular decoder.

## DVB-S2: compare like error criteria

[`dvbs2`](../src/dvbs2.rs) contains five **normal-frame, no-pilots profiles** from
[ETSI EN 302 307-1 V1.4.1 (2014-11), section 6, Table 13](https://www.etsi.org/deliver/etsi_en/302300_302399/30230701/01.04.01_60/en_30230701v010401p.pdf).
The source's simulated thresholds assume AWGN, perfect synchronization, no phase
noise, and 50 fixed-point LDPC iterations. The criterion is PER 1e-7 for useful
188-byte transport-stream packets, not arbitrary BER. `dvbs2::REFERENCE` keeps
those conditions beside the values. Closed mode variants prevent accidental
requests for short frames or unsupported criteria.

The values are factual table entries independently transcribed for interoperability;
no ETSI document text, decoder code, diagrams, or PDF payload is redistributed.
ETSI retains rights in its document; consult its notices before redistributing it.

Start with C/N0 at the common receiver input plane. Specify symbol rate rather
than using a noise-bandwidth label as a proxy:

```rust
use linkbudget::dvbs2::{self, Mode};

let rs = dvbs2::symbol_rate_from_occupied_bandwidth(12e6, 0.2)?; // 10 Msymbol/s
let candidates = [Mode::QpskR12, Mode::QpskR34];
let selected = dvbs2::select_mode(&candidates, 74.5, rs, 0.5)?; // C/N0 dB-Hz
assert_eq!(selected, Some(Mode::QpskR12));
let rate_bps = selected.unwrap().information_rate_bps(rs)?;
assert_eq!(rate_bps, 9_888_580.0);
# Ok::<(), linkbudget::ModelError>(())
```

This uses `Es/N0 = C/N0 - 10 log10(Rs)` in dB. Available Es/N0 is 4.5 dB;
subtracting the 0.5 dB implementation allowance leaves margins of +3.00 dB for
QPSK 1/2 and -0.03 dB for QPSK 3/4. An empty candidate set or no closing mode
returns `Ok(None)`. Invalid finite-domain inputs return `ModelError` rather than
producing a misleading selection.

Information bits per symbol include the reference system's framing overhead;
they are not just `log2(M)*code_rate`. They are also not net application throughput.
A pilot-bearing system or different framing needs another justified efficiency
model; do not reuse the reference rate as if overhead were absent.

### Compatibility and troubleshooting

- Existing `coding::dvbs2_*` constructors and `LinkBudget` methods retain their
  heuristic BER behavior. New code requiring sourced normal-frame performance
  should use `dvbs2::Mode`, including the real APSK identities. No silent change
  to the old QAM substitutes is made in a compatible release.
- A positive profile margin is against the stated ideal reference plus your
  allowance, not proof that a measured modem meets PER 1e-7.
- `LinkBudget::throughput_bps` assumes zero roll-off and is not used here.
- The selector is stateless; fade dynamics, feedback delay, and hysteresis belong
  in a separate operational ACM policy.
