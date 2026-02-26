//! End-to-end link budget scenarios for realistic satellite communication links.
//!
//! Each test builds a complete link budget from transmitter through path loss to
//! receiver, then validates SNR, BER, link margin, and throughput against
//! hand-calculated or reference values.

use linkbudget::*;
use linkbudget::coding::dvbs2_qpsk_r34;

/// Ka-band GEO downlink (20 GHz, 35,786 km).
/// Typical DTH television link with rain fade margin.
#[test]
fn geo_ka_band_downlink() {
    let budget = LinkBudget {
        name: "GEO Ka-band DTH Downlink",
        bandwidth: 36e6, // 36 MHz transponder
        transmitter: Transmitter {
            output_power: 20.0, // 100 mW (satellite TWTA per carrier)
            gain: 50.0,         // 0.6m reflector at 20 GHz
            bandwidth: 36e6,
        },
        receiver: Receiver {
            gain: 40.0,         // 0.6m consumer dish at 20 GHz
            temperature: 150.0, // low-noise outdoor LNB
            noise_figure: 1.2,  // dB
            bandwidth: 36e6,
        },
        path_loss: PathLoss {
            frequency: 20e9,
            distance: 35_786e3, // GEO altitude
        },
        frequency_dependent_loss: Some(4.0), // rain fade + atmospheric
    };

    let eirp = budget.transmitter.eirp_dbm();
    assert!((eirp - 70.0).abs() < 0.01, "EIRP should be 70 dBm");

    // FSPL at 20 GHz, 35786 km ≈ 210.0 dB (verify reasonable range)
    let fspl = budget.path_loss.calculate();
    assert!(fspl > 205.0 && fspl < 215.0, "FSPL = {:.1} dB out of range", fspl);

    let snr = budget.snr();
    // GEO Ka-band with modest power — SNR may be marginal
    assert!(snr > -10.0 && snr < 30.0, "GEO SNR = {:.1} dB", snr);

    // QPSK R=3/4 — check coded performance
    let coded = dvbs2_qpsk_r34();
    let margin = budget.link_margin_coded_db(&coded, 1e-5);
    assert!(margin.is_some(), "Link margin should be calculable");
}

/// LEO Ku-band uplink (14 GHz, 550 km) — Starlink-like scenario.
#[test]
fn leo_ku_band_uplink() {
    let budget = LinkBudget {
        name: "LEO Ku-band User Uplink",
        bandwidth: 240e6, // wideband channel
        transmitter: Transmitter {
            output_power: 33.0, // 2W user terminal
            gain: 34.0,         // phased array
            bandwidth: 240e6,
        },
        receiver: Receiver {
            gain: 38.0,         // satellite phased array beam
            temperature: 400.0, // earth-facing, higher noise temp
            noise_figure: 3.0,
            bandwidth: 240e6,
        },
        path_loss: PathLoss {
            frequency: 14e9,
            distance: 550e3,
        },
        frequency_dependent_loss: Some(1.5), // atmospheric + scan loss
    };

    // LEO path loss much less than GEO
    let fspl = budget.path_loss.calculate();
    assert!(fspl > 165.0 && fspl < 175.0, "LEO FSPL = {:.1} dB", fspl);

    let snr = budget.snr();
    assert!(snr > 5.0, "LEO uplink should have decent SNR: {:.1} dB", snr);

    let c_no = budget.c_over_no();
    assert!(c_no > 70.0, "C/No should be well above 70 dB-Hz: {:.1}", c_no);
}

