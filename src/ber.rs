//! Bit Error Rate (BER) curves for common modulation schemes.
//!
//! Provides theoretical BER as a function of Eb/No (uncoded) or
//! required Eb/No for a target BER.
//!
//! All Eb/No values are in **linear** (not dB) unless suffixed with `_db`.

use std::f64::consts::PI;

use crate::modulation::Modulation;

/// Complementary error function approximation (erfc).
/// Uses Abramowitz & Stegun approximation 7.1.26, max error ~1.5e-7.
pub fn erfc(x: f64) -> f64 {
    if x < 0.0 {
        return 2.0 - erfc(-x);
    }
    let t = 1.0 / (1.0 + 0.3275911 * x);
    let poly = t
        * (0.254829592
            + t * (-0.284496736
                + t * (1.421413741 + t * (-1.453152027 + t * 1.061405429))));
    poly * (-x * x).exp()
}

/// Q-function: Q(x) = 0.5 * erfc(x / sqrt(2))
pub fn q_function(x: f64) -> f64 {
    0.5 * erfc(x / std::f64::consts::SQRT_2)
}

/// BER for BPSK/QPSK (they have the same BER vs Eb/No)
/// BER = Q(sqrt(2 * Eb/No))
pub fn ber_bpsk(eb_no_linear: f64) -> f64 {
    q_function((2.0 * eb_no_linear).sqrt())
}

/// BER for M-PSK (M >= 4, Gray coded)
/// BER ≈ (2/k) * Q(sqrt(2*k*Eb/No) * sin(π/M))
/// where k = log2(M)
pub fn ber_mpsk(eb_no_linear: f64, m: u32) -> f64 {
    if m == 2 {
        return ber_bpsk(eb_no_linear);
    }
    let k = (m as f64).log2();
    let sin_term = (PI / m as f64).sin();
    (2.0 / k) * q_function((2.0 * k * eb_no_linear).sqrt() * sin_term)
}

/// BER for rectangular M-QAM (Gray coded, M = 4, 16, 64, 256, ...)
/// BER ≈ (4/k) * (1 - 1/√M) * Q(sqrt(3*k*Eb/No / (M-1)))
///
/// For M=4 (QPSK), this reduces to the QPSK formula.
pub fn ber_mqam(eb_no_linear: f64, m: u32) -> f64 {
    if m == 4 {
        return ber_bpsk(eb_no_linear); // QPSK = BPSK in Eb/No
    }
    let k = (m as f64).log2();
    let sqrt_m = (m as f64).sqrt();
    let coeff = (4.0 / k) * (1.0 - 1.0 / sqrt_m);
    let arg = (3.0 * k * eb_no_linear / (m as f64 - 1.0)).sqrt();
    coeff * q_function(arg)
}

/// BER for any supported modulation type
pub fn ber(eb_no_linear: f64, modulation: &Modulation) -> f64 {
    match modulation {
        Modulation::Bpsk => ber_bpsk(eb_no_linear),
        Modulation::Qpsk => ber_bpsk(eb_no_linear),
        Modulation::Mpsk(m) => ber_mpsk(eb_no_linear, *m),
        Modulation::Mqam(m) => ber_mqam(eb_no_linear, *m),
        Modulation::Msk => ber_bpsk(eb_no_linear), // MSK has same BER as BPSK
    }
}

/// BER from Eb/No in dB
pub fn ber_from_db(eb_no_db: f64, modulation: &Modulation) -> f64 {
    let eb_no_linear = 10.0_f64.powf(eb_no_db / 10.0);
    ber(eb_no_linear, modulation)
}

