# linkbudget

RF link budget analysis for satellite and terrestrial communication systems.

[![Crates.io](https://img.shields.io/crates/v/linkbudget.svg)](https://crates.io/crates/linkbudget)

## Features

| Module              | Description                                                              |
|---------------------|--------------------------------------------------------------------------|
| **Transmitter**     | Output power, gain, EIRP (dBm/dBW)                                      |
| **Receiver**        | Gain, noise temperature, noise figure, SNR, G/T                         |
| **Path Loss**       | Free space path loss (FSPL) from frequency and distance                  |
| **Link Budget**     | End-to-end: TX → path loss → RX → SNR → Eb/No → BER → margin           |
| **PHY Rate**        | Shannon capacity from SNR and bandwidth                                  |
| **Orbits**          | Slant range, circular orbit speed/period                                 |
| **Doppler**         | Doppler shift, received frequency, max radial velocity                   |
| **Power Flux Density** | PFD (dBW/m²) and PFD per MHz for regulatory analysis                 |
| **Quantization**    | ADC/DAC quantization SNR and ENOB                                        |
| **Modulation**      | BPSK, QPSK, M-PSK, M-QAM, MSK — symbol rate, bandwidth, spectral efficiency |
| **Energy (Eb/No)**  | Eb/No, Es/No, Ec/No, C/No conversions between all energy-per-bit metrics |
| **BER**             | Theoretical BER curves (erfc/Q-function), required Eb/No, link margin    |
| **Sensitivity**     | Receiver MDS from modulation, code rate, NF, target BER                  |
| **EVM**             | Error Vector Magnitude ↔ SNR conversions and margin checking             |
| **Coding (FEC)**    | CodedModulation, FecCode enum, coding gain constants, DVB-S2 presets     |

## Link Budget — End-to-End Example

```rust
use linkbudget::{LinkBudget, PathLoss, Transmitter, Receiver, Modulation};
use linkbudget::coding;

let budget = LinkBudget {
    name: "Ka-band LEO downlink",
    bandwidth: 36e6,              // 36 MHz channel
    transmitter: Transmitter {
        output_power: 10.0,       // dBm
        gain: 35.0,               // dBi
        bandwidth: 36e6,          // Hz
    },
    receiver: Receiver {
        gain: 40.0,               // dBi
        temperature: 290.0,       // K
        noise_figure: 2.0,        // dB
        bandwidth: 36e6,          // Hz
    },
    path_loss: PathLoss {
        frequency: 20.0e9,        // 20 GHz Ka-band
        distance: 550.0e3,        // 550 km LEO
    },
    frequency_dependent_loss: Some(3.0), // rain fade margin
};

// RF metrics
println!("EIRP: {:.1} dBm", budget.transmitter.eirp_dbm());
println!("G/T: {:.1} dB/K", budget.receiver.g_over_t_db());
println!("Path Loss: {:.1} dB", budget.path_loss());
println!("C/No: {:.1} dB·Hz", budget.c_over_no());

// Uncoded performance
println!("SNR: {:.1} dB", budget.snr());
println!("Eb/No (QPSK): {:.1} dB", budget.eb_no_db(&Modulation::Qpsk));
println!("BER (QPSK uncoded): {:.2e}", budget.ber(&Modulation::Qpsk));
println!("Shannon capacity: {:.1} Mbps", budget.phy_rate().mbps());

let margin = budget.link_margin_db(&Modulation::Qpsk, 1e-5).unwrap();
println!("Link margin (uncoded, BER=1e-5): {:.1} dB", margin);

// With FEC coding — DVB-S2 QPSK rate 3/4 (LDPC)
let coded = coding::dvbs2_qpsk_r34();
println!("BER (coded): {:.2e}", budget.ber_coded(&coded));
println!("Throughput: {:.0} Mbps", budget.throughput_bps(&coded) / 1e6);
println!("Coded margin: {:.1} dB",
    budget.link_margin_coded_db(&coded, 1e-5).unwrap());
```

## Modulation & BER

```rust
use linkbudget::{Modulation, ber};

let mod_qpsk = Modulation::Qpsk;

// Symbol rate and occupied bandwidth
let rs = mod_qpsk.symbol_rate(10e6, 0.75); // 10 Mbps, rate 3/4
let bw = mod_qpsk.occupied_bandwidth(rs, 0.35); // 35% roll-off
println!("Symbol rate: {:.2} Msps", rs / 1e6);
println!("Occupied BW: {:.2} MHz", bw / 1e6);

// BER at Eb/No = 10 dB
println!("BER: {:.2e}", ber::ber_from_db(10.0, &mod_qpsk));

// Required Eb/No for BER = 1e-6
println!("Required Eb/No: {:.1} dB",
    ber::required_eb_no_db(1e-6, &mod_qpsk).unwrap());

// Link margin: actual vs required
println!("Margin: {:.1} dB",
    ber::link_margin_db(12.0, 1e-6, &mod_qpsk).unwrap());
```

## Energy-per-Bit Conversions

```rust
use linkbudget::energy;

// SNR = 20 dB in 10 MHz noise bandwidth
let c_no = energy::snr_to_c_over_no(20.0, 10e6);   // → 80 dB·Hz
let eb_no = energy::c_over_no_to_eb_over_no(c_no, 5e6); // 5 Mbps → 13 dB
println!("C/No: {:.1} dB·Hz", c_no);
println!("Eb/No: {:.1} dB", eb_no);

// Full chain: SNR → Eb/No via modulation and code rate
use linkbudget::Modulation;
let eb_no = energy::snr_to_eb_over_no(
    20.0,   // SNR (dB)
    10e6,   // noise bandwidth
    &Modulation::Qpsk,
    5e6,    // symbol rate
    0.75,   // code rate
);
```

## Coded Modulation (FEC)

```rust
use linkbudget::coding::{self, CodedModulation, FecCode};
use linkbudget::Modulation;

// DVB-S2 preset
let cm = coding::dvbs2_qpsk_r34();
println!("{}", cm); // "QPSK + LDPC (R=0.75)"
println!("η = {:.2} bits/s/Hz", cm.spectral_efficiency());
println!("Throughput in 36 MHz: {:.0} Mbps", cm.throughput_bps(36e6) / 1e6);

// Required Eb/No with coding gain
println!("Required Eb/No: {:.1} dB", cm.required_eb_no_db(1e-5).unwrap());

// Link margin at 8 dB Eb/No
println!("Margin: {:.1} dB", cm.link_margin_db(8.0, 1e-5).unwrap());

// Custom: 16-QAM + Turbo rate 1/2
let custom = CodedModulation::new(
    Modulation::Mqam(16),
    FecCode::Turbo { rate: 0.5 },
);
```

### Available DVB-S2 Presets

| Preset                  | Modulation | FEC        | η (bits/s/Hz) |
|-------------------------|------------|------------|----------------|
| `dvbs2_qpsk_r12()`      | QPSK       | LDPC R=1/2 | 1.00           |
| `dvbs2_qpsk_r34()`      | QPSK       | LDPC R=3/4 | 1.50           |
| `dvbs2_8psk_r23()`      | 8-PSK      | LDPC R=2/3 | 2.00           |
| `dvbs2_16apsk_r34()`    | 16-APSK    | LDPC R=3/4 | 3.00           |
| `dvbs2_32apsk_r56()`    | 32-APSK    | LDPC R=5/6 | 4.17           |

## Sensitivity

```rust
use linkbudget::{Modulation, sensitivity};

// Matched filter (root-raised-cosine) — ideal, α-independent
let matched = sensitivity::sensitivity_matched_filter_dbm(
    &Modulation::Qpsk, 10e6, 0.75, 3.0, 1e-6, 2.0,
).unwrap();
println!("Matched filter: {:.1} dBm", matched);

// Bandpass filter — practical, includes roll-off penalty
let bandpass = sensitivity::sensitivity_bandpass_dbm(
    &Modulation::Qpsk, 10e6, 0.75, 3.0, 1e-6, 2.0, 0.35,
).unwrap();
println!("Bandpass (α=0.35): {:.1} dBm", bandpass);

// Roll-off penalty
println!("Penalty: {:.2} dB", sensitivity::rolloff_penalty_db(0.35));
```

### Roll-Off Factor (α)

The roll-off factor α controls the excess bandwidth of raised-cosine pulse shaping:

| α    | Excess BW                      | Sensitivity Penalty |
|------|--------------------------------|---------------------|
| 0.00 | 0% (brick-wall, impractical)   | 0.00 dB             |
| 0.20 | 20% (DVB-S2)                   | 0.79 dB             |
| 0.25 | 25%                            | 0.97 dB             |
| 0.35 | 35% (legacy DVB-S)             | 1.30 dB             |
| 0.50 | 50%                            | 1.76 dB             |
| 1.00 | 100%                           | 3.01 dB             |

With a matched filter (RRC at TX + RX), noise bandwidth = symbol rate regardless of α, so sensitivity is unaffected. The penalty applies when using a simple bandpass filter set to the occupied bandwidth Rs×(1+α).

## EVM

```rust
use linkbudget::evm;

// EVM ↔ SNR
println!("EVM at 25 dB SNR: {:.1}%", evm::evm_percent_from_snr_db(25.0));
println!("SNR for 5% EVM: {:.1} dB", evm::snr_db_from_evm_percent(5.0));

// Pass/fail check with margin
let (pass, margin) = evm::evm_margin(5.0, 8.0); // measured 5%, required 8%
println!("Pass: {}, margin: {:.1} dB", pass, margin);
```

## Doppler

```rust
use linkbudget::doppler;

let shift = doppler::doppler_shift_hz(14e9, 7000.0); // 14 GHz, 7 km/s
println!("Doppler shift: {:.0} Hz", shift); // ~327 kHz

let received = doppler::doppler_received_frequency(14e9, 7000.0);
println!("Received freq: {:.6} GHz", received / 1e9);
```

## Power Flux Density

```rust
use linkbudget::{power_flux_density_dbw_per_m2, pfd_per_mhz};

let pfd = power_flux_density_dbw_per_m2(45.0, 550e3);
println!("PFD: {:.1} dBW/m²", pfd);

let pfd_mhz = pfd_per_mhz(45.0, 550e3, 36.0);
println!("PFD/MHz: {:.1} dBW/m²/MHz", pfd_mhz);
```

## Quantization

```rust
use linkbudget::{quantization_snr_db, enob_from_snr};

println!("12-bit SNR: {:.2} dB", quantization_snr_db(12)); // 74.00
println!("ENOB at 65 dB: {:.1} bits", enob_from_snr(65.0)); // 10.5
```

## CLI

```bash
linkbudget          # runs the built-in example
```

## References

- [Path Loss, Antenna Gain, and Frequency Dependence — Eric Jacobsen](https://www.dsprelated.com/showarticle/62.php)
- [Understanding Eb/No, SNR, and Power Efficiency — Eric Jacobsen](https://www.dsprelated.com/showarticle/168.php)
- [Raised-cosine filter — Wikipedia](https://en.wikipedia.org/wiki/Raised-cosine_filter)
- Proakis, J. (1995). *Digital Communications* (3rd ed.). McGraw-Hill.
- ETSI EN 302 307 — DVB-S2 (LDPC + BCH coding, modulation schemes)
