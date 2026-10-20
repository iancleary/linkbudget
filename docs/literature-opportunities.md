# Literature-backed improvements for linkbudget

This is the original research shortlist, not an API reference. Implemented slices
and current verification status are tracked in [production model workflows](production-models.md).
The proposals below describe the research-stage gaps and acceptance goals. It compares
published papers and engineering standards with the current crate. The priority
is better link-closure decisions, not adding a modem simulator or duplicating
`gainlineup`, `touchstone`, or `rfconversions`.

## Recommended order

Effort is a relative engineering judgment, not a delivery estimate. It includes
reference-data verification and documentation, not just implementing equations.

| Priority | Opportunity | User question improved | Relative effort |
| --- | --- | --- | --- |
| 1 | Traceable DVB-S2 performance profiles | Which waveform actually closes this link? | Medium |
| 2 | Coupled atmospheric attenuation and sky noise | Does weather hurt both signal and receiver noise? | Small–medium |
| 3 | Rain-fade availability from ITU-R models | What fade margin is needed for an annual availability target? | Medium–large |
| 4 | Uncertainty propagation with sampling-error reporting | How confident should I be in this margin distribution? | Medium |
| 5 | Accurate Gaussian tails and log-BER | Can I trust very small BER values and their inverses? | Small–medium |
| 6 | Finite-blocklength planning estimates | How far below Shannon capacity should short packets operate? | Medium |

The strongest first project is **a small set of sourced DVB-S2 profiles**, not a
new universal FEC abstraction. Atmospheric noise is a good independent, smaller
project. Full weather availability should follow only after its input-data and
validation requirements are understood.

## 1. Replace implied DVB-S2 accuracy with sourced performance profiles

**Publication:** [ETSI EN 302 307-1 V1.4.1 (2014-11)][dvbs2], section 6,
Table 13, printed/physical page 36.

### Why this applies

