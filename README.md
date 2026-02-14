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

## CLI

```bash
linkbudget          # runs the built-in example
```

## Articles and Papers

- [Path Loss, Antenna Gain, and Frequency Dependence](https://www.dsprelated.com/showarticle/62.php)
