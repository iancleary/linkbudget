# From received power to bit errors

This chapter derives the calculations in `energy`, `modulation`, `ber`, `coding`,
and `sensitivity`. Start with [RF power and noise](physics-rf-link.md) for the
received carrier power and noise density. The [documentation index](README.md)
connects the other models. These derivations explain the implemented models;
they do not certify a particular modem or decoder.

## Keep the quantities separate

Refer signal power and noise density to the same receiver input plane. Include
receiver noise as an equivalent input noise density at that plane. A power gain
applied to both quantities leaves their ratio unchanged.

| Symbol | Meaning | Linear unit |
|---|---|---|
| $C$ | Average received signal power | W |
| $N_0$ | Noise density in the convention $N=N_0B_n$ | W/Hz, equivalently J |
| $B_n$ | Equivalent receiver noise bandwidth | Hz |
| $B_{occ}$ | Full RF spectral width of the chosen pulse model | Hz |
| $R_s$ | Modulation symbol rate | symbols/s |
| $R_c$ | Coded bit rate entering the modulator | bits/s |
| $R_b$ | Information bit rate before FEC | bits/s |
| $m$ | Bits per modulation symbol, $\log_2M$; called `k` in code | bits/symbol |
| $R$ | FEC rate, information bits divided by coded bits | dimensionless |
| $E_s,E_c,E_b$ | Energy per symbol, coded bit, information bit | J |
| $k_B$ | Boltzmann constant | J/K |

Use $0<R\leq1$, positive finite rates and bandwidths, and a valid constellation.
The scalar functions generally do not enforce these conditions. A numeric result
from an arbitrary `Mpsk(M)` or `Mqam(M)` is not validation of that constellation.
Ordinary binary labels require a power-of-two $M$; square QAM further requires
even $\log_2M$.

In the real Gaussian decision model below, two-sided noise density is $N_0/2$.
This is consistent with the RF convention $N=N_0B_n$; it is not an extra factor
of two in the link budget. $C/N_0$ has units Hz and is reported in dB-Hz.
Energy/noise ratios and SNR are dimensionless and are reported in dB.

## Energy accounting proves the conversion signs

In a duration $t$, the signal supplies energy $Ct$ and sends $R_st$ symbols.
Dividing gives $E_s=C/R_s$. Apply the same count to coded and information bits:

$$
R_c=mR_s,\qquad R_b=RR_c=mRR_s,
\qquad E_c=\frac{C}{R_c}=\frac{E_s}{m},
\qquad E_b=\frac{C}{R_b}=\frac{E_c}{R}.
$$

Thus a rate-half code makes energy per information bit twice the energy per
coded bit at the same transmitted power and coded bit rate. It does not create
energy. Each information bit accounts for two transmitted coded bits.

For white noise, integrating its density gives $N=N_0B_n$. Substitution proves
the complete conversion, before any logarithms:

$$
\frac{C}{N_0}=\mathrm{SNR}\,B_n,\qquad
\frac{E_b}{N_0}=\frac{C/N_0}{R_b}
=\mathrm{SNR}\frac{B_n}{mRR_s}.
$$

Taking $10\log_{10}$ of products and quotients gives every conversion in
`src/energy.rs`. In particular,

$$
(E_b/N_0)_{dB}=(E_s/N_0)_{dB}-10\log_{10}m-10\log_{10}R.
$$

