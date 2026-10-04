//! Independent thermal-noise and ideal AWGN capacity checks.
//!
//! The oracles use SI defining constants, spherical power spreading, effective
//! aperture, and kT noise density. They do not call the crate's RF conversions.

use std::f64::consts::PI;

use linkbudget::{
    CodedModulation, FecCode, LinkBudget, Modulation, PathLoss, Receiver, ReceiverNoise,
    Transmitter,
};

// Exact SI defining constants: https://www.bipm.org/en/measurement-units/si-defining-constants
const BOLTZMANN_J_PER_K: f64 = 1.380_649e-23;
const SPEED_OF_LIGHT_M_PER_S: f64 = 299_792_458.0;
const CHANNEL_BANDWIDTH_HZ: f64 = 20e6;
const SYSTEM_TEMPERATURE_K: f64 = 340.0;

fn assert_close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "got {actual}, expected {expected}, tolerance {tolerance}"
    );
}

fn watts_to_dbm(watts: f64) -> f64 {
    10.0 * (watts / 1e-3).log10()
}

fn cold_source_noise() -> ReceiverNoise {
    ReceiverNoise::SourceAndNoiseFigure {
        source_temperature_k: 50.0,
        noise_figure_db: 10.0 * 2.0_f64.log10(),
    }
}

fn physical_budget(noise: ReceiverNoise, noise_bandwidth_hz: f64) -> LinkBudget {
    // Choose transmitter power to provide C/N0 = 1e8 Hz (80 dB-Hz) at 340 K.
    let frequency_hz = 10e9;
    let distance_m: f64 = 1e6;
    let tx_gain_linear = 1e3;
    let rx_gain_linear = 1e4;
    let extra_loss_db = 3.0;
    let extra_loss_linear = 10.0_f64.powf(extra_loss_db / 10.0);
    let wavelength_m = SPEED_OF_LIGHT_M_PER_S / frequency_hz;
    let rx_aperture_m2 = rx_gain_linear * wavelength_m.powi(2) / (4.0 * PI);
    let received_power_w = BOLTZMANN_J_PER_K * SYSTEM_TEMPERATURE_K * 1e8;
    let transmitted_power_w = received_power_w * 4.0 * PI * distance_m.powi(2) * extra_loss_linear
        / (tx_gain_linear * rx_aperture_m2);

    LinkBudget {
        name: "Independent physics oracle",
        bandwidth: CHANNEL_BANDWIDTH_HZ,
        transmitter: Transmitter {
            output_power: watts_to_dbm(transmitted_power_w),
            gain: 30.0,
            bandwidth: CHANNEL_BANDWIDTH_HZ,
        },
        receiver: Receiver {
            gain: 40.0,
            noise,
            bandwidth: noise_bandwidth_hz,
        },
        path_loss: PathLoss {
            frequency: frequency_hz,
            distance: distance_m,
        },
        frequency_dependent_loss: Some(extra_loss_db),
    }
}

#[test]
fn capacity_uses_channel_bandwidth_when_noise_bandwidth_differs() {
    // The receiver admits the entire signal in both cases. Changing its noise
    // bandwidth changes integrated noise, not signal power or noise density.
    let narrow = physical_budget(cold_source_noise(), 30e6);
    let wide = physical_budget(cold_source_noise(), 60e6);
    let expected_capacity_bps = CHANNEL_BANDWIDTH_HZ * (1.0 + 1e8 / CHANNEL_BANDWIDTH_HZ).log2();

    assert_close(narrow.c_over_no(), 80.0, 1e-10);
    assert_close(wide.c_over_no(), 80.0, 1e-10);
    assert_close(wide.snr() - narrow.snr(), -10.0 * 2.0_f64.log10(), 1e-10);
    assert_close(narrow.phy_rate().bps(), expected_capacity_bps, 1e-6);
    assert_close(wide.phy_rate().bps(), expected_capacity_bps, 1e-6);
    assert_close(narrow.phy_rate().mbps(), 51.699_250_014_423_12, 1e-10);
}

