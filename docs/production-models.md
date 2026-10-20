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
| 3 | Rain attenuation/exceedance | P.838-3 + P.618-14 annual rain quantiles → explicit full-link loss; pinned validation CSV and independent numeric cases |
| 4 | Monte Carlo uncertainty | Real seeded example CLI + independent CSV/report statistics; analytic receiver/rate control |
| 5 | Gaussian-tail numerics | Planned |
| 6 | Finite-blocklength estimates | Planned |

The stack uses no automatic climate-data downloads, production writes, modem
simulations, or new runtime dependencies. Validation of formulas is not field
validation of their inputs or performance guarantees for a particular decoder.

## Monte Carlo: conditional uncertainty, not availability

The [receiver example](../examples/montecarlo-cn0-target/README.md) reuses
`MonteCarloEngine` for 50,000 trials at seed 99, with named receiver samples and
explicit 10 Mbps information rate. It converts each receiver SNR through C/N0
to Eb/N0 before comparing with the real uncoded QPSK BER 1e-6 threshold. A failed
threshold calculation propagates an error. Noise bandwidth is not information
rate; under flat noise and full signal capture it cancels from Eb/N0.

[JCGM 101:2008](https://www.bipm.org/documents/20126/2071204/JCGM_101_2008_E.pdf),
*Evaluation of measurement data — Supplement 1 to the GUM — Propagation of
distributions using a Monte Carlo method*, sections 5.9–5.10 and 7.2–7.9,
provides the distribution-propagation framework. This example uses a fixed trial
count, **not** its adaptive numerical-tolerance procedure and not a claim of
complete JCGM conformance. Its independent uniforms are illustrative, not climate
data. Three distinct uncertainties matter:

- **Input uncertainty/spread:** choices of distributions, bounds, and joint
  dependence determine the modeled margin distribution. The example makes the
  independence assumption explicit; measured correlations are not available.
- **Monte Carlo sampling uncertainty:** the finite count of margins below zero
  estimates a conditional failure probability. The two-sided 95% Wilson score
  interval uses z=1.959963984540054; see E. B. Wilson (1927),
  [*Probable Inference, the Law of Succession, and Statistical Inference*](https://doi.org/10.1080/01621459.1927.10502953).
  If no failures occur, an additional one-sided exact 95% upper bound is
  `-expm1(log(0.05)/N)` (binomial zero-count inversion). These are different
  intervals, not bounds on annual availability. Margin quantiles describe spread,
  not confidence intervals on the quantiles themselves.
- **Model discrepancy:** ideal AWGN/QPSK, complete signal capture, and thermal
  noise omit real receiver impairments and time-varying weather. More trials
  cannot correct these omissions or unjustified input distributions.

Actual offline verification ran the example CLI with its real dependencies and
checked the sorted `eb_no_margin_db` CSV against the summary using the
[stdlib-only verifier](../examples/montecarlo-cn0-target/verify_output.py).
Observed N=50,000, failures=4,039, p=0.08078, Wilson95=
[0.078423583224, 0.083200828482]; p05/p50/p95 margins=
-0.407631310 / 4.267138194 / 8.880109851 dB. The independent zero-count upper
bound at this N is 5.99128506245549e-5. The public Receiver/energy workflow
checks SI thermal noise at 340 K, -90 dBm against C/N0=83.28437800279511 dB-Hz
and Eb/N0=13.284378002795108 dB at 10 Mbps, across three noise bandwidths and a
doubled-rate control. No repeated-run coverage study, plotting runtime check,
field validation, or adaptive convergence proof is claimed.

For future changes, run the example and verifier **from its directory** as shown
in its README, then the root `CARGO_NET_OFFLINE=true just fmt-check lint test
doc-check` gates. Keep the CSV header, plot reader, report labels, and README
paths in sync. The old tracked SNR PNG is historical and must not be presented
as current evidence. Do not update input distributions from rain quantiles
without a separately justified joint stochastic model and time basis.

## Rain: annual exceedance is not pass availability

[`rain`](../src/rain.rs) implements two pure offline models:

- `specific_attenuation_db_per_km`: [ITU-R P.838-3 (03/2005)](https://www.itu.int/rec/R-REC-P.838-3-200503-I/en),
  equations 1–5 and Tables 1–4, over 1–1000 GHz. It evaluates `gamma = k R^alpha`
  using frequency-dependent H/V coefficients and elevation/polarization geometry.
- `RainPath::attenuation_exceeded_db`: [ITU-R P.618-14 (08/2023)](https://www.itu.int/rec/R-REC-P.618-14-202308-I/en),
  section 2.2.1.1 steps 1–10, over 1–55 GHz and 0.001–5 percent annual exceedance.
  It computes the rain-layer slant path (including effective Earth curvature below
  5°), horizontal reduction, latitude-dependent vertical adjustment, A0.01, then
  the percentage scaling with the tropical/elevation branches of step 10.

The caller supplies rain height (for example from P.839) and **one-minute point
rain rate exceeded for 0.01% of an average year** (R0.01, for example from P.837).
An instantaneous rain gauge reading is not a replacement for R0.01. Heights use
meters above a common mean-sea-level datum, not height above the antenna. There
are no embedded maps, network lookups, dependencies, or implicit climate defaults.

```rust
use linkbudget::rain::{specific_attenuation_db_per_km, RainPath};

let gamma = specific_attenuation_db_per_km(20e9, 50.0, 30.0, 45.0)?;
assert!((gamma - 5.073415344222843).abs() < 1e-10); // dB/km
let rain = RainPath {
    frequency_hz: 20e9,
    elevation_deg: 30.0,
    latitude_deg: -10.0,
    station_height_m: 100.0,
    rain_height_m: 5000.0,
    rain_rate_001_mm_h: 80.0,
    polarization_tilt_deg: 90.0, // H=0°, V=90°, circular=45°
};
// 99.9% annual rain-only availability -> 0.1 percent exceedance, NOT 99.9.
let attenuation_db = rain.attenuation_exceeded_db(0.1)?;
assert!((attenuation_db - 17.772331503917453).abs() < 1e-10);
# Ok::<(), linkbudget::ModelError>(())
```

For a `LinkBudget`, put the returned attenuation in `frequency_dependent_loss`,
adding other justified **signal** losses once rather than replacing or duplicating
FSPL. The integration workflow in [`tests/published_models.rs`](../tests/published_models.rs)
uses this example with 10 dBm TX power, 35/40 dBi TX/RX gains, 550 km range,
20 MHz channel/noise bandwidths, and an explicitly supplied 200 K system noise
temperature. Received power, C/N0, and SNR all fall by 17.772332 dB with that
fixed noise. This deliberately isolates the signal-loss term; a real wet-sky
receiver also needs a justified emission/antenna-noise model. A rain quantile
alone does not specify the absorbing component, radiating temperature, or joint
weather statistics. **Never feed scintillation or other nonabsorptive losses into
`atmosphere::sky_noise_temperature_k`.** Do not silently treat a rain-only result
as total atmospheric attenuation for a noise calculation.

### Reference provenance and numerical evidence

The standards' factual coefficients and equations were transcribed and checked
against the source PDFs (including visual checks of the equations and coefficient
tables); no standard text, PDF, diagrams, or third-party implementation code is
redistributed here. ITU retains rights in its publications. The implementation
is written for this crate; the references below supply interoperability data,
not a license to redistribute their software or documents.

Reference revision: [ITU-Rpy `f739993c4b6d34076de22249ef53d03fa5a53d73`](https://github.com/inigodelportillo/ITU-Rpy/tree/f739993c4b6d34076de22249ef53d03fa5a53d73).
The linked repository's [validation description](https://github.com/inigodelportillo/ITU-Rpy/blob/f739993c4b6d34076de22249ef53d03fa5a53d73/docs/validation.rst)
attributes its examples to ITU validation workbook revision 5.1. That historical
workbook URL returned HTTP 404 during this implementation; these are therefore
**repository-hosted validation examples**, not a newly authenticated workbook.
Pinned data used:

- [P.838-3 specific-attenuation CSV](https://github.com/inigodelportillo/ITU-Rpy/blob/f739993c4b6d34076de22249ef53d03fa5a53d73/test/test_data/838/ITURP838-3_rain_specific_attenuation.csv):
  four literal H/V examples at 14.25 and 29 GHz, tolerance `1e-8 dB/km`.
- [P.618-14 rain CSV](https://github.com/inigodelportillo/ITU-Rpy/blob/f739993c4b6d34076de22249ef53d03fa5a53d73/test/test_data/618/ITURP618-14_A_rain.csv)
  plus [P.839-4 rain-height CSV](https://github.com/inigodelportillo/ITU-Rpy/blob/f739993c4b6d34076de22249ef53d03fa5a53d73/test/test_data/839/ITURP839-4_rain_height.csv):
  16 literal quantiles for four paths, including tropical elevations below/above
  25°, at 0.001, 0.01, 0.1, and 1 percent. All inputs are explicit in the test;
  `1e-6 dB` tolerance accommodates rounded climate inputs and expected results.

Additional reference outputs were obtained by executing only the pinned
[`itu838.py`](https://github.com/inigodelportillo/ITU-Rpy/blob/f739993c4b6d34076de22249ef53d03fa5a53d73/itur/models/itu838.py)
and [`itu618.py`](https://github.com/inigodelportillo/ITU-Rpy/blob/f739993c4b6d34076de22249ef53d03fa5a53d73/itur/models/itu618.py)
numerical routines offline with NumPy 2.4.2. Unit wrappers were removed and the
rain-height lookup replaced with explicit scalar inputs; numeric equations were
unchanged. This checks an independent implementation of the equations, **not**
the complete ITU-Rpy package, Astropy integration, or geographic interpolation.
Those routines agreed with 56 of the repository's rain CSV rows having matching
rain-height CSV inputs to within `4.5e-8 dB`. The added literal cases cover H/V/
circular polarization from 1 to 1000 GHz and five complete rain paths, each at
0.001, 0.01, 0.1, 1, and 5 percent (`1e-10 dB` tolerance):

| Frequency GHz | Elevation ° | Latitude ° | Station/rain height m | R0.01 mm/h | Tilt ° | A0.01 dB | A0.1 dB |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 20 | 2 | 10 | 100 / 5000 | 80 | 45 | 171.650285 | 87.727787 |
| 20 | 5 | 10 | 100 / 5000 | 80 | 45 | 116.597759 | 66.704129 |
| 20 | 30 | -10 | 100 / 5000 | 80 | 90 | 40.183986 | 17.772332 |
| 55 | 90 | 36 | 0 / 4000 | 30 | 0 | 68.797123 | 28.116281 |
| 1 | 10 | 45 | -400 / 2500 | 10 | 0 | 0.002464 | 0.000349 |

One reference discrepancy is intentional: the pinned ITU-Rpy step-10 code uses
`elevation > 25°`; P.618-14 specifies `>= 25°`. This crate follows P.618-14.
A separate formula check at exactly 25° uses the reference A0.01 = 46.863003643322216
and directly evaluates equation 8 with beta = 0.13, giving A0.1 =
20.62467194392991 dB (20 GHz, latitude 10°, 100/5000 m, R0.01=80, circular).
Zero rain and station at/above rain height are analytic zero cases. No field
validation or independent low-elevation measurement data is claimed.

### Domains, troubleshooting, and repeatable verification

All public inputs must be finite. The scalar P.838 helper accepts elevations
0–90°; `RainPath` requires 0 < elevation <= 90° and latitude -90–90°. Rain height
and rate must be nonnegative; station height may be below sea level. Tilt is
reduced modulo 180° before trigonometry, so large finite angles cannot overflow
angle conversion. Positive frequencies outside the implemented band and positive
percentages outside 0.001–5 return `UnsupportedRegime`; malformed physical values
return `InvalidInput`. Validation runs even for physical-zero cases. Nonfinite
calculations and unusable positive intermediates return `NumericalFailure`, never
a NaN/infinite attenuation. Extreme inputs are not silently clamped into a forecast.

For future edits, first re-check the specific recommendation revision and the
unit/degree conventions, then compare literal public outputs against the pinned
CSVs and numerical routines above. Preserve their distinction from field data.
Run `CARGO_NET_OFFLINE=true just fmt-check lint test doc-check` and
`git diff --check`. Do not refresh expected values from this crate's own output.
If an independent source disagrees, resolve the equation/branch and record the
reason rather than relaxing tolerances. No new tool installation or automatic
map download is part of this procedure.

Annual quantiles are not stochastic time series, fade durations, worst-month
statistics, or availability during moving-satellite passes. Do not multiply
this quantile by a path length or sum separately exceeded weather quantiles as
if they were simultaneous observations. Climate uncertainty and year-to-year
variation remain; site measurements and operational requirements must justify
using this planning model, especially near the horizon.

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