[`src/coding.rs`](../src/coding.rs) assumes a constant horizontal shift of an
uncoded BER curve for each FEC family. Its DVB-S2 16-APSK and 32-APSK presets
actually use `Mqam(16)` and `Mqam(32)`. The latter is also outside the square-QAM
formula's derivation. These limitations are already explained in
[the communications guide](physics-communications.md#coding-gain-is-an-assumed-curve-shift).

The standard supplies a much better production input: a named mode, an error
criterion, and an ideal required **Es/N0**, rather than a universal coding gain.
Table 13 specifies normal 64,800-bit FECFRAMEs and a quasi-error-free packet error
ratio of 10^-7. The accompanying conditions include 50 fixed-point LDPC decoding
iterations, perfect synchronization, no phase noise, and an AWGN channel.

Selected published values:

| Mode | Ideal required Es/N0, dB | System information bits per symbol, no pilots |
| --- | ---: | ---: |
| QPSK 1/2 | 1.00 | 0.988858 |
| QPSK 3/4 | 4.03 | 1.487473 |
| 8PSK 2/3 | 6.62 | 1.980636 |
| 16APSK 3/4 | 10.21 | 2.966728 |
| 32APSK 5/6 | 14.28 | 4.119540 |

**Smallest useful contract:** a performance profile records mode identity,
frame length, error metric and target, Es/N0 threshold, efficiency definition,
channel/decoder assumptions, and source revision. Calculate
`Es/N0_dB = C/N0_dBHz - 10*log10(symbol_rate_sps)`, then subtract the sourced
threshold and a separately named implementation allowance to obtain margin.
Do not accept an arbitrary BER target when the source only establishes one PER
threshold. Explicitly distinguish APSK from QAM without promising an analytical
APSK BER curve.

For example, among just QPSK 1/2 and 3/4, an available Es/N0 of 4.5 dB and a
0.5 dB implementation allowance admits 1/2 but not 3/4: their margins are
3.00 dB and -0.03 dB. This is a simple, useful mode selector without implementing
a decoder. A stateless selector is enough initially; feedback delay and
hysteresis belong in a later operational ACM controller if one is needed.

**Throughput accounting:** the table's efficiency is per symbol rate, not per
occupied Hz. For raised-cosine allocation, first use
`Rs = occupied_bandwidth_hz / (1 + rolloff)`. Account for pilots and application
framing separately; do not treat the table as net application throughput.
[`LinkBudget`](../src/budget.rs) currently assumes bandwidth equals symbol rate
in its modulation-aware methods, so the new workflow needs explicit symbol rate.

**Evidence needed to ship:** check every adopted row against the source table;
exercise an end-to-end C/N0 → Es/N0 → margin → mode → rate workflow with published
thresholds as independent expected values. Reject requests for unsupported
frame lengths or error criteria instead of extrapolating. An adoption plan must
also address the existing misleading APSK preset names and public compatibility.

## 2. Couple atmospheric attenuation to receiver sky noise

**Publication:** [ITU-R P.618-14 (2023)][p618], section 3, equations (69)–(70),
printed page 24, physical PDF page 26.

### Why this applies

[`LinkBudget::frequency_dependent_loss`](../src/budget.rs) only reduces received
power. The explicit [`ReceiverNoise`](../src/receiver.rs) model can already
represent a changing source temperature, but nothing couples it to propagation.
The standard notes that atmospheric emission noise can be especially important
for low-noise front ends.

Its simplified sky-temperature estimate is:

```text
transmission = 10^(-atmospheric_attenuation_db / 10)
T_sky_k = T_mean_radiating_k * (1 - transmission) + 2.7 * transmission
```

**Smallest useful contract:** a pure helper returning sky brightness temperature
from atmospheric attenuation and mean radiating temperature. An example then
combines that with separately accounted antenna/spillover contributions and
receiver-added noise at a common reference plane. Leave `ReceiverNoise`'s
existing system-vs-source distinction intact.

**Important limits:** the attenuation excludes scintillation fading. Do not
feed free-space loss or arbitrary implementation/pointing losses into this
thermal-emission equation. Sky brightness is not automatically the antenna's
complete input noise temperature; antenna-pattern weighting and other sources
still matter. Never add the result to a supplied total system temperature that
already includes the same sky contribution.

**Evidence needed to ship:** verify the zero-attenuation limit (2.7 K), the
large-attenuation limit (mean radiating temperature), and a real receiver workflow
where atmospheric attenuation changes both received power and input noise. The
helper alone does not establish a full antenna-noise model.

## 3. Turn a hand-entered rain loss into an availability calculation

**Publications:** [ITU-R P.838-3 (2005)][p838], equations (1)–(5), and
[ITU-R P.618-14 (2023)][p618], section 2.2.1.1, steps 1–10.

### Why this applies

The README's Ka-band example inserts 3 dB of extra loss. That is a useful input,
but it cannot answer whether a particular site meets 99.9% annual availability.
The ITU methods connect rain statistics, geometry, polarization, and frequency
to attenuation exceeded for a specified percentage of an average year.

**Smallest useful contract, delivered in stages:**

1. Implement P.838's specific rain attenuation `gamma_R = k * R^alpha`, with
   frequency and polarization dependence. Its output is dB/km, **not** total
   link attenuation. Rain rate is mm/h; the standard uses GHz internally.
2. Implement P.618's effective slant-path calculation and exceedance scaling,
   accepting caller-supplied station latitude/height, elevation, rain height,
   frequency, polarization, and `R_0.01` (the one-minute rain rate exceeded for
   0.01% of an average year). Keep the initial API offline; no automatic map
   downloads or licensed/climatological data bundling.
3. Compare the resulting attenuation with available fade margin. Label the
   result as rain-only unless other propagation effects are also modeled.

P.838 covers coefficient frequencies from 1 to 1,000 GHz. That **does not** extend
P.618 section 2.2.1.1 beyond its stated range up to 55 GHz. Its exceedance scaling
covers 0.001%–5% of an average year. A 99.9% availability target corresponds to
`exceedance_percent = 0.1`, not a probability value silently passed as a percent.

**Important limits:** annual attenuation quantiles are not individual storm
forecasts, outage durations, or LEO-pass availability. Varying geometry and
weather correlation need additional modeling. Do not sum equal-percentile
rain/cloud/scintillation losses as if simultaneous exceedance were guaranteed;
P.618 section 2.5 provides a combined-effects method with its own assumptions.

**Evidence needed to ship:** compare fixed, fully specified cases with ITU-R
validation examples or an independently validated implementation using the same
recommendation revisions and climate inputs. No such numerical cross-validation
was performed for this research. Pin sources and review reuse terms before
shipping coefficient tables or geographic datasets.

## 4. Make Monte Carlo results honest about uncertainty

**Publication:** [JCGM 101:2008][jcgm], especially sections 6.1, 7.4, and 7.9.
It describes propagation of input distributions through a model, including
joint distributions and adaptive numerical stabilization.

### Why this applies

The existing [Monte Carlo example](../examples/montecarlo-cn0-target/src/main.rs)
samples independent uniform inputs, runs 50,000 trials, and writes percentiles
and a probability of meeting its target. Those outputs do not quantify sampling
error or establish that the input distributions represent a deployed system.

**Smallest useful contract:** improve the existing example, not a second Monte
Carlo engine. Record input-distribution assumptions, seed, count, and energy/rate
conventions; report margin quantiles and a binomial interval for the estimated
failure probability. Add bounded convergence checks if repeated use justifies
adaptive sampling. The JCGM adaptive procedure concerns estimates, standard
uncertainties, and coverage-interval endpoints; rare failure probabilities need
an appropriate separate stopping/interval rule.

There are three different uncertainties to keep visible:

- physical variability represented by the input distributions;
- numerical sampling error from a finite trial count;
- model/data inadequacy, which more trials do not fix.

A concrete limit: **zero failures in 50,000 independent trials still permits a
failure probability of about 5.99e-5 at a one-sided 95% binomial confidence
level**, since `q_upper = 1 - 0.05^(1/N)`. This is conditional on the sampling
model; it is not proof of actual annual availability. A JCGM coverage interval
for margin is also not the same thing as a confidence interval for estimated
failure probability.

Before extending the example, make its current assumption `Rb/Bn = 1 bit/s/Hz`
explicit in the samples or calculate Eb/N0 using an explicit information rate.
Its present target equates SNR with Eb/N0 while randomizing bandwidth, and it
has an arbitrary fallback threshold if BER inversion fails. Neither convention
should silently become a production probability model.

**Evidence needed to ship:** use an end-to-end reference case with a known
analytical output distribution, then a realistic link scenario. Check coverage
and sampling behavior over independent seeds, not just reproducibility of one
seed. Support correlations when justified by data; do not invent them. Keep
sampling infrastructure with the existing example/owning Monte Carlo library.

## 5. Improve Gaussian-tail numerics without overstating the impact

**Publication:** W. J. Cody (1969), [“Rational Chebyshev approximations for the
error function”][cody], *Mathematics of Computation* 23, 631–638. The inspected
[Netlib CALERF implementation][calerf] explicitly identifies this method and
provides `erf`, `erfc`, and scaled `erfcx` entry points. The paper's DOI metadata
was verified; its publisher PDF was unavailable (HTTP 403), so the implementation
was the accessible technical source.

### Why this applies

[`src/ber.rs`](../src/ber.rs) uses an Abramowitz–Stegun approximation with an
absolute-error claim. Low-BER users care about relative tail error and underflow,
not just absolute error. Candidate improvements are a vetted piecewise rational
`erfc`, scaled-tail evaluation, and a separate log-BER/inversion path.

A disposable Rust consumer of the **actual public crate API**, compared with
Python's platform `math.erfc`, produced:

| Argument x | Relative error of current erfc(x) |
| --- | ---: |
| 3 | 0.0663% |
| 5 | 0.6737% |
| 10 | 4.4039% |
| 20 | 12.3447% |

This is not catastrophic link-margin error: QPSK inverse thresholds differed
from a `statistics.NormalDist` reference by about **0.00031 dB at BER 1e-5** and
**0.00113 dB at BER 1e-12**. That makes it a numerical-quality improvement,
not a reason to promise materially more accurate hardware predictions.

**Smallest useful contract:** document and verify a supported relative-accuracy
range; prefer a license-compatible, maintained implementation that works with
the crate's Rust minimum version over casually copying old coefficients. If
users need probabilities beyond ordinary floating-point range, expose log-BER
explicitly rather than silently returning zero. This does not improve the
physical validity of heuristic FEC or unsupported constellations.

**Evidence needed to ship:** independent high-precision reference values across
piecewise boundaries and the supported tail range, plus forward/inverse threshold
workflows. The platform comparison above is exploratory evidence, not a certified
accuracy bound or a high-precision acceptance dataset.

## 6. Add finite-blocklength estimates alongside, not instead of, Shannon capacity

**Publication:** Y. Polyanskiy, H. V. Poor, and S. Verdú (2010),
[“Channel Coding Rate in the Finite Blocklength Regime”][ppv],
*IEEE Transactions on Information Theory* 56(5), 2307–2359.
The inspected [author-hosted manuscript][ppv-pdf] gives the normal approximation
and its comparison with non-asymptotic bounds in section IV, particularly
Theorem 54 and equations (291)–(298). Its equation numbers refer to that manuscript.

### Why this applies

[`LinkBudget::phy_rate`](../src/budget.rs) correctly labels its result as ideal
AWGN capacity, but capacity does not express a finite-packet reliability or
latency requirement. A separate planning estimate could make that gap visible.

For a complex AWGN channel with SNR `rho`, blocklength `n` complex channel uses,
and target block error probability `epsilon`, the second-order approximation is:

```text
C = log2(1 + rho)                          # bits per complex channel use
V = (1 - (1 + rho)^(-2)) * log2(e)^2
R_approx = C - sqrt(V/n) * Q_inverse(epsilon)
```

This is the complex-channel normalization of the paper's real-AWGN expression;
real-channel capacity and dispersion have factors of one half. Fix that
convention in any API. The manuscript also discusses a logarithmic correction;
results must identify which approximation order is used.

For illustration, at 0 dB SNR, `n = 1024`, and `epsilon = 1e-3`, the second-order
formula gives about **0.8793 bits/complex use**, versus asymptotic capacity of
1 bit/complex use. This number was calculated from the formula, not measured
from a modem or certified as a finite-length bound.

**Smallest useful contract:** a separate AWGN finite-blocklength planning helper
with explicit blocklength units and block-error criterion. Keep it distinct from
DVB-S2 profile predictions and `phy_rate()`. Convert channel uses to elapsed time
only after specifying the signaling/bandwidth convention.

**Important limits:** the normal approximation is not an achievability guarantee,
a rigorous upper bound, or the performance of a particular decoder. A Gaussian
input model is not a finite-constellation modem model. Reject invalid inputs and
flag regimes where the estimate is unhelpful; do not silently clamp a negative
rate and present it as a validated result.

**Evidence needed to ship:** reproduce selected published AWGN comparisons with
matching normalization and approximation order. Short-block/extreme-error regimes
need additional bounds or an explicit unsupported-domain policy.

## Research verification and boundaries

- Inspected the crate exports, link budget, BER, FEC, and Monte Carlo example,
  along with existing derivations. No production behavior was changed.
- Retrieved and extracted the ETSI, ITU-R, JCGM, and author-hosted manuscripts;
  checked the ITU listing pages for the cited P.618 and P.838 revisions.
- Ran an offline disposable Cargo consumer for the numeric BER observations.
  Sampled `erfc(x)` at x = 0 through 26 in increments of 0.1. Python standard
  library references used `math.erfc(x)` and
  `10*log10((-NormalDist().inv_cdf(p))**2 / 2)` for the QPSK threshold.
- The finite-blocklength illustration and zero-failure confidence limit are
  calculations under the stated models, not measured software improvements.
- Proposed acceptance workflows above have **not** been run for unimplemented
  features. Standards data reuse and implementation licensing remain adoption
  gates. No full decoder, climate-data service, or propagation simulator is
  proposed as the first increment.

[dvbs2]: https://www.etsi.org/deliver/etsi_en/302300_302399/30230701/01.04.01_60/en_30230701v010401p.pdf
[p618]: https://www.itu.int/dms_pubrec/itu-r/rec/p/R-REC-P.618-14-202308-I!!PDF-E.pdf
[p838]: https://www.itu.int/dms_pubrec/itu-r/rec/p/R-REC-P.838-3-200503-I!!PDF-E.pdf
[jcgm]: https://www.bipm.org/documents/20126/2071204/JCGM_101_2008_E.pdf
[cody]: https://doi.org/10.1090/S0025-5718-1969-0247736-4
[calerf]: https://netlib.org/specfun/erf
[ppv]: https://doi.org/10.1109/TIT.2010.2043769
[ppv-pdf]: https://people.lids.mit.edu/yp/homepage/data/finite_block.pdf
