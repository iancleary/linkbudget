//! Error Vector Magnitude (EVM) calculations.
//!
//! EVM quantifies the difference between measured and ideal constellation points.
//! It's the standard metric for modem/transmitter quality.
//!
//! Key relationships:
//! - EVM_rms = 1 / sqrt(SNR_linear)
//! - SNR = -20·log10(EVM_rms)
//! - EVM is often expressed as a percentage: EVM% = EVM_rms × 100

/// EVM (rms, fractional) from SNR in dB
/// EVM_rms = 10^(-SNR_dB / 20)
pub fn evm_from_snr_db(snr_db: f64) -> f64 {
    10.0_f64.powf(-snr_db / 20.0)
}

/// EVM as a percentage from SNR in dB
pub fn evm_percent_from_snr_db(snr_db: f64) -> f64 {
    evm_from_snr_db(snr_db) * 100.0
}

/// SNR in dB from EVM (rms, fractional)
/// SNR_dB = -20·log10(EVM_rms)
pub fn snr_db_from_evm(evm_rms: f64) -> f64 {
    -20.0 * evm_rms.log10()
}

/// SNR in dB from EVM percentage
pub fn snr_db_from_evm_percent(evm_percent: f64) -> f64 {
    snr_db_from_evm(evm_percent / 100.0)
}

/// EVM from SNR linear (not dB)
pub fn evm_from_snr_linear(snr_linear: f64) -> f64 {
    1.0 / snr_linear.sqrt()
}

/// Check if measured EVM meets a requirement
/// Returns (pass, margin_db) where margin is how much better than required
pub fn evm_margin(measured_evm_percent: f64, required_evm_percent: f64) -> (bool, f64) {
    let measured_snr = snr_db_from_evm_percent(measured_evm_percent);
    let required_snr = snr_db_from_evm_percent(required_evm_percent);
    let margin = measured_snr - required_snr;
    (margin >= 0.0, margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evm_at_20db_snr() {
        // SNR = 20 dB → EVM = 10% = 0.1
        let evm = evm_from_snr_db(20.0);
        assert!((evm - 0.1).abs() < 1e-10);
    }

    #[test]
    fn evm_percent_at_20db() {
        let evm_pct = evm_percent_from_snr_db(20.0);
        assert!((evm_pct - 10.0).abs() < 1e-8);
    }

    #[test]
    fn snr_from_10_percent_evm() {
        // EVM = 10% → SNR = 20 dB
        let snr = snr_db_from_evm_percent(10.0);
        assert!((snr - 20.0).abs() < 1e-8);
    }

    #[test]
    fn roundtrip_snr_evm() {
        let snr = 25.0;
        let evm = evm_from_snr_db(snr);
        let snr_back = snr_db_from_evm(evm);
        assert!((snr - snr_back).abs() < 1e-10);
    }

    #[test]
    fn evm_linear_vs_db() {
        let snr_linear = 100.0; // 20 dB
        let evm_lin = evm_from_snr_linear(snr_linear);
        let evm_db = evm_from_snr_db(20.0);
        assert!((evm_lin - evm_db).abs() < 1e-10);
    }

    #[test]
    fn evm_margin_pass() {
        // Measured 5%, required 8% → pass with margin
        let (pass, margin) = evm_margin(5.0, 8.0);
        assert!(pass);
        assert!(margin > 0.0);
    }

    #[test]
    fn evm_margin_fail() {
        // Measured 12%, required 8% → fail
        let (pass, margin) = evm_margin(12.0, 8.0);
        assert!(!pass);
        assert!(margin < 0.0);
    }

    #[test]
    fn common_evm_values() {
        // 64-QAM typically requires EVM < 8% → SNR > ~22 dB
        let snr_64qam = snr_db_from_evm_percent(8.0);
        assert!(snr_64qam > 21.0 && snr_64qam < 23.0);

        // 256-QAM typically requires EVM < 3.5% → SNR > ~29 dB
        let snr_256qam = snr_db_from_evm_percent(3.5);
        assert!(snr_256qam > 28.0 && snr_256qam < 30.0);
    }
}
