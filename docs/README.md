# Calculation guides

Start with the physical model, then follow the equation into the Rust API.
These guides explain why the calculations work, the units they require, and
where the model stops describing a real receiver.

## Reading order

1. [RF power, noise, and capacity](physics-rf-link.md): follow transmitted
   power through free-space propagation, receiver noise, G/T, C/No, and
   Shannon capacity. Includes the receiver-temperature proof and a worked link.
2. [Modulation, energy, BER, and sensitivity](physics-communications.md):
   relate symbols and information bits to energy, derive the binary error
   probability, and distinguish decoding approximations from exact results.
3. [Geometry, Doppler, EVM, and quantization](physics-geometry-and-errors.md):
   derive the supporting orbit, frequency, and measurement calculations.

Each guide maps equations to implementation files and existing tests. The
[README examples](../README.md) show how to construct the public types.
The [receiver-noise decision record](adr/0001-explicit-receiver-noise.md)
explains the breaking API choice in 0.7.0.

## Conventions used throughout

| Symbol | Meaning | Unit or convention |
| --- | --- | --- |
| `C`, `N` | Received signal power and integrated noise power | W, at one receiver input plane |
| `N0` | Noise power spectral density | W/Hz; `N = N0 * Bn` |
| `Bn` | Receiver equivalent noise bandwidth | Hz |
| `Bch` | Channel bandwidth used for capacity | Hz |
| `Bocc` | Occupied waveform bandwidth | Hz; depends on pulse shape |
| `Rs`, `Rb` | Symbol rate and information bit rate | symbol/s and bit/s |
| `m`, `r` | Bits per transmitted symbol and code rate | `m = log2(M)`, `0 < r <= 1` |
| `Tsys`, `Te` | Total system and receiver-added noise temperatures | K, input-referred |
| `F`, `NF` | Noise factor and noise figure | Linear ratio and `10 log10(F)` dB |

RF `N0` here is the density that integrates over a positive-frequency RF band
to give `N0 * Bn`. In a real low-pass AWGN decision model, the two-sided
density is `N0/2`. Keep that convention when comparing matched-filter formulas.

`C/No` in API names means `C/N0`. Its linear unit is Hz; its logarithmic unit
is dB-Hz. `Eb/N0` is dimensionless. Neither quantity is a noise power.
Numerical log arguments use the indicated reference units, such as 1 W,
1 mW, 1 Hz, or 1 K.

## What a proof establishes

An equation can be exact within an ideal model and still be insufficient for
hardware prediction. Free-space loss assumes the far field. Thermal-noise
formulas assume a common reference plane. Capacity assumes an ideal AWGN
channel. Tests verify the implemented equations and selected invariants; they
do not establish the model for every antenna, waveform, or decoder.

The BER, FEC, Doppler-envelope, and sensitivity guides label approximations
explicitly. In particular, a tabulated coding gain is an input to a model,
not a result proved by the receiver-noise equations.

Most APIs use unvalidated `f64` values. Use finite, physically meaningful
inputs and the domains stated in each guide. Do not infer validation from a
successful function call.

## Repository maintenance

- [Agent operating loop](agent-operating-loop.md)
- [Release process](release.md)
