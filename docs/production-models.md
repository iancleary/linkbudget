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
| 2 | Atmospheric sky noise | P.618-14 eq. 69 → receiver source noise + path attenuation → full-link SNR |
| 3 | Rain attenuation/exceedance | Planned |
| 4 | Monte Carlo uncertainty | Planned |
| 5 | Gaussian-tail numerics | Planned |
| 6 | Finite-blocklength estimates | Planned |

The stack uses no automatic climate-data downloads, production writes, modem
simulations, or new runtime dependencies. Validation of formulas is not field
validation of their inputs or performance guarantees for a particular decoder.

## Atmospheric attenuation: update signal and noise together

[`atmosphere::sky_noise_temperature_k`](../src/atmosphere.rs) implements
[ITU-R P.618-14 (08/2023), section 3, equation 69](https://www.itu.int/dms_pubrec/itu-r/rec/p/R-REC-P.618-14-202308-I!!PDF-E.pdf):

`T_sky = T_m * (1 - t) + 2.7 * t`, where `t = 10^(-A/10)`.

The atmosphere both attenuates the incoming signal and emits thermal noise.
The same transmission factor attenuates the 2.7 K background. For a low-noise
receiver, the resulting noise increase can cost more SNR than attenuation alone.
`A` excludes scintillation and all nonabsorptive losses: never pass total link
path loss (FSPL, pointing, or other geometric losses) into this equation.

### Decision and composition

The API is a pure, validated scalar helper returning `Result<f64, ModelError>`.
Attenuation must be finite and nonnegative; atmospheric mean radiating temperature
must be finite and positive. The implementation uses `exp_m1` for the absorbed
fraction to avoid cancellation near zero attenuation. There are no dependencies,
climate lookups, or automatic changes to `ReceiverNoise` or `LinkBudget`.

Supply a mean radiating temperature from your model or observations. Section 3
suggests 275 K in the absence of local data; it is not an implicit API default.
For an ideal ground-station beam filled by this sky, without feed loss or
spillover, compose both effects explicitly:

```rust
use linkbudget::atmosphere::sky_noise_temperature_k;
use linkbudget::{LinkBudget, PathLoss, Receiver, ReceiverNoise, Transmitter};

let attenuation_db = 3.0; // atmospheric only, not FSPL or pointing
let sky_k = sky_noise_temperature_k(attenuation_db, 275.0)?;
let budget = LinkBudget {
    name: "Atmospheric emission: Ka-band LEO downlink",
    bandwidth: 20e6, // channel bandwidth, Hz
    transmitter: Transmitter {
        output_power: 10.0, // dBm
        gain: 35.0, // dBi
        bandwidth: 20e6,
    },
    receiver: Receiver {
        gain: 40.0, // dBi
        noise: ReceiverNoise::SourceAndNoiseFigure {
            source_temperature_k: sky_k,
            noise_figure_db: 1.0,
        },
        bandwidth: 20e6, // equivalent noise bandwidth, Hz
    },
    path_loss: PathLoss { frequency: 20e9, distance: 550e3 }, // Hz, m
    frequency_dependent_loss: Some(attenuation_db),
};
assert!((sky_k - 138.52671628329376).abs() < 1e-10);
assert!((budget.snr() - 11.016911094178127).abs() < 1e-10);
# Ok::<(), linkbudget::ModelError>(())
```

### Verification and limits

The public workflow in [`tests/published_models.rs`](../tests/published_models.rs)
checks literal analytic reference values, independently evaluated with 50-digit
decimal arithmetic using equation 69, Friis, and SI Boltzmann's constant. At zero
attenuation, sky temperature is 2.7 K, total receiver temperature is 77.788369 K,
and SNR is 18.404084 dB. At 3 dB attenuation, sky temperature is 138.526716 K,
total temperature is 213.615086 K, and SNR is 11.016911 dB. Received power falls
from -88.275637 to -91.275637 dBm, while noise rises from -106.679721 to
-102.292548 dBm. The total SNR penalty is 7.387173 dB, not just 3 dB.
Boundary checks cover zero, tiny, and very large attenuation and rejected
negative/nonfinite attenuation or nonpositive/nonfinite temperature.
Run `CARGO_NET_OFFLINE=true just fmt-check lint test doc-check` to verify locally.

This is sky **brightness**, not full antenna or system noise temperature. Real
antennas require beam weighting, spillover, feed loss, other sky sources (including
the Sun and galactic background), and receiver noise at a common input plane.
Do not put this result in `SystemTemperature` without including those terms, or
apply receiver noise figure again if a supplied system temperature already
includes it. A result of 2.7 K at `A = 0` is the model's background limit, not a
claim that a real clear sky has zero atmospheric attenuation. The helper predicts
neither attenuation nor weather statistics; field validity depends on the caller's
inputs and antenna model. If SNR drops by only the entered attenuation, check
that source noise was updated too; if sky noise approaches `T_m` unexpectedly,
check that FSPL or pointing loss was not included in `A`.

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