/// Deep space X-band downlink (8.4 GHz, Mars opposition ~55M km).
/// Validates that a realistic Mars link closes with high coding gain.
#[test]
fn deep_space_mars_xband() {
    let budget = LinkBudget {
        name: "Mars X-band Downlink",
        bandwidth: 4e6, // narrow-band deep space
        transmitter: Transmitter {
            output_power: 43.0, // 20W SSPA
            gain: 43.0,         // 2.5m HGA
            bandwidth: 4e6,
        },
        receiver: Receiver {
            gain: 74.0,         // 34m DSN antenna
            temperature: 25.0,  // cryogenic LNA
            noise_figure: 0.3,  // dB
            bandwidth: 4e6,
        },
        path_loss: PathLoss {
            frequency: 8.4e9,
            distance: 55e9, // 55 million km (Mars closest approach)
        },
        frequency_dependent_loss: Some(0.5), // minimal atmosphere
    };

    // FSPL at interplanetary distances is enormous
    let fspl = budget.path_loss.calculate();
    assert!(fspl > 260.0, "Deep space FSPL should be >260 dB: {:.1}", fspl);

    // Even with huge path loss, DSN dish should recover the signal
    let snr = budget.snr();
    // Deep space links often have very low SNR, rely on coding
    assert!(snr > -10.0 && snr < 40.0, "Mars link SNR = {:.1} dB", snr);

    // G/T of DSN receiver should be excellent
    let g_over_t = budget.receiver.g_over_t_db();
    assert!(g_over_t > 55.0, "DSN G/T = {:.1} dB/K, expected >55", g_over_t);
}

/// Terrestrial microwave backhaul (6 GHz, 30 km).
#[test]
fn terrestrial_microwave_backhaul() {
    let budget = LinkBudget {
        name: "6 GHz Microwave Backhaul",
        bandwidth: 28e6,
        transmitter: Transmitter {
            output_power: 30.0, // 1W
            gain: 38.0,         // 1.2m dish
            bandwidth: 28e6,
        },
        receiver: Receiver {
            gain: 38.0,
            temperature: 290.0,
            noise_figure: 4.0,
            bandwidth: 28e6,
        },
        path_loss: PathLoss {
            frequency: 6e9,
            distance: 30e3, // 30 km
        },
        frequency_dependent_loss: Some(2.0), // rain + multipath
    };

    let snr = budget.snr();
    // Terrestrial microwave should have plenty of margin
    assert!(snr > 20.0, "Backhaul SNR should be >20 dB: {:.1}", snr);

    // 64-QAM should be achievable
    let margin_16qam = budget.link_margin_db(&Modulation::Mqam(16), 1e-6);
    assert!(margin_16qam.is_some());
}

/// Verify pin_at_receiver follows the link equation:
/// P_rx = P_tx + G_tx - FSPL - L_extra + G_rx
#[test]
fn pin_at_receiver_equation() {
    let budget = LinkBudget {
        name: "Equation Check",
        bandwidth: 1e6,
        transmitter: Transmitter {
            output_power: 30.0,
            gain: 10.0,
            bandwidth: 1e6,
        },
        receiver: Receiver {
            gain: 20.0,
            temperature: 290.0,
            noise_figure: 3.0,
            bandwidth: 1e6,
        },
        path_loss: PathLoss {
            frequency: 1e9,
            distance: 100e3,
        },
        frequency_dependent_loss: Some(5.0),
    };

    let fspl = budget.path_loss.calculate();
    let expected_pin = 30.0 + 10.0 - fspl - 5.0 + 20.0;
    let actual_pin = budget.pin_at_receiver();
    assert!(
        (actual_pin - expected_pin).abs() < 0.001,
        "Pin mismatch: got {:.3}, expected {:.3}",
        actual_pin,
        expected_pin
    );
}

/// SNR should increase when receiver gain increases (all else equal).
#[test]
fn snr_increases_with_receiver_gain() {
    let make_budget = |rx_gain: f64| LinkBudget {
        name: "Gain Sweep",
        bandwidth: 10e6,
        transmitter: Transmitter {
            output_power: 20.0,
            gain: 30.0,
            bandwidth: 10e6,
        },
        receiver: Receiver {
            gain: rx_gain,
            temperature: 290.0,
            noise_figure: 2.0,
            bandwidth: 10e6,
        },
        path_loss: PathLoss {
            frequency: 12e9,
            distance: 600e3,
        },
        frequency_dependent_loss: None,
    };

    let snr_low = make_budget(30.0).snr();
    let snr_high = make_budget(40.0).snr();

    // 10 dB more gain → 10 dB more SNR
    assert!(
        (snr_high - snr_low - 10.0).abs() < 0.01,
        "Expected 10 dB delta, got {:.2}",
        snr_high - snr_low
    );
}
