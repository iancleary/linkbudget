# Explicit receiver noise and consistent channel capacity

Status: accepted for the next minor release.

Related: [issue #81](https://github.com/iancleary/linkbudget/issues/81) and the
[Eb/No review in PR #80](https://github.com/iancleary/linkbudget/pull/80).

## Context

In 0.6.x, `Receiver.temperature` was documented as system temperature.
Noise power then multiplied `k * T * B_noise` by receiver noise factor.
G/T used the temperature without that factor. Those operations cannot both
represent the same total input-referred noise.

`LinkBudget::phy_rate()` used receiver SNR with channel bandwidth. That gives
the wrong AWGN capacity when receiver noise bandwidth differs from channel
bandwidth.

## Noise proof

Use a matched, classical thermal-noise model. All noise powers and signal
powers refer to the same receiver input plane. Noise spectra are flat over the
band of interest. Independent source and receiver noise powers add.

At the reference source temperature `T0 = 290 K`, the input noise density is
`k * T0`. Let `G` be receiver power gain and let `N_added` be receiver-added
output noise in bandwidth `B_noise`. The definition of noise factor gives:

```text
F = SNR_in / SNR_out
  = (C / (k T0 B_noise)) / (G C / (G k T0 B_noise + N_added))
  = 1 + N_added / (G k T0 B_noise)

T_e = N_added / (G k B_noise) = T0 (F - 1)
```

For any source temperature within this model, input-referred receiver noise
adds to source noise:

```text
T_sys = T_source + T0 (F - 1)
N0 = k T_sys                            [W/Hz]
N = k T_sys B_noise                     [W]
G/T = G_rx_dBi - 10 log10(T_sys / 1 K)  [dB/K]
```

`T_source * F` equals this total only when `T_source = T0` or `F = 1`.
For `T_source = 50 K` and `F = 2`, the correct total is `340 K`.
The old formula gave `100 K`, overstating C/No by
`10 log10(340/100) = 5.314789 dB`.

If a supplied temperature is already total system temperature, use it
directly. Adding receiver noise again would double count that contribution.
An equivalent receiver noise temperature alone is not total system
temperature: it still needs the source contribution.

The C/No link equation also checks consistency with G/T. With path loss `L`
in dB and the same input reference plane:

```text
C/No_dBHz = EIRP_dBW - L_dB + (G/T)_dB/K - 10 log10(k)
```

This must agree with `10 log10(C / (k T_sys))` from received power in watts.

## Capacity proof

For a flat AWGN channel with channel bandwidth `B_channel`, Shannon capacity
is:

```text
capacity = B_channel log2(1 + C / (N0 B_channel))  [bit/s]
```

Receiver SNR measured over noise bandwidth `B_noise` is `C / (N0 B_noise)`.
Thus channel SNR is measured SNR multiplied by `B_noise / B_channel`.
Equivalently, subtract `10 log10(B_channel / 1 Hz)` from C/No in dB-Hz.

At `C/No = 80 dB-Hz` and `B_channel = 20 MHz`, channel SNR is 5 in linear
units. Capacity is `20e6 * log2(6) = 51.699250 Mbit/s`.
Changing receiver noise bandwidth from 30 MHz to 60 MHz changes measured SNR,
but leaves this capacity unchanged. The old formula instead gave 42.309544
and 28.300750 Mbit/s.

This is an ideal bound. It assumes that the receiver captures all signal
power. It does not describe a filter that clips the signal, colored noise,
interference, or a practical modulation and coding limit. In particular, the
invariance is not a claim that arbitrarily narrowing a real filter is harmless.

## Decision and compatibility

Replace the ambiguous temperature and noise-figure fields with `ReceiverNoise`:

- `SystemTemperature { temperature_k }` holds total input-referred temperature.
- `SourceAndNoiseFigure { source_temperature_k, noise_figure_db }` derives the
  total using the 290 K noise-figure reference.

Use `system_noise_temperature_k()` for both noise power and G/T.
Keep `calculate_noise_floor()` as an alias for total noise power. A total
temperature cannot determine source-only noise.

This is a breaking API change for the next minor release. Existing receiver
literals must select an interpretation. The README has migration examples.
Standard 290 K source fixtures keep their old noise power. Their corrected G/T
decreases by NF in dB. The 150 K LNB and 25 K DSN examples explicitly specify
total system temperature. The 400 K earth-facing example specifies source
temperature plus receiver NF.

We rejected silently reinterpreting the old field as source temperature. Its
system-temperature documentation and cold-receiver examples made that unsafe.
We also rejected accepting both a total temperature and an independently
applied NF: that would preserve the double-counting ambiguity.

Keep the existing plain `f64` API style. Document finite, positive total
temperature and bandwidth, plus nonnegative source temperature and NF. There
is no new runtime validation or hardware noise-correlation model.

## Verification

Receiver unit tests use the SI Boltzmann constant to verify total noise power,
the 290 K reference case, a noiseless receiver, and a zero-temperature source
with nonzero receiver noise. Integration tests derive received power from
spherical spreading and effective aperture. They check G/T against the kT
link equation, both noise representations, coded and uncoded Eb/No, and
capacity with equal and unequal bandwidths. README examples are mirrored in
the integration suite.

## Sources

- [Keysight, Fundamentals of RF and Microwave Noise Figure Measurements](https://www.keysight.com/us/en/assets/7018-06808/application-notes/5952-8255.pdf):
  noise figure and equivalent input noise temperature at the 290 K reference.
- [Shannon, A Mathematical Theory of Communication](https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf):
  Theorem 17, the band-limited white Gaussian noise channel capacity theorem.
