# Physical models, orbit geometry, and measurement errors

[Calculation guide](README.md) · [Source modules](../src)

These derivations explain the equations implemented by the supporting helpers.
Each result follows from a stated model. Agreement with a test confirms the
implementation at that input; it does not prove that the model fits every link.
The helpers accept plain numbers and do not validate the physical domains below.

## Constants and units

[`constants.rs`](../src/constants.rs) provides SI values. Distances are metres,
masses are kilograms, times are seconds, and frequencies are hertz.
Degrees enter the elevation helpers; trigonometric functions use radians.
Convert kilometres to metres before combining them with these constants.

The vacuum speed of light, `c = 299792458 m/s`, is exact by the SI definition of
the metre. The stored `G = 6.67430e-11 m³/(kg·s²)` is a measured central value;
its published standard uncertainty is `0.00015e-11 m³/(kg·s²)`.
[NIST SI definition](https://www.nist.gov/pml/special-publication-330/sp-330-section-2)
and [CODATA table](https://physics.nist.gov/cuu/Constants/Table/allascii.txt).

The stored Earth, Moon, Sun, and Mars masses and radii are approximate physical
inputs. A mean radius represents a sphere; it does not describe local terrain or
flattening. Do not interpret literal equality tests as measurement accuracy.
[JPL's physical parameters](https://ssd.jpl.nasa.gov/planets/phys_par.html)
distinguish mean and equatorial radii and state uncertainties for body properties.

## Circular orbital speed and period

[`orbits::calculate_standard_gravitational_parameter`](../src/orbits/mod.rs)
forms `μ = GM`, whose units are `m³/s²`. The speed and period functions in
[`orbits::circular`](../src/orbits/circular.rs) require `M > 0` and `r > 0`.
Here `r` is distance from the central body's centre: `r = R + h`, not altitude h.
Use an orbit outside the body, with negligible satellite mass, a spherical
gravity field, and no drag, thrust, or perturbations.

Newtonian gravity supplies the inward force required for circular motion.
Cancel the satellite mass m from gravity and centripetal force:

```text
GMm/r² = mv²/r
v² = GM/r = μ/r
v = sqrt(μ/r)
T = circumference / speed = 2πr/v = 2π sqrt(r³/μ)
```

The units check: `μ/r` has units `m²/s²`, and `r³/μ` has units `s²`.
The satellite still accelerates because its velocity direction changes.
[NASA's circular-motion lesson](https://imagine.gsfc.nasa.gov/features/yba/CygX1_mass/gravity/circular_motion.html)
explains this force balance and the circumference/period relation.

Using the crate's `R = 6371000 m`, `M = 5.972e24 kg`, and `h = 1000000 m`:

```text
r = 7371000 m
μ = 3.98589196e14 m³/s²
v = 7353.592433 m/s
T = 6298.058986 s = 104.967650 min
```

These numbers reproduce the stored-constant model, not an ephemeris solution.
For example, [JPL's DE440 parameters](https://ssd.jpl.nasa.gov/astro_par.html)
give Earth's GM directly; it differs from the product of the rounded mass and G
used here. An eccentric orbit also has varying speed, so this circular-speed
formula cannot describe the entire orbit.

## Slant range from a spherical surface

[`SlantRange::calculate`](../src/orbits/slant_range.rs) returns the straight-line
station-to-satellite distance d. Assume a station on a sphere of radius `R > 0`,
satellite altitude `h >= 0`, and elevation `0° <= e <= 90°` above the local horizon.
It does not predict elevation, bend the ray in the atmosphere, or model terrain.

Choose a cross-section through the centre, station, and ray. Put the centre at
`(0,0)`, the station at `(R,0)`, and the satellite at
`(R + d sin e, d cos e)`. The satellite lies on the circle of radius `r = R+h`.
This coordinate construction gives the quadratic directly:

```text
(R + d sin e)² + (d cos e)² = (R+h)²
d² + 2R sin(e)d + R² - (R+h)² = 0
d = -R sin(e) + sqrt((R+h)² - R² cos²(e))
  = R [sqrt(((R+h)/R)² - cos²(e)) - sin(e)]
```

The last line is the implementation. It selects the nonnegative, forward-ray
root. The other root points backwards when `h > 0` in this visible-link domain.
The derivation needs only a circle and a straight ray; it does not need a
particular orbit. At the degenerate case `h=0`, the forward distance is zero.

Two limits provide useful physical checks:

```text
zenith:  e = 90° → d = (R+h) - R = h
horizon: e =  0° → d = sqrt((R+h)² - R²) = sqrt(2Rh + h²)
```

For `R = 6371000 m` and `h = 1000000 m`, the range is `1000000 m` at zenith,
`1551086.307581 m` at 35°, and `3707020.366818 m` at the horizon.
The horizon distance is finite because the surface curves away from the ray.
For extremely small `h/R`, subtraction of similar terms can lose floating-point
precision; the geometric derivation alone does not guarantee numerical accuracy.

## Doppler shift and the meaning of radial velocity

[`doppler_shift_hz`](../src/doppler.rs) takes closing velocity `v_closing`,
positive when separation decreases. If d is station-to-spacecraft range and
`u` is a unit vector from station to spacecraft, then
`d_dot = (v_spacecraft - v_station) · u` and `v_closing = -d_dot`.
Use velocities in the same reference frame. Total orbital speed is not generally
the projection along this line of sight.

To first order in speed divided by c, propagation delay gives received phase
`φ(t) ≈ 2πf[t-d(t)/c]`. Differentiate phase and divide by `2π`:

```text
f_received ≈ f(1 - d_dot/c) = f(1 + v_closing/c)
Δf = f_received - f ≈ f v_closing/c
```

Thus approach gives a positive shift and recession a negative one, as described
in [NASA's Doppler explanation](https://science.nasa.gov/learn/basics-of-space-flight/chapter6-4/).
The product `Hz × (m/s)/(m/s)` remains in hertz. Use `f > 0` and relative speeds
much smaller than c. This is a one-way approximation: it omits relativistic
time dilation, gravitational frequency shifts, and two-way transponder factors.

At `f = 12e9 Hz` and `v_closing = 7600 m/s`, the code gives
`Δf = 304210.454821 Hz` and `f_received = 12000304210.454821 Hz`.
Changing the velocity sign changes the shift sign. This example specifies a
radial velocity; it does not infer one from a 7600 m/s orbital speed.

`max_radial_velocity_circular` returns `v cos(e)`. It projects a velocity
assumed parallel to the station's local horizon onto the sight line, with its
horizontal direction chosen toward the satellite. For nonnegative speed and
`0°..90°`, its result is a magnitude; the caller must supply approach/recede sign.
It is an approximation, despite the function name.

For comparison, the exact maximum projection over tangential directions at a
circular satellite orbit of radius r, with a stationary surface station, is
`v (R/r) cos(e)`. To see why, the distance from the centre to the sight line is
`R cos(e)`; divided by r, this is the sight line's tangential component at the
satellite. Multiplying by v gives that bound. At `h = 1000000 m`, the example
orbit's horizon bound is `6355.954062 m/s`, while the helper returns
`7353.592433 m/s`. Ground rotation and orbit direction need additional geometry.

## EVM as a normalized RMS error

Let s[k] be ideal complex symbols and y[k] the aligned measured symbols, with
`e[k] = y[k] - s[k]`. For K samples and nonzero reference power:

```text
P_signal = (1/K) Σ |s[k]|²
P_error  = (1/K) Σ |e[k]|²
EVM_rms  = sqrt(P_error/P_signal)
```

This is RMS error divided by RMS reference amplitude, not the average of each
symbol's relative error. Instrument normalization and compensation matter;
see [Keysight's EVM definition](https://www.keysight.com/ie/en/assets/7018-01305/application-notes/5989-3144.pdf)
and [normalization settings](https://helpfiles.keysight.com/csg/89600B/Webhelp/Subsystems/digdemod/content/dlg_digdemod_comp_evmnormref.htm).

If the only error is additive noise, `P_error = P_noise`. With matching signal
reference, filtering, and bandwidth, `SNR_linear = P_signal/P_noise`, so:

```text
EVM_rms = 1/sqrt(SNR_linear)
SNR_dB = 10 log10(1/EVM_rms²) = -20 log10(EVM_rms)
EVM_rms = 10^(-SNR_dB/20)
EVM_percent = 100 EVM_rms
```

The minus sign makes smaller error imply larger SNR. The factor 20 follows from
squaring an amplitude ratio. [`evm.rs`](../src/evm.rs) implements these equations,
not a constellation estimator. At 20 dB, linear SNR is 100 and EVM is 0.1, or
10%. Passing `10` to the fractional-EVM helper instead gives -20 dB.

EVM caused by distortion, phase error, or synchronization error describes total
error, not necessarily thermal noise. The conversion then gives an equivalent
signal/error ratio. Peak-constellation normalization also changes the ratio.
Use finite, positive EVM and linear SNR for finite physical conversions. Zero EVM gives
positive infinite SNR; negative EVM gives NaN through the logarithm.

For positive percentages, `evm_margin(measured, required)` computes
`20 log10(required/measured)`. Measured 5% against an 8% maximum gives
`4.082400 dB` and passes. The ratio cancels the factor 100; equality passes.

## Quantization noise and effective bits

An ideal N-bit uniform ADC with input span `V_pp` has step `Δ = V_pp/2^N`.
Assume no clipping and an error uniformly distributed over `[-Δ/2, Δ/2]`.
Its mean is zero by symmetry, and its variance follows by integration:

```text
P_quantization = (1/Δ) ∫[-Δ/2 to Δ/2] q² dq = Δ²/12
```

A full-scale sine has peak amplitude `V_pp/2` and mean square `V_pp²/8`.
Dividing the two mean squares cancels the voltage units:

```text
SNR_linear = (V_pp²/8) / (V_pp²/(12·2^(2N))) = (3/2) 2^(2N)
SNR_dB = 20 N log10(2) + 10 log10(3/2)
       ≈ 6.02 N + 1.76
```

This explains the coefficients in [`quantization.rs`](../src/quantization.rs).
[Analog Devices MT-229](https://www.analog.com/media/en/training-seminars/tutorials/MT-229.pdf)
derives the RMS and full-scale sine result. The implementation uses rounded
coefficients: 12 bits gives 74.0 dB; unrounded coefficients give 74.008112 dB.
The formula has no useful ADC interpretation at zero bits, although the helper
accepts zero and evaluates the expression.

For white quantization noise uncorrelated with the signal, total noise spans
`0..f_sample/2`. Filtering to bandwidth `0 < B <= f_sample/2` retains the fraction
`2B/f_sample`, giving process gain `10 log10(f_sample/(2B))` dB. Merely sampling
faster does not add this gain unless the unwanted noise is removed. Quantization
error can instead correlate with a periodic or small input and form harmonics;
the white-noise assumption then fails. See
[Analog Devices MT-001](https://www.analog.com/media/en/training-seminars/tutorials/MT-001.pdf).

Inverting the rounded formula gives `ENOB = (ratio_dB - 1.76)/6.02`.
The helper `enob_from_snr` takes SNR. Conventional ADC ENOB uses full-scale
SINAD, which includes distortion; noise-only SNR describes equivalent noise
resolution. A sine backed off by b dB loses b dB of ratio under the same noise
model. Normalize that measurement to full scale before applying the inversion
when the comparison requires it. The helper performs neither backoff nor
bandwidth correction. [Analog Devices MT-003](https://www.analog.com/media/en/training-seminars/tutorials/MT-003.pdf)
defines these measurement differences. For example, 61.96 dB maps to 10 bits.

## Existing verification

Tests live in each linked source file's `tests` module. Their coverage is bounded:

| Module | Existing test examples | What they check |
| --- | --- | --- |
| `constants` | `speed_of_light`, `gravitational_constant`, body-value tests | Stored literals; no physical uncertainty estimate |
| `orbits::circular` | `leo_earth`, `leo_earth_period`, `leo_earth_period_higher` | Speed and periods using the stored Earth constants |
| `orbits::slant_range` | `straight_above`, `horizon`, `thirty_five_degrees`, LEO/MEO/GEO cases | Geometric limits and numerical reference cases |
| `doppler` | `doppler_shift_leo_ku_band`, approach/recede tests, horizon/zenith tests | First-order arithmetic, signs, and the projection helper's endpoints |
| `evm` | `roundtrip_snr_evm`, `evm_linear_vs_db`, percentage and margin tests | Conversion consistency, percentage scale, and margin sign |
| `quantization` | `snr_8_bit`, `snr_12_bit`, `snr_16_bit`, `enob_roundtrip` | Rounded linear formula and its inverse |

These tests do not establish orbit-prediction accuracy, RF measurement validity,
or ADC performance. Select the model and inputs before relying on its output.
