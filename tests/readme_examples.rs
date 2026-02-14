//! Integration tests matching every README code example.
//! If the README compiles, these compile. If these fail, the README is wrong.

use linkbudget::coding::{self, CodedModulation, FecCode};
use linkbudget::modulation::Modulation;
use linkbudget::sensitivity;
use linkbudget::{ber, energy, evm};
use linkbudget::{LinkBudget, PathLoss, Receiver, Transmitter};

// =====================================================================
// Helper: shared Ka-band LEO budget used across examples
// =====================================================================

fn ka_band_leo() -> LinkBudget {
    LinkBudget {
        name: "Ka-band LEO downlink",
        bandwidth: 36e6,
        transmitter: Transmitter {
            output_power: 10.0,
            gain: 35.0,
            bandwidth: 36e6,
        },
        receiver: Receiver {
            gain: 40.0,
            temperature: 290.0,
            noise_figure: 2.0,
            bandwidth: 36e6,
        },
        path_loss: PathLoss {
            frequency: 20.0e9,
            distance: 550.0e3,
        },
        frequency_dependent_loss: Some(3.0),
    }
}

// =====================================================================
// Link Budget — End-to-End Example
// =====================================================================

#[test]
fn link_budget_eirp() {
    let b = ka_band_leo();
    // EIRP = output_power + gain = 10 + 35 = 45 dBm
    assert!((b.transmitter.eirp_dbm() - 45.0).abs() < 0.01);
}

#[test]
fn link_budget_g_over_t() {
    let b = ka_band_leo();
    // G/T = 40 - 10*log10(290) ≈ 40 - 24.62 = 15.38
    let g_over_t = b.receiver.g_over_t_db();
    assert!((g_over_t - 15.38).abs() < 0.1);
}

#[test]
fn link_budget_path_loss_positive() {
    let b = ka_band_leo();
    let pl = b.path_loss();
    // Ka-band LEO path loss should be large (>170 dB)
    assert!(pl > 170.0, "Path loss should be >170 dB, got {:.1}", pl);
}

#[test]
fn link_budget_c_over_no() {
    let b = ka_band_leo();
    let c_no = b.c_over_no();
    // C/No = SNR + 10*log10(BW)
    let expected = b.snr() + 10.0 * 36e6_f64.log10();
    assert!((c_no - expected).abs() < 0.01);
}

#[test]
fn link_budget_snr() {
    let b = ka_band_leo();
    let snr = b.snr();
    // Should be a reasonable positive or negative number
    assert!(snr > -50.0 && snr < 100.0);
}

#[test]
fn link_budget_eb_no_qpsk() {
    let b = ka_band_leo();
    let eb_no = b.eb_no_db(&Modulation::Qpsk);
    // Eb/No = SNR - 10*log10(2) for QPSK
    let expected = b.snr() - 10.0 * 2.0_f64.log10();
    assert!((eb_no - expected).abs() < 0.01);
}

#[test]
fn link_budget_ber_valid() {
    let b = ka_band_leo();
    let ber_val = b.ber(&Modulation::Qpsk);
    assert!(ber_val >= 0.0 && ber_val <= 0.5);
}

#[test]
fn link_budget_shannon_capacity() {
    let b = ka_band_leo();
    let rate = b.phy_rate().mbps();
    assert!(rate > 0.0, "Shannon capacity should be positive");
}

#[test]
fn link_budget_margin_uncoded() {
    let b = ka_band_leo();
    let margin = b.link_margin_db(&Modulation::Qpsk, 1e-5);
    assert!(margin.is_some());
}

#[test]
fn link_budget_coded_performance() {
    let b = ka_band_leo();
    let coded = coding::dvbs2_qpsk_r34();

    let ber_coded = b.ber_coded(&coded);
    let ber_uncoded = b.ber(&Modulation::Qpsk);
    assert!(ber_coded <= ber_uncoded, "Coded BER should be <= uncoded");

    // Throughput = 36e6 * 2 * 0.75 = 54 Mbps
    let tp = b.throughput_bps(&coded);
    assert!((tp - 54e6).abs() < 1.0);

    let coded_margin = b.link_margin_coded_db(&coded, 1e-5);
    assert!(coded_margin.is_some());
}

// =====================================================================
// Modulation & BER
// =====================================================================

