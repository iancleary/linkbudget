# Illustrative receiver Eb/N0 margin uncertainty

This standalone crate propagates independent input distributions through the real
`Receiver` and energy APIs using the existing `MonteCarloEngine`. It is not a
second simulation engine, a weather model, or an availability forecast.

## Model and output contract

Each of 50,000 trials (seed 99) draws independent uniforms:

| Input | Distribution |
| --- | --- |
| Source temperature | 250–300 K |
| Receiver noise figure (290 K reference) | 1.2–3.5 dB |
| Equivalent noise bandwidth Bn | 5–25 MHz |
| Captured receiver input power | -92 to -82 dBm |

These are illustrative choices, not measured bounds or calibrated probabilities.
Signal and noise refer to the same input plane. Flat thermal noise and complete
signal capture are assumed; receiver gain is not added to already-received input
power. The model has no FEC or implementation allowance. It assumes coherent,
Gray-coded QPSK in AWGN, without synchronization or distortion penalties.

Information rate Rb is explicitly **10 Mbps**, not inferred from Bn. Each sample
builds a receiver, computes SNR, then `C/N0 = SNR + 10 log10(Bn)` and
`Eb/N0 = C/N0 - 10 log10(Rb)`. Margin is actual Eb/N0 minus the crate's calculated
required Eb/N0 for BER 1e-6. Threshold failure stops the program; there is no
fallback. For fixed signal power and temperature, Bn cancels from Eb/N0. This is
intentional under the complete-capture assumption, not a filter-clipping model.

The CSV is sorted, has header `eb_no_margin_db`, and retains full floating-point
precision for downstream checks. The summary records inputs, seed, N, Rb, BER,
threshold, mean, and p05/p50/p95 (linear interpolation at rank `p*(N-1)`). These
quantiles describe the modeled **margin spread**, not confidence limits on its
estimated quantiles.

Failures are strictly `margin < 0`; equality meets the target. `failures/N` is the
point estimate. The two-sided 95% Wilson score interval uses fixed
`z=1.959963984540054`. If there are zero failures, a separate **one-sided exact
95% upper bound** is `1 - 0.05^(1/N)`, evaluated as
`-expm1(log(0.05)/N)`. At N=50,000 it is `5.99128506245549e-5`, not zero. The
Wilson interval and exact bound have different sidedness and construction; they
must not be labeled interchangeably. Both concern binomial sampling uncertainty
conditional on the input model, not uncertainty in that model's physical truth.

## Run and verify

Prerequisite: **Rust 1.80 or newer** for this optional standalone example. Its
existing locked `rayon 1.11.0` and `rayon-core 1.13.0` dependencies declare Rust
1.80; the source's `std::io::Error::other` is available since Rust 1.74. This is a
dependency/source-derived minimum, not a Rust 1.80 compiler test: verification
used the installed Rust 1.99.0 compiler only. The root library remains Rust 1.70.

From the repository root, enter this directory **before running**; the Rust
program writes relative to the working directory, while the Python consumers
read beside their scripts:

```bash
cd examples/montecarlo-cn0-target
cargo run --offline --locked
uv run --offline --no-project python verify_output.py
cargo clippy --offline --all-targets -- -D warnings
cargo test --offline
cargo fmt --all -- --check
```

The stdlib-only verifier independently recomputes counts, quantiles, mean, and
score-test quadratic roots from the actual CSV and compares the summary. It also
checks the zero-failure bound and compares the threshold with Python's `erfc`
inversion (1e-10 dB tolerance checks the accurate Gaussian-tail kernel).
The public receiver workflow test uses SI thermal noise at 340 K and -90 dBm:
C/N0 = 83.28437800279511 dB-Hz, Eb/N0 = 13.284378002795108 dB at 10 Mbps,
and ideal QPSK margin approximately 2.754546303 dB. It checks several noise
bandwidths and doubling the information rate against independent analytic values.

Generated files are ignored locally:

- `output/receiver_ebno_margin_samples.csv`
- `output/receiver_ebno_margin_summary.txt`
- `output/receiver_ebno_margin_plots.png` (optional)

If the declared Python plotting dependencies are already available, run
`uv run --offline plot_cn0_margin.py` from this directory. No plotting dependency
installation is needed for the numeric verification. The tracked
`output/receiver_snr_margin_plots.png` is a **historical artifact of the incorrect
SNR-versus-Eb/N0 comparison**. It was not regenerated and does not depict the
current output. Consumers must use the new CSV name/header and new plot name.
If a file is missing, check the working directory and rerun the Rust command;
if a plotting import is unavailable, skip plotting rather than treating a
historical image as verification.

## Interpretation and limits

The seeded run produced 4,038 failures, p=0.08076, Wilson interval
[0.078403854422, 0.083180560357], and p05/p50/p95 margins of
-0.407177696 / 4.267591808 / 8.880563465 dB. The QPSK threshold is
10.529831699571446 dB. Replacing the former Gaussian-tail approximation
increased margins by about 0.000454 dB and removed one observed failure;
inputs and seed are unchanged. These are reproducibility evidence
for this example and lockfile, not a precision or coverage proof. No repeated-run
statistical coverage study was performed. The zero-failure formula is checked
independently, but this default scenario has nonzero failures.

The fixed bounded trial count offers no adaptive convergence guarantee, especially
for rare tails. Input uncertainty (distribution selection and limits), omitted
correlations, and model discrepancy remain outside the binomial interval. No
field validation, annual availability, fade duration, or climatic interpretation
is supported. See the [production model guide](../../docs/production-models.md#monte-carlo-conditional-uncertainty-not-availability)
for sources and uncertainty categories.