#[test]
fn capacity_preserves_equal_bandwidth_result() {
    let budget = physical_budget(cold_source_noise(), CHANNEL_BANDWIDTH_HZ);
    let expected_snr = 1e8 / CHANNEL_BANDWIDTH_HZ;
    let expected_capacity_bps = CHANNEL_BANDWIDTH_HZ * (1.0 + expected_snr).log2();

    assert_close(budget.snr_linear(), expected_snr, 1e-10);
    assert_close(budget.phy_rate().snr, budget.snr_linear(), 1e-10);
    assert_close(budget.phy_rate().bps(), expected_capacity_bps, 1e-6);
}

#[test]
fn equivalent_receiver_descriptions_produce_the_same_link() {
    let source = physical_budget(cold_source_noise(), 30e6);
    let system = physical_budget(
        ReceiverNoise::SystemTemperature {
            temperature_k: SYSTEM_TEMPERATURE_K,
        },
        30e6,
    );
    let coded = CodedModulation::new(Modulation::Qpsk, FecCode::Ldpc { rate: 0.5 });

    assert_close(
        source.receiver.system_noise_temperature_k(),
        system.receiver.system_noise_temperature_k(),
        1e-10,
    );
    assert_close(
        source.receiver.calculate_noise_power(),
        system.receiver.calculate_noise_power(),
        1e-10,
    );
    assert_close(source.c_over_no(), system.c_over_no(), 1e-10);
    assert_close(
        source.receiver.g_over_t_db(),
        system.receiver.g_over_t_db(),
        1e-10,
    );
    assert_close(
        source.eb_no_db(&Modulation::Qpsk),
        system.eb_no_db(&Modulation::Qpsk),
        1e-10,
    );
    assert_close(
        source.eb_no_coded_db(&coded),
        system.eb_no_coded_db(&coded),
        1e-10,
    );
    assert_close(source.phy_rate().bps(), system.phy_rate().bps(), 1e-6);
}

#[test]
fn cold_source_matches_kt_noise_and_g_over_t_link_equation() {
    let budget = physical_budget(cold_source_noise(), 30e6);
    // F = 2 is specified at the 290 K reference temperature, so the receiver
    // contributes Te = 290*(2-1) = 290 K. A 50 K source makes Tsys = 340 K.
    let noise_density_w_per_hz = BOLTZMANN_J_PER_K * SYSTEM_TEMPERATURE_K;
    let expected_noise_w = noise_density_w_per_hz * 30e6;
    let expected_signal_w = noise_density_w_per_hz * 1e8;
    let expected_g_over_t_db = 40.0 - 10.0 * SYSTEM_TEMPERATURE_K.log10();

    assert_close(budget.receiver.system_noise_temperature_k(), 340.0, 1e-10);
    assert_close(
        budget.receiver.calculate_noise_power(),
        watts_to_dbm(expected_noise_w),
        1e-10,
    );
    assert_close(
        budget.pin_at_receiver(),
        watts_to_dbm(expected_signal_w),
        1e-10,
    );
    assert_close(budget.receiver.g_over_t_db(), expected_g_over_t_db, 1e-10);
    assert_close(
        budget.c_over_no(),
        10.0 * (expected_signal_w / noise_density_w_per_hz).log10(),
        1e-10,
    );

    // C/N0[dB-Hz] = EIRP[dBW] - path loss[dB] + G/T[dB/K] - k[dBW/K/Hz].
    let wavelength_m = SPEED_OF_LIGHT_M_PER_S / budget.path_loss.frequency;
    let free_space_loss_db = 20.0 * (4.0 * PI * budget.path_loss.distance / wavelength_m).log10();
    let c_over_no_from_g_over_t =
        budget.transmitter.output_power - 30.0 + budget.transmitter.gain - free_space_loss_db - 3.0
            + budget.receiver.g_over_t_db()
            - 10.0 * BOLTZMANN_J_PER_K.log10();
    assert_close(budget.c_over_no(), c_over_no_from_g_over_t, 1e-10);

    // QPSK sends two bits per zero-roll-off symbol: Eb = C/(2*20e6).
    let expected_eb_no_db = 10.0 * (expected_signal_w / (40e6 * noise_density_w_per_hz)).log10();
    assert_close(budget.eb_no_db(&Modulation::Qpsk), expected_eb_no_db, 1e-10);
}
