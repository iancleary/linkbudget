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
| 5 | Gaussian-tail numerics | Independent high-precision tails/BER/inverse references; real Monte Carlo caller reverified |
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
Observed N=50,000, failures=4,038, p=0.08076, Wilson95=
[0.078403854422, 0.083180560357]; p05/p50/p95 margins=
-0.407177696 / 4.267591808 / 8.880563465 dB. The accurate Gaussian kernel
sets the QPSK threshold to 10.529831699571446 dB; replacing the former tail
approximation shifts margins upward by about 0.000454 dB and removes one
observed failure, without changing inputs or seed. The independent zero-count upper
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

## Gaussian tails: log probabilities before underflow

### Source, decision, and domain

[`ber`](../src/ber.rs) now shares a pure private Gaussian kernel rather than the
former Abramowitz–Stegun 7.1.26 approximation. The implementation is independently
written from [DLMF 7.6.1 (erf series)](https://dlmf.nist.gov/7.6.E1),
[7.11.2 (incomplete-gamma identity)](https://dlmf.nist.gov/7.11.E2), and
[8.9.2 (gamma continued fraction)](https://dlmf.nist.gov/8.9.E2). The source
TeX equations were checked against those NIST URLs. Even contraction of the
fraction gives the implemented `b_n=x*x+0.5+2*n`, `a_n=-n*(n-0.5)` recurrence.
No Numerical Recipes, Netlib, fdlibm, libm, or other implementation code was
copied, and no runtime dependency was added. The [physics guide](physics-communications.md#numerical-agreement-is-not-physical-accuracy)
explains the branch choice, normalization, and logarithmic evaluation.

The decision is to retain the existing linear API while adding checked natural-log
BER and a reusable checked inverse Q. The small-x series stops when its term is
at most machine epsilon times its sum. The positive-tail modified Lentz recurrence
stops when its multiplicative update differs from one by at most two epsilons;
both have a 10,000-iteration failure cap, not a fallback approximation. `erfc`
retains normal NaN/infinity handling and representable underflow. Checked APIs
report `NumericalFailure` for nonfinite log results or failed convergence.

Use finite Eb/No in dB and a supported binary constellation: power-of-two PSK
M>=2, square power-of-two QAM M>=4. Higher orders still use nearest-neighbor
low-BER approximations. Coherent precoded MSK remains the BPSK bit curve. New
checked APIs reject non-square `Mqam(32)`; legacy linear BER/inversion and
heuristic presets intentionally retain their old unvalidated geometry. More
accurate Gaussian tails do **not** improve the provenance of FEC gain constants.

```rust
use linkbudget::{ber, Modulation};

// Linear BER underflows, but its natural logarithm remains usable.
let modulation = Modulation::Qpsk;
assert_eq!(ber::ber_from_db(30.0, &modulation), 0.0);
let log_ber = ber::log_ber_from_db(30.0, &modulation)?;
assert!((log_ber + 1004.7198891395119).abs() < 2e-12);
let threshold_db = ber::required_eb_no_from_log_ber(-1000.0, &modulation)?;
assert!((threshold_db - 29.97946356108686).abs() < 1e-11);
let decision_sigma = ber::inverse_q_function(1e-300)?;
assert!((decision_sigma - 37.0470962993612).abs() < 3e-14);
# Ok::<(), linkbudget::ModelError>(())
```

The log inverse accepts only finite negative targets reached in the inclusive
[-5, 50] dB bracket. Legacy `required_eb_no_db` preserves its `Option` signature
and linear endpoint rejection, but now solves in log space to f64 resolution.
Inverse Q accepts the open interval (0,1); 0 and 1 are errors, not infinities.
It uses symmetry and positive-tail log bisection, supporting the least subnormal
input. Rounding near probability 1/2 limits relative accuracy of a near-zero
threshold; extreme probabilities near 1 are limited by the input's representable
complement. No finite result is promised for log-BER beyond f64's finite range.

### Independent evidence and repeatable verification

The literal references in [`tests/published_models.rs`](../tests/published_models.rs)
were calculated independently with Python stdlib `decimal` at **1,200 digits**:
Machin's `pi=16*atan(1/5)-4*atan(1/239)` using the alternating arctangent series,
then the erf Taylor series and `1-erf`. That deliberately uses cancellation with
enough precision rather than reusing the production tail continued fraction.
Inputs for erfc references are exact Decimal conversions of the f64 inputs;
BER references use exact decimal dB values. Thresholds were obtained by 65
bisections in the erfc argument and conversion to dB, not from crate output.
No high-precision package was installed or required.

The maintained, self-contained generator is
[`gaussian_references.py`](../.agents/skills/rf-literature-review/scripts/gaussian_references.py),
owned by the literature-review skill. From the checkout root:

```sh
uv run --offline --no-project python .agents/skills/rf-literature-review/scripts/gaussian_references.py
```

It repeats the calculation at **1,200 and 1,600 digits**, checks that every
rounded binary64 output is identical, and prints the references only after that
check passes. It uses no crate output, high-precision packages, network access,
or file writes. The erf and arctangent series have explicit iteration-cap errors;
bisection checks its bracket and takes the original fixed 65 argument steps.
8-PSK uses the exact radical for `sin(pi/8)`; square QAM uses Decimal logarithms
for bits per symbol. The QPSK log target is inverted directly without exponentiating
it into binary64.

The supported computation set is deliberately finite: the 17 erfc arguments listed
in the helper (0 through 27.3, including the 1.5 branch neighbors); binary
modulation at 0/10/20/30 dB, 8-PSK at 10 dB, 16-QAM at 15 dB, and 64-QAM at
20 dB; QPSK thresholds for decimal BER 1e-6/1e-100/1e-300 and log BER -1000;
and inverse Q for decimal p=.1/1e-100/1e-300 and the exact binary64 least
subnormal. Decimal probability inputs intentionally reproduce the original
reference procedure, rather than claiming exact conversion of binary64 targets.
No flags extend this set. Precision agreement is observed stability, not a proof
of correct rounding or a guarantee for arbitrary tails: cancellation consumes
precision in both the large Taylor terms and the tiny residual. Investigate any
cap, bracket, or stability error before changing literals or extending the set.

Observed on the installed Rust 1.99.0/macOS environment (no older compiler test):

| Check | Actual evidence |
| --- | --- |
| 13 erfc literals from 0 through 26, including 1.5 +/- 1e-12 | Maximum relative error `8.1e-14`, gate `1e-12` |
| Additional 26,001-point erfc grid at x=i/1000, 0<=i<=26000 | Maximum relative difference `2.02e-13` against platform Python `math.erfc`; supporting evidence, not high-precision proof |
| Subnormal erfc | x=27 gives `5.23705e-319`; x=27.2 gives two least-subnormal units; x=27.25 rounds to zero |
| Public BER/log-BER | BPSK/QPSK/MSK, 8-PSK, 16/64-QAM literal values; log QPSK at 30 dB `-1004.7198891395119` |
| QPSK inverse thresholds | BER 1e-6, 1e-100, 1e-300: `10.529831699571448`, `23.546460034850334`, `28.36478352821862` dB; gate `1e-11 dB` |
| Log target beyond exp underflow | log BER -1000 -> `29.97946356108686 dB`; gate `1e-11 dB` |
| Inverse Q | p=.1, 1e-100, 1e-300, least subnormal; absolute threshold gate `3e-14`; minimum maps to `38.467405617144344` |
| Real Monte Carlo consumer | CLI, receiver control, and CSV/report verifier pass offline; independent threshold comparison strengthened to `1e-10 dB` |

These are **measured errors, not rigorous global bounds**. Subnormal relative
error can be large; on this platform `math.erfc(27.2)` itself returned zero,
whereas the Decimal reference rounds to two subnormal units. Do not use a
platform function as an unquestioned reference at its underflow boundary.
A forward/inverse round trip alone is also insufficient evidence.

For future changes, inspect the source equations and preserve the exact bit-energy
and constellation assumptions first. Recompute independent literals with the
maintained Decimal helper above (for new points, independently establish enough
precision for cancellation and rounded stability). Keep branch-boundary,
negative/NaN/infinity, subnormal, invalid
constellation, and below-underflow log targets in the public workflows. Run
`CARGO_NET_OFFLINE=true just fmt-check lint test doc-check`, then run the real
standalone example and `verify_output.py` from its directory following its README,
and `git diff --check`. If seeded observations change, recheck the independently
computed threshold before updating reported numbers. Do not relax numerical
checks to match crate output. Root Rust MSRV stays 1.70; the optional example's
existing locked dependencies require 1.80, inferred from metadata/source only.

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
