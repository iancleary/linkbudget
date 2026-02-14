//! Receiver sensitivity calculator.
//!
//! Computes the minimum detectable signal (MDS) for a target BER,
//! given modulation, code rate, noise figure, and bandwidth.
//!
//! Sensitivity = noise_floor + required_SNR + implementation_loss
//!             = -174 + 10·log10(BW) + NF + required_Eb/No + 10·log10(Rb/BW) + impl_loss

use crate::ber;
use crate::modulation::Modulation;

/// Thermal noise floor in dBm for a given bandwidth
/// N = -174 dBm/Hz + 10·log10(BW_Hz) + NF_dB
pub fn noise_floor_dbm(bandwidth_hz: f64, noise_figure_db: f64) -> f64 {
    -174.0 + 10.0 * bandwidth_hz.log10() + noise_figure_db
}

/// Receiver sensitivity in dBm.
///
/// # Arguments
/// * `modulation` - Modulation scheme
/// * `info_bit_rate_bps` - Information (payload) bit rate Rb
/// * `code_rate` - FEC code rate R (e.g. 0.5 for rate-1/2)
/// * `noise_figure_db` - Receiver noise figure in dB
/// * `target_ber` - Required BER (e.g. 1e-6)
/// * `implementation_loss_db` - Additional loss margin (modem imperfections, etc.)
/// * `rolloff` - Pulse shaping roll-off factor (e.g. 0.35)
///
/// # Returns
/// Minimum input power in dBm to achieve the target BER.
pub fn sensitivity_dbm(
    modulation: &Modulation,
    info_bit_rate_bps: f64,
    code_rate: f64,
    noise_figure_db: f64,
    target_ber: f64,
    implementation_loss_db: f64,
    _rolloff: f64,
) -> Option<f64> {
    // Required Eb/No for the target BER (uncoded)
    let required_eb_no_db = ber::required_eb_no_db(target_ber, modulation)?;

    // Required SNR = Eb/No + 10·log10(Rb/BW_noise)
    // But more directly:
    // Sensitivity = N₀ + Eb/No + 10·log10(Rb)
    //             = -174 + NF + Eb/No(dB) + 10·log10(Rb) + impl_loss
    // This is equivalent to: noise_floor + required_SNR where
    // required_SNR = Eb/No(dB) + 10·log10(Rb) - 10·log10(BW_noise)
    let sensitivity = -174.0 + noise_figure_db + required_eb_no_db
        + 10.0 * info_bit_rate_bps.log10()
        + implementation_loss_db;

    Some(sensitivity)
}

/// Simplified sensitivity: just noise floor + required SNR
/// For quick estimates when you know the required SNR directly.
pub fn sensitivity_from_snr_dbm(
    bandwidth_hz: f64,
    noise_figure_db: f64,
    required_snr_db: f64,
    implementation_loss_db: f64,
) -> f64 {
    noise_floor_dbm(bandwidth_hz, noise_figure_db) + required_snr_db + implementation_loss_db
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modulation::Modulation;

    #[test]
    fn noise_floor_1mhz_3db_nf() {
        // -174 + 60 + 3 = -111 dBm
        let nf = noise_floor_dbm(1e6, 3.0);
        assert!((nf - (-111.0)).abs() < 0.01);
    }

    #[test]
    fn noise_floor_100mhz_0db_nf() {
        // -174 + 80 + 0 = -94 dBm
        let nf = noise_floor_dbm(100e6, 0.0);
        assert!((nf - (-94.0)).abs() < 0.01);
    }

    #[test]
    fn sensitivity_bpsk_1mbps() {
        // BPSK, 1 Mbps, rate 1 (uncoded), NF=3 dB, BER=1e-5, 0 dB impl loss, rolloff 0.35
        // Required Eb/No ≈ 9.6 dB
        // Sensitivity ≈ -174 + 3 + 9.6 + 10*log10(1e6) + 0
        //             = -174 + 3 + 9.6 + 60 = -101.4 dBm
        let sens = sensitivity_dbm(
            &Modulation::Bpsk,
            1e6,
            1.0,
            3.0,
            1e-5,
            0.0,
            0.35,
        ).unwrap();
        assert!(sens > -103.0 && sens < -100.0,
            "Expected ~-101.4 dBm, got {}", sens);
    }

    #[test]
    fn sensitivity_qpsk_10mbps() {
        // QPSK, 10 Mbps, rate 3/4, NF=5 dB, BER=1e-6, 2 dB impl loss
        let sens = sensitivity_dbm(
            &Modulation::Qpsk,
            10e6,
            0.75,
            5.0,
            1e-6,
            2.0,
            0.35,
        ).unwrap();
        // Should be a reasonable number in the -80 to -95 dBm range
        assert!(sens > -100.0 && sens < -75.0,
            "Expected sensitivity in -100 to -75 dBm range, got {}", sens);
    }

    #[test]
    fn sensitivity_from_snr_simple() {
        // 10 MHz BW, 3 dB NF, 10 dB required SNR, 1 dB impl loss
        // -174 + 70 + 3 + 10 + 1 = -90 dBm
        let sens = sensitivity_from_snr_dbm(10e6, 3.0, 10.0, 1.0);
        assert!((sens - (-90.0)).abs() < 0.01);
    }

    #[test]
    fn higher_rate_needs_more_power() {
        // Same modulation/BER, higher bit rate should need more power
        let sens_1m = sensitivity_dbm(&Modulation::Bpsk, 1e6, 1.0, 3.0, 1e-5, 0.0, 0.35).unwrap();
        let sens_10m = sensitivity_dbm(&Modulation::Bpsk, 10e6, 1.0, 3.0, 1e-5, 0.0, 0.35).unwrap();
        assert!(sens_10m > sens_1m, "Higher bit rate should require more power");
        // Difference should be 10 dB (10x bit rate)
        assert!(((sens_10m - sens_1m) - 10.0).abs() < 0.5);
    }
}