/// Required Eb/No (dB) for a target BER, found by bisection search.
/// Returns None if no solution found in [−5, 50] dB range.
pub fn required_eb_no_db(target_ber: f64, modulation: &Modulation) -> Option<f64> {
    let mut lo = -5.0_f64;
    let mut hi = 50.0_f64;

    // BER decreases as Eb/No increases, so we search for the crossing
    for _ in 0..100 {
        let mid = (lo + hi) / 2.0;
        let ber_mid = ber_from_db(mid, modulation);
        if (ber_mid - target_ber).abs() / target_ber < 1e-6 {
            return Some(mid);
        }
        if ber_mid > target_ber {
            lo = mid; // need more Eb/No
        } else {
            hi = mid; // too much Eb/No
        }
    }
    Some((lo + hi) / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modulation::Modulation;

    #[test]
    fn erfc_zero() {
        assert!((erfc(0.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn erfc_large() {
        assert!(erfc(5.0) < 1e-10);
    }

    #[test]
    fn erfc_negative() {
        assert!((erfc(-0.0) - 1.0).abs() < 1e-6);
        // erfc(-x) = 2 - erfc(x)
        let val = erfc(-1.0);
        assert!((val - (2.0 - erfc(1.0))).abs() < 1e-6);
    }

    #[test]
    fn bpsk_ber_at_known_points() {
        // At Eb/No = 0 dB (linear = 1): BER ≈ 0.0786
        let ber_0db = ber_bpsk(1.0);
        assert!((ber_0db - 0.0786).abs() < 0.005);

        // At Eb/No = 10 dB (linear = 10): BER ≈ 3.87e-6
        let ber_10db = ber_bpsk(10.0);
        assert!(ber_10db < 1e-4);
        assert!(ber_10db > 1e-7);
    }

    #[test]
    fn qpsk_same_as_bpsk() {
        // QPSK and BPSK have identical BER vs Eb/No
        let eb_no = 5.0;
        let ber_bpsk_val = ber_bpsk(eb_no);
        let ber_qpsk_val = ber(eb_no, &Modulation::Qpsk);
        assert!((ber_bpsk_val - ber_qpsk_val).abs() < 1e-15);
    }

    #[test]
    fn higher_order_qam_needs_more_eb_no() {
        // 64-QAM needs more Eb/No than 16-QAM for same BER
        let eb_no = 10.0; // linear
        let ber_16 = ber_mqam(eb_no, 16);
        let ber_64 = ber_mqam(eb_no, 64);
        assert!(ber_64 > ber_16, "64-QAM should have higher BER than 16-QAM at same Eb/No");
    }

    #[test]
    fn required_eb_no_bpsk_1e_minus_5() {
        // BPSK BER = 1e-5 requires Eb/No ≈ 9.6 dB
        let eb_no = required_eb_no_db(1e-5, &Modulation::Bpsk).unwrap();
        assert!((eb_no - 9.6).abs() < 0.2, "Expected ~9.6 dB, got {}", eb_no);
    }

    #[test]
    fn required_eb_no_16qam_1e_minus_6() {
        // 16-QAM BER = 1e-6 requires Eb/No ≈ 14.4 dB
        let eb_no = required_eb_no_db(1e-6, &Modulation::Mqam(16)).unwrap();
        assert!(eb_no > 13.0 && eb_no < 16.0, "Expected ~14.4 dB, got {}", eb_no);
    }

    #[test]
    fn ber_from_db_wrapper() {
        let ber_val = ber_from_db(10.0, &Modulation::Bpsk);
        let ber_direct = ber_bpsk(10.0_f64.powf(10.0 / 10.0));
        assert!((ber_val - ber_direct).abs() < 1e-15);
    }

    #[test]
    fn msk_same_as_bpsk() {
        let eb_no = 7.0;
        let ber_msk = ber(eb_no, &Modulation::Msk);
        let ber_bpsk_val = ber_bpsk(eb_no);
        assert!((ber_msk - ber_bpsk_val).abs() < 1e-15);
    }

    #[test]
    fn ber_8psk() {
        // 8-PSK at Eb/No = 10 dB should have higher BER than QPSK
        let eb_no = 10.0_f64.powf(10.0 / 10.0);
        let ber_qpsk = ber(eb_no, &Modulation::Qpsk);
        let ber_8psk = ber(eb_no, &Modulation::Mpsk(8));
        assert!(ber_8psk > ber_qpsk, "8-PSK should have higher BER than QPSK");
    }
}