#[test]
fn modulation_symbol_rate() {
    let m = Modulation::Qpsk;
    // 10 Mbps, rate 3/4: Rc = 10e6/0.75 = 13.33 Mbps, Rs = 13.33/2 = 6.67 Msps
    let rs = m.symbol_rate(10e6, 0.75);
    assert!((rs - 6.667e6).abs() < 1e3);
}

#[test]
fn modulation_occupied_bandwidth() {
    let m = Modulation::Qpsk;
    let rs = m.symbol_rate(10e6, 0.75);
    let bw = m.occupied_bandwidth(rs, 0.35);
    // BW = Rs * 1.35
    assert!((bw - rs * 1.35).abs() < 1.0);
}

#[test]
fn ber_at_10db() {
    let ber_val = ber::ber_from_db(10.0, &Modulation::Qpsk);
    assert!(ber_val > 0.0 && ber_val < 1e-3);
}

#[test]
fn required_eb_no_for_1e6() {
    let req = ber::required_eb_no_db(1e-6, &Modulation::Qpsk).unwrap();
    // QPSK BER=1e-6 needs ~10.5 dB
    assert!(req > 9.0 && req < 12.0, "Expected ~10.5 dB, got {:.1}", req);
}

#[test]
fn ber_link_margin() {
    let margin = ber::link_margin_db(12.0, 1e-6, &Modulation::Qpsk).unwrap();
    // 12 dB actual - ~10.5 required ≈ 1.5 dB margin
    assert!(margin > 0.0 && margin < 5.0);
}

// =====================================================================
// Energy-per-Bit Conversions
// =====================================================================

#[test]
fn snr_to_c_over_no() {
    let c_no = energy::snr_to_c_over_no(20.0, 10e6);
    // 20 + 70 = 90 dB·Hz
    assert!((c_no - 90.0).abs() < 0.01);
}

#[test]
fn c_over_no_to_eb_over_no() {
    let c_no = energy::snr_to_c_over_no(20.0, 10e6);
    let eb_no = energy::c_over_no_to_eb_over_no(c_no, 5e6);
    // 90 - 10*log10(5e6) = 90 - 66.99 = 23.01
    assert!((eb_no - 23.01).abs() < 0.1);
}

#[test]
fn full_chain_snr_to_eb_no() {
    let eb_no = energy::snr_to_eb_over_no(20.0, 10e6, &Modulation::Qpsk, 5e6, 0.75);
    // Should be a reasonable value
    assert!(eb_no > 10.0 && eb_no < 30.0);
}

// =====================================================================
// Coded Modulation (FEC)
// =====================================================================

#[test]
fn dvbs2_preset_display() {
    let cm = coding::dvbs2_qpsk_r34();
    let s = format!("{}", cm);
    assert!(s.contains("QPSK"));
    assert!(s.contains("LDPC"));
}

#[test]
fn dvbs2_spectral_efficiency() {
    let cm = coding::dvbs2_qpsk_r34();
    // QPSK (k=2) * R=0.75 = 1.5
    assert!((cm.spectral_efficiency() - 1.5).abs() < 1e-10);
}

#[test]
fn dvbs2_throughput() {
    let cm = coding::dvbs2_qpsk_r34();
    let tp = cm.throughput_bps(36e6);
    // 36e6 * 1.5 = 54 Mbps
    assert!((tp - 54e6).abs() < 1.0);
}

#[test]
fn dvbs2_required_eb_no() {
    let cm = coding::dvbs2_qpsk_r34();
    let req = cm.required_eb_no_db(1e-5).unwrap();
    // With LDPC coding gain ~6.5 dB, should be much less than uncoded ~9.6
    assert!(req < 5.0, "Coded Eb/No should be < 5 dB, got {:.1}", req);
}

#[test]
fn dvbs2_link_margin() {
    let cm = coding::dvbs2_qpsk_r34();
    let margin = cm.link_margin_db(8.0, 1e-5).unwrap();
    assert!(margin > 0.0, "8 dB Eb/No should close DVB-S2 QPSK R=3/4");
}

#[test]
fn custom_coded_modulation() {
    let custom = CodedModulation::new(Modulation::Mqam(16), FecCode::Turbo { rate: 0.5 });
    assert_eq!(custom.code_rate(), 0.5);
    // 16-QAM (k=4) * R=0.5 = 2.0
    assert!((custom.spectral_efficiency() - 2.0).abs() < 1e-10);
}