Because $\log_{10}R<0$ for $R<1$, the last subtraction increases information-bit
energy. The inverse adds both terms. MathWorks independently uses this same
code-rate sign in its [energy-ratio conversion table](https://www.mathworks.com/help/comm/ref/convertsnr.html).
Its sample-based SNR also includes samples per symbol; that is not automatically
the receiver noise bandwidth used here.

For example, QPSK at $R_s=1$ Msymbol/s and $R=1/2$ carries $R_b=1$ Mbit/s.
An SNR of 10 dB in $B_n=1$ MHz gives $C/N_0=70$ dB-Hz, $E_s/N_0=10$ dB,
$E_c/N_0=6.9897$ dB, and $E_b/N_0=10$ dB. At fixed $C$ and $N_0$, reporting SNR
in 1.35 MHz instead gives 8.6967 dB. Adding $10\log_{10}(1.35\times10^6)$ still
returns 70 dB-Hz. A changed measurement bandwidth alone cannot change $E_b/N_0$.

## Pulse shape determines spectral width

Nyquist zero-ISI signaling requires the combined transmit/receive pulse to be
zero at every other symbol sampling instant. The ideal sinc pulse has zeros at
integer multiples of $T_s=1/R_s$ and spectral support from $-R_s/2$ to $R_s/2$.
That is one-sided lowpass width $R_s/2$, or full RF width $R_s$.

Raised-cosine shaping expands each edge to $(1+\alpha)R_s/2$, so the full width
is $B_{occ}=(1+\alpha)R_s$, for $0\leq\alpha\leq1$. A matched pair of
root-raised-cosine (RRC) filters produces the raised-cosine response at the
decision point. Infinite pulses are idealizations; finite filters can leave
residual ISI. See the [MathWorks pulse-shaping implementation notes](https://www.mathworks.com/help/comm/ug/raised-cosine-filtering.html).

This explains `Modulation::occupied_bandwidth`. Its name means ideal spectral
support here, not a measured 99%-power bandwidth. It applies the same expression
to every enum variant, including MSK; that call does not derive an MSK spectrum.

The ideal RRC noise bandwidth is also calculable. Normalize its frequency
response to unity at the center and integrate its squared magnitude. The square
is a raised-cosine response: its flat portion contributes $(1-\alpha)R_s$ and
its two transition portions contribute $\alpha R_s$. Therefore

$$
B_n=\frac{\int_{-\infty}^{\infty}|H_{RRC}(f)|^2\,df}{|H_{RRC}(0)|^2}=R_s.
$$

This uses full complex-baseband width, equivalent to the RF width above. It is
not a statement that every matched filter has noise bandwidth $R_s$.

For rectangular symbols of duration $T_s$, the transform is proportional to
$\operatorname{sinc}(fT_s)$. The first zeros at $f=\pm R_s$ give full
null-to-null width $2R_s$. Sidelobes extend beyond those zeros. MSK instead has
half-sinusoidal pulses with a spectral factor

$$
\frac{\cos^2(2\pi fT_b)}{(1-16f^2T_b^2)^2}.
$$

The apparent zeros at $\pm1/(4T_b)$ cancel with the denominator. The first true
zeros are $\pm3/(4T_b)$, giving width $1.5/T_b=1.5R_s$ for binary MSK. This is
the special branch in `null_bandwidth`, not a raised-cosine rule. The pulse and
spectrum appear in [KFUPM's continuous-phase modulation lecture, pages 8–9](https://faculty.kfupm.edu.sa/EE/samir/EE571/SetG%20Continous%20Phase%20Modulation.pdf).
Here $T_b$ is a transmitted bit interval; with outer FEC it is $1/R_c$.

Finally, payload efficiency is $R_b/B_{occ}=mR/(1+\alpha)$. The crate's
`spectral_efficiency` returns $mR$, and `CodedModulation::throughput_bps(B)`
returns $BmR$. Both assume the ideal $B=R_s$ allocation. Guard bands, pilots,
framing, and roll-off overhead are not included. For a raised-cosine allocation,
compute $R_s=B_{occ}/(1+\alpha)$ before multiplying by $mR$.

## A Gaussian decision gives the BPSK curve

Assume equiprobable bits, additive white Gaussian noise (AWGN), coherent carrier
and timing recovery, and no residual ISI. Represent a BPSK bit as
$s(t)=\pm\sqrt{E_b}\,p(t)$ with $\int p^2(t)dt=1$. Correlation with $p$ gives
$y=\pm\sqrt{E_b}+n$, where $n$ is Gaussian with variance $N_0/2$.

Why use that correlator? For another real template $h$, the signal output is
$\pm\sqrt{E_b}\int ph$ and noise variance is $(N_0/2)\int h^2$. Cauchy–Schwarz
gives $(\int ph)^2\leq\int p^2\int h^2$, with equality when $h$ is proportional
to $p$. Thus matching the template maximizes the decision sample's signal to
noise ratio. The time-reversed template implements that correlation as a filter.
[MIT's matched-filter lecture](https://www.ocw.mit.edu/courses/6-011-signals-systems-and-inference-spring-2018/dfe2855f36e44436678bfd3736c937fe_MIT6_011S18lec24.pdf)
describes the decision statistic and Gaussian tail.

For a positive bit, error means $n<-\sqrt{E_b}$. Standardizing by its standard
deviation proves

$$
P_b=Q\!\left(\sqrt{2E_b/N_0}\right),\qquad
Q(x)=\frac{1}{\sqrt{2\pi}}\int_x^\infty e^{-u^2/2}du
=\tfrac12\operatorname{erfc}(x/\sqrt2).
$$

The last equality follows by substituting $u=\sqrt2v$ in the integral. Gray
QPSK has two orthogonal BPSK decisions, each with energy $E_s/2=E_b$, so its bit
error probability is identical. Its symbol error probability is different.
These are exact ideal-model BER expressions before numerical approximation.

`ber` maps MSK to this same curve. That corresponds to coherent precoded MSK;
differential encoding and other detectors need different expressions. The API
has no detector or precoding selector. Compare the separate MSK cases in the
[MathWorks analytical BER reference](https://www.mathworks.com/help/comm/ug/analytical-expressions-used-in-berawgn-function-and-bit-error-rate-analysis-app.html).

## Nearest neighbors explain the higher-order approximations

For coherent $M$-PSK, a symbol lies on a circle of radius $\sqrt{E_s}$. The
perpendicular distance to either adjacent decision boundary is
$\sqrt{E_s}\sin(\pi/M)$. Divide by the projected noise standard deviation
$\sqrt{N_0/2}$, count two adjacent boundaries, and approximate one Gray-label
bit error per wrong symbol. Since $E_s=mE_b$ for the uncoded curve,

$$
P_b\approx\frac{2}{m}Q\!\left(\sqrt{2mE_b/N_0}\sin(\pi/M)\right).
$$

This is `ber_mpsk`: exact BPSK is selected for $M=2$, and $M=4$ algebraically
reduces to the QPSK bit curve. For higher orders, counting adjacent crossings
neglects overlap and more distant decisions. Use it as a low-BER approximation,
not an exact result for arbitrary SNR or labeling.

For square QAM, let $L=\sqrt M$ and let adjacent levels on each axis be $2d$
apart. Averaging the squared levels $\pm d,\pm3d,\ldots$ gives
$d^2(L^2-1)/3$ per axis; this follows from the finite sum of odd squares. Thus
$E_s=2d^2(M-1)/3$. A nearest decision boundary is distance $d$ away, so its
Gaussian tail argument is $d/\sqrt{N_0/2}=\sqrt{3E_s/((M-1)N_0)}$.

An axis has two neighboring boundaries except at its two outer levels, where
there is one. Its average is $2(L-1)/L$. Two axes and division by $m$ bits give

$$
P_b\approx\frac4m\left(1-\frac1{\sqrt M}\right)
Q\!\left(\sqrt{\frac{3mE_b/N_0}{M-1}}\right).
$$

`ber_mqam` implements this approximation, with an exact ideal QPSK branch for
$M=4$. The approximation assigns one bit error to each erroneous axis decision.
A simultaneous nearest-neighbor error on both axes contributes two bits and is
already counted. Non-nearest level decisions can have different Gray-label
Hamming weights, so exact bit-error expressions involve more terms.
Non-square QAM needs distinct axis sizes and energies; compare the separate
square and rectangular cases in the [MathWorks reference](https://www.mathworks.com/help/comm/ug/analytical-expressions-used-in-berawgn-function-and-bit-error-rate-analysis-app.html).

In particular, `Mqam(32)` evaluates the square formula with $\sqrt{32}$ even
though that is not an integer number of levels. Its output is outside this
derivation. The 16-APSK preset substitutes 16-QAM, so it does not model APSK ring
geometry; the 32-APSK preset also inherits the non-square 32-QAM limitation.

## Coding gain is an assumed curve shift

At a fixed target error rate, define $G$ as the uncoded required information-bit
$E_b/N_0$ in dB minus the coded requirement in dB. This definition proves
`required_coded = required_uncoded - G`. It does not prove a value of $G$.

`CodedModulation::ber_from_db(x)` further assumes that the entire coded curve is
the uncoded curve evaluated at $x+G$. The same constant shift is used at every
target BER. A real decoder can have a different slope, a waterfall, and an error
floor. No encoder, decoder, block length, interleaver, or iteration count is
modeled here.

| Stored family | Rate → assumed gain in dB |
|---|---|
| Convolutional K=7 | 1/2 → 5.0; 3/4 → 3.5 |
| Turbo | 1/2 → 7.5; 3/4 → 5.5 |
| LDPC | 1/2 → 8.0; 2/3 → 7.0; 3/4 → 6.5; 5/6 → 5.5; 9/10 → 5.0 |

These are heuristic constants labeled for BER near $10^{-5}$, not values
derived from a decoder or traced to a specific measured data set. The source's
general textbook and DVB-S2 references do not establish these numbers as
universal gains. `lerp_gain` linearly interpolates in rate and clamps to endpoint
gains; LDPC selects adjacent table points first. This is a numerical convention,
not a theorem that decoder performance varies linearly with rate.

`Custom` lets callers supply a gain, but still uses the same curve-shift model.
Use compatible information-bit energy definitions when calibrating it. The rate
conversion $E_b=E_c/R$ is separate accounting; do not count its dB term a second
time as measured coding gain. For example, the LDPC rate-half preset subtracts
8 dB from the roughly 9.59 dB uncoded QPSK requirement at BER $10^{-5}$, yielding
roughly 1.59 dB **in this heuristic model**.

## Sensitivity follows from energy per information bit

For source temperature $T_0=290$ K and receiver noise factor $F$, the equivalent
input density is $N_0=k_BT_0F$. If the required information-bit ratio is
$\gamma_b$, then $E_b/N_0=C/(R_bN_0)\geq\gamma_b$ requires

$$
C_{min}=k_BT_0F R_b\gamma_b L_{impl}.
$$

$L_{impl}=10^{L_{impl,dB}/10}$ is the supplied implementation-loss factor.
In dBm this becomes the expression in `sensitivity_matched_filter_dbm`:

$$
C_{min,dBm}\approx-174+NF+10\log_{10}R_b+\gamma_{b,dB}+L_{impl,dB}.
$$

The -174 dBm/Hz term is rounded from $10\log_{10}(k_B\,290/10^{-3})$, about
-173.98 dBm/Hz. It is a 290 K convention, not a temperature-independent
constant. For a different source temperature, use the input-referred noise
model in [RF power and noise](physics-rf-link.md).

The same result follows from $C_{min}=N_0B_n\,\mathrm{SNR}_{required}$ and
$\mathrm{SNR}_{required}=\gamma_bR_b/B_n$. The $B_n$ factors cancel. This is why
changing bandwidth labels cannot by itself alter ideal sensitivity. Pulse
mismatch, discarded signal energy, ISI, and interference need additional models.

The current sensitivity helper obtains $\gamma_b$ from the **uncoded** `ber`
module and ignores its `code_rate` argument. It does not apply `FecCode` or a
coded BER threshold. Consequently its payload-rate interpretation directly
describes an uncoded link ($R=1$); a nonunit rate argument does not establish a
coded-link sensitivity. For a coded calculation, use a justified information-bit
threshold in the power equation above. For raw errors in a coded stream before
decoding, energy and rate instead refer to transmitted coded bits.

For BPSK at 1 Mbit/s, NF = 3 dB, BER = $10^{-5}$, and no implementation loss,
the implemented threshold is about 9.59 dB. Sensitivity is therefore
$-174+3+60+9.59=-101.41$ dBm. Increasing the bit rate tenfold needs 10 dB more
power at the same target. No bandwidth term must be added again.

`sensitivity_bandpass_dbm` adds $10\log_{10}(1+\alpha)$. This assumes the admitted
noise increases from a reference width $R_s$ to $(1+\alpha)R_s$, while useful
decision signal and required decision SNR remain fixed. It is an explicit loss
model, not a universal penalty or a worst-case bound for non-matched receivers.
Changing the bandwidth in a consistently converted power SNR would instead
cancel as above. At $\alpha=0.35$ this implemented loss is 1.3033 dB, giving
-100.11 dBm in the example. The legacy `sensitivity_dbm` ignores roll-off and
calls the matched-filter helper. `sensitivity_from_snr_dbm` accepts a caller's
SNR threshold, which must use the same $B_n$ as its noise-floor calculation.

## Numerical agreement is not physical accuracy

`erfc` uses the finite polynomial approximation identified in the source as
Abramowitz–Stegun 7.1.26. Its roughly $1.5\times10^{-7}$ quoted error is an
absolute error, not a guarantee of small relative error in very small tails.
Negative arguments use $\operatorname{erfc}(-x)=2-\operatorname{erfc}(x)$.
The coefficients approximate a special function; they are not physical constants.

`required_eb_no_db` checks finite endpoint BER values in [-5, 50] dB and rejects
nonpositive, nonfinite, or unbracketed targets. It bisects the decreasing
implemented curve for at most 100 iterations, stopping early when relative BER
residual is below $10^{-6}$. Each bisection halves the interval; floating-point
resolution eventually limits that process. The residual measures agreement with
the implemented approximation, not with an exact Gaussian tail or a real modem.
No bracketed result validates an unsupported constellation, FEC model, or input.
Link margin is actual minus required $E_b/N_0$ on the same bit-energy basis.

Existing tests provide these useful checks, with the stated limits:

| Existing tests | Evidence supplied |
|---|---|
| `energy::tests::ec_no_chain`, `es_no_to_eb_no_qpsk_rate_half`, `full_chain_snr_to_eb_no` | Rate and energy accounting, including the FEC sign |
| `modulation::tests::occupied_bandwidth_with_rolloff`, `msk_null_bandwidth` | Implemented width conventions |
| `ber::tests::qpsk_same_as_bpsk`, `msk_same_as_bpsk` | Shared implementation curves; no detector validation |
| `ber::tests::required_eb_no_bpsk_1e_minus_5`, `required_eb_no_16qam_1e_minus_6` | Approximate known thresholds, with broad tolerances |
| `ber::tests::required_eb_no_rejects_invalid_or_unreachable_targets` | Bracket rejection and forward/inverse consistency |
| `coding::tests::coded_requires_less_eb_no`, `ldpc_interpolated_gain` | Stored shift and interpolation behavior; no decoder proof |
| `sensitivity::tests::sensitivity_matched_bpsk_1mbps`, `higher_rate_needs_more_power` | Example power and rate scaling |
| `sensitivity::tests::sensitivity_bandpass_worse_than_matched`, `sensitivity_legacy_wrapper` | Assumed roll-off penalty and legacy behavior |
