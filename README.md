# linkbudget

RF link budget analysis for satellite and terrestrial communication systems.

[![Crates.io](https://img.shields.io/crates/v/linkbudget.svg)](https://crates.io/crates/linkbudget)

## Features

| Module | Description |
| :--- | :--- |
| **Transmitter** | Output power, gain, EIRP (dBm/dBW) |
| **Receiver** | Gain, noise temperature, noise figure, SNR, G/T |
| **Path Loss** | Free space path loss (FSPL) from frequency and distance |
| **Link Budget** | End-to-end budget: transmitter → path loss → receiver → SNR |
| **PHY Rate** | Shannon capacity from SNR and bandwidth |
| **Orbits** | Slant range, circular orbit speed/period |
| **Doppler** | Doppler shift, received frequency, max radial velocity |
| **Power Flux Density** | PFD (dBW/m²) and PFD per MHz for regulatory analysis |
| **Quantization** | ADC/DAC quantization SNR and ENOB |
| **Modulation** | BPSK, QPSK, M-PSK, M-QAM, MSK — symbol rate, bandwidth, spectral efficiency |
| **Energy (Eb/No)** | Eb/No, Es/No, Ec/No, C/No conversions between all energy-per-bit metrics |
| **BER** | Theoretical BER curves (erfc/Q-function), required Eb/No for target BER |
| **Sensitivity** | Receiver minimum detectable signal from modulation, code rate, NF, target BER |
| **EVM** | Error Vector Magnitude ↔ SNR conversions and margin checking |

## Example

```rust
use linkbudget::{LinkBudget, PathLoss, Transmitter, Receiver};

let budget = LinkBudget {
    name: "Ka-band LEO downlink",
    bandwidth: 250.0e6,
    transmitter: Transmitter {
        output_power: 10.0,  // dBm
        gain: 35.0,          // dBi
        bandwidth: 250.0e6,  // Hz
    },
    receiver: Receiver {
        gain: 40.0,           // dBi
        temperature: 290.0,   // K
        noise_figure: 2.0,    // dB
        bandwidth: 250.0e6,   // Hz
    },
    path_loss: PathLoss {
        frequency: 20.0e9,   // 20 GHz
        distance: 550.0e3,   // 550 km LEO
    },
    frequency_dependent_loss: Some(3.0), // rain fade
};

println!("EIRP: {:.1} dBm", budget.transmitter.eirp_dbm());
println!("G/T: {:.1} dB/K", budget.receiver.g_over_t_db());
println!("Path Loss: {:.1} dB", budget.path_loss());
println!("SNR: {:.1} dB", budget.snr());
println!("PHY Rate: {:.1} Mbps", budget.phy_rate().mbps());
```

## Doppler

```rust
use linkbudget::doppler;

let freq = 14.0e9; // 14 GHz Ku-band
let radial_velocity = 7000.0; // m/s (LEO satellite)

let shift = doppler::doppler_shift_hz(freq, radial_velocity);
println!("Doppler shift: {:.0} Hz", shift); // ~327 kHz
```

## Power Flux Density

```rust
use linkbudget::pfd;

let eirp_dbw = 45.0;
let distance = 550.0e3; // 550 km

let pfd = pfd::power_flux_density_dbw_per_m2(eirp_dbw, distance);
println!("PFD: {:.1} dBW/m²", pfd);
```

## Quantization

```rust
use linkbudget::quantization;

let snr = quantization::quantization_snr_db(12); // 12-bit ADC
println!("Quantization SNR: {:.2} dB", snr); // 74.00 dB

let enob = quantization::enob_from_snr(65.0);
println!("ENOB: {:.1} bits", enob); // 10.5 bits
```

## Modulation & BER

```rust
use linkbudget::{Modulation, ber, energy, sensitivity};

// QPSK, 10 Mbps, rate-3/4 FEC
let mod_qpsk = Modulation::Qpsk;
let rb = 10.0e6; // info bit rate
let code_rate = 0.75;

// Symbol rate and occupied bandwidth
let rs = mod_qpsk.symbol_rate(rb, code_rate);
let bw = mod_qpsk.occupied_bandwidth(rs, 0.35); // 35% roll-off
println!("Symbol rate: {:.2} Msps", rs / 1e6);
println!("Occupied BW: {:.2} MHz", bw / 1e6);

// BER at Eb/No = 10 dB
let ber_val = ber::ber_from_db(10.0, &mod_qpsk);
println!("BER at 10 dB Eb/No: {:.2e}", ber_val);

// Required Eb/No for BER = 1e-6
let req = ber::required_eb_no_db(1e-6, &mod_qpsk).unwrap();
println!("Required Eb/No for BER=1e-6: {:.1} dB", req);
```

## Energy-per-Bit Conversions

```rust
use linkbudget::energy;

// SNR = 20 dB in 10 MHz noise bandwidth → C/No → Eb/No
let c_no = energy::snr_to_c_over_no(20.0, 10e6);
let eb_no = energy::c_over_no_to_eb_over_no(c_no, 5e6); // 5 Mbps
println!("C/No: {:.1} dB·Hz", c_no);
println!("Eb/No: {:.1} dB", eb_no);
```

## Sensitivity

```rust
use linkbudget::{Modulation, sensitivity};

// QPSK, 10 Mbps, rate-3/4, NF=3 dB, target BER=1e-6, 2 dB impl loss

// Matched filter (root-raised-cosine) — ideal, α-independent
let matched = sensitivity::sensitivity_matched_filter_dbm(
    &Modulation::Qpsk, 10e6, 0.75, 3.0, 1e-6, 2.0,
).unwrap();
println!("Matched filter sensitivity: {:.1} dBm", matched);

// Bandpass filter — practical, includes roll-off penalty
let bandpass = sensitivity::sensitivity_bandpass_dbm(
    &Modulation::Qpsk, 10e6, 0.75, 3.0, 1e-6, 2.0, 0.35,
).unwrap();
println!("Bandpass sensitivity (α=0.35): {:.1} dBm", bandpass);

// Roll-off penalty: how much worse bandpass is vs matched
let penalty = sensitivity::rolloff_penalty_db(0.35);
println!("Roll-off penalty: {:.2} dB", penalty); // ~1.30 dB
```

### Roll-Off Factor (α)

The roll-off factor α controls the excess bandwidth of raised-cosine pulse shaping:

| α | Excess BW | Sensitivity Penalty |
|------|-----------|---------------------|
| 0.00 | 0% (brick-wall, impractical) | 0.00 dB |
| 0.20 | 20% (DVB-S2) | 0.79 dB |
| 0.25 | 25% | 0.97 dB |
| 0.35 | 35% (legacy DVB-S) | 1.30 dB |
| 0.50 | 50% | 1.76 dB |
| 1.00 | 100% | 3.01 dB |

With a matched filter (RRC at TX + RX), noise bandwidth = symbol rate regardless of α, so sensitivity is unaffected. The penalty applies when using a simple bandpass filter set to the occupied bandwidth Rs×(1+α).

## EVM

```rust
use linkbudget::evm;

let evm_pct = evm::evm_percent_from_snr_db(25.0);
println!("EVM at 25 dB SNR: {:.1}%", evm_pct);

let snr = evm::snr_db_from_evm_percent(5.0);
println!("SNR for 5% EVM: {:.1} dB", snr);

let (pass, margin) = evm::evm_margin(5.0, 8.0);
println!("Pass: {}, margin: {:.1} dB", pass, margin);
```

## CLI

```bash
linkbudget          # runs the built-in example
```

## Articles and Papers

- [Path Loss, Antenna Gain, and Frequency Dependence](https://www.dsprelated.com/showarticle/62.php)
- [Definition of Terms: Eb/No, Es/No, C/No, SNR](https://www.dsprelated.com/showarticle/168.php)