// =====================================================================
// Sensitivity
// =====================================================================

#[test]
fn sensitivity_matched_filter() {
    let matched = sensitivity::sensitivity_matched_filter_dbm(
        &Modulation::Qpsk, 10e6, 0.75, 3.0, 1e-6, 2.0,
    )
    .unwrap();
    // Should be a reasonable sensitivity in -100 to -75 dBm range
    assert!(
        matched > -110.0 && matched < -70.0,
        "Expected reasonable sensitivity, got {:.1}",
        matched
    );
}

#[test]
fn sensitivity_bandpass_worse_than_matched() {
    let matched = sensitivity::sensitivity_matched_filter_dbm(
        &Modulation::Qpsk, 10e6, 0.75, 3.0, 1e-6, 2.0,
    )
    .unwrap();
    let bandpass = sensitivity::sensitivity_bandpass_dbm(
        &Modulation::Qpsk, 10e6, 0.75, 3.0, 1e-6, 2.0, 0.35,
    )
    .unwrap();
    assert!(bandpass > matched, "Bandpass should need more power");
}

#[test]
fn rolloff_penalty() {
    let penalty = sensitivity::rolloff_penalty_db(0.35);
    // 10*log10(1.35) ≈ 1.30
    assert!((penalty - 1.303).abs() < 0.01);
}

#[test]
fn rolloff_zero_no_penalty() {
    assert!((sensitivity::rolloff_penalty_db(0.0) - 0.0).abs() < 1e-10);
}

// =====================================================================
// EVM
// =====================================================================

#[test]
fn evm_at_25db_snr() {
    let evm_pct = evm::evm_percent_from_snr_db(25.0);
    // 10^(-25/20) * 100 = 5.62%
    assert!((evm_pct - 5.623).abs() < 0.01);
}

#[test]
fn snr_from_5pct_evm() {
    let snr = evm::snr_db_from_evm_percent(5.0);
    // -20*log10(0.05) = 26.02
    assert!((snr - 26.02).abs() < 0.1);
}

#[test]
fn evm_margin_pass() {
    let (pass, margin) = evm::evm_margin(5.0, 8.0);
    assert!(pass);
    assert!(margin > 0.0);
}

#[test]
fn evm_margin_fail() {
    let (pass, _margin) = evm::evm_margin(12.0, 8.0);
    assert!(!pass);
}

// =====================================================================
// Doppler
// =====================================================================

#[test]
fn doppler_shift_ku_band() {
    let shift = linkbudget::doppler_shift_hz(14e9, 7000.0);
    // 14e9 * 7000 / 299792458 ≈ 327 kHz
    assert!((shift - 327e3).abs() < 1e3);
}

#[test]
fn doppler_received_frequency() {
    let received = linkbudget::doppler_received_frequency(14e9, 7000.0);
    assert!(received > 14e9, "Approaching → higher frequency");
}

// =====================================================================
// Power Flux Density
// =====================================================================

#[test]
fn pfd_basic() {
    let pfd_val = linkbudget::power_flux_density_dbw_per_m2(45.0, 550e3);
    // Should be a negative number (power spread over sphere)
    assert!(pfd_val < 0.0);
}

#[test]
fn pfd_per_mhz_less_than_total() {
    let total = linkbudget::power_flux_density_dbw_per_m2(45.0, 550e3);
    let per_mhz = linkbudget::pfd_per_mhz(45.0, 550e3, 36.0);
    assert!(per_mhz < total, "PFD/MHz should be less than total PFD");
}

// =====================================================================
// Quantization
// =====================================================================

#[test]
fn quantization_snr_12bit() {
    let snr = linkbudget::quantization_snr_db(12);
    // 6.02*12 + 1.76 = 74.00
    assert!((snr - 74.0).abs() < 1e-10);
}

#[test]
fn enob_at_65db() {
    let enob = linkbudget::enob_from_snr(65.0);
    // (65 - 1.76) / 6.02 = 10.506
    assert!((enob - 10.506).abs() < 0.01);
}

#[test]
fn quantization_roundtrip() {
    let bits = 14;
    let snr = linkbudget::quantization_snr_db(bits);
    let enob = linkbudget::enob_from_snr(snr);
    assert!((enob - bits as f64).abs() < 1e-10);
}
