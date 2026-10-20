//! Bit Error Rate (BER) curves for common modulation schemes.
//!
//! Provides theoretical BER as a function of Eb/No (uncoded) or
//! required Eb/No for a target BER.
//!
//! Eb/No inputs are **linear** (not dB) unless suffixed with `_db`.
//! Required-Eb/No inverse helpers return **dB**, as documented on each helper.
//! Assume coherent detection in AWGN, correct synchronization, and no residual
//! ISI. Higher-order curves are nearest-neighbor approximations, not exact BER.
//! See `docs/physics-communications.md` for Gaussian-decision derivations,
//! constellation restrictions, and numerical accuracy limits.

use std::f64::consts::PI;

use crate::gaussian::{log_erfc_positive, log_q_positive};
use crate::modulation::Modulation;
use crate::ModelError;

/// Complementary error function, using a Taylor series and gamma continued fraction.
///
/// Empirically checked to relative error below 1e-12 through x=26 (not a
/// rigorous bound). Subnormal tails have limited relative precision and may
/// round to zero. Negative inputs use erfc(-x)=2-erfc(x); infinities return
/// 0 or 2, and NaN propagates. See `docs/production-models.md` for provenance.
#[must_use]
pub fn erfc(x: f64) -> f64 {
    if x < 0.0 {
        return 2.0 - erfc(-x);
    }
    if x.is_nan() {
        return f64::NAN;
    }
    if x * x == f64::INFINITY {
        return 0.0;
    }
    log_erfc_positive(x).map_or(f64::NAN, f64::exp)
}

/// Gaussian upper-tail probability Q(x) = 0.5 * erfc(x / sqrt(2)).
///
/// NaN propagates; Q(+infinity)=0 and Q(-infinity)=1. Small tails are
/// exponentiated only once, avoiding premature subnormal rounding of erfc.
#[must_use]
pub fn q_function(x: f64) -> f64 {
    if x < 0.0 {
        return 1.0 - q_function(-x);
    }
    if x.is_nan() {
        return f64::NAN;
    }
    if (x / std::f64::consts::SQRT_2).powi(2) == f64::INFINITY {
        return 0.0;
    }
    log_q_positive(x).map_or(f64::NAN, f64::exp)
}

/// Inverse Gaussian upper tail: the finite x for which Q(x)=probability.
///
/// Requires a finite probability strictly between 0 and 1; endpoints are
/// rejected, not mapped to infinities. Supports every positive f64 probability,
/// including subnormals. Symmetry handles probabilities above 1/2; bisection
/// compares logarithms so no underflowed forward probability is inverted.
/// Numerical accuracy is limited by f64 and the tail kernel, not physical data.
///
/// # Errors
/// Returns `InvalidInput` outside the open probability domain, or
/// `NumericalFailure` if the Gaussian evaluation or solver fails.
pub fn inverse_q_function(probability: f64) -> Result<f64, ModelError> {
    if !probability.is_finite() || probability <= 0.0 || probability >= 1.0 {
        return Err(ModelError::InvalidInput("probability"));
    }
    if probability == 0.5 {
        return Ok(0.0);
    }
    let (tail, sign) = if probability > 0.5 {
        (1.0 - probability, -1.0)
    } else {
        (probability, 1.0)
    };
    // Q(40) is below the least positive f64. Every valid tail is bracketed.
    let log_target = tail.ln();
    let mut lo = 0.0;
    let mut hi = 40.0;
    for _ in 0..200 {
        let mid = (lo + hi) * 0.5;
        if mid == lo || mid == hi {
            return Ok(sign * mid);
        }
        let value = log_q_positive(mid)?;
        if value == log_target {
            return Ok(sign * mid);
        }
        if value > log_target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Err(ModelError::NumericalFailure("inverse Q convergence"))
}

/// Ideal coherent BPSK and Gray-QPSK BER in AWGN.
///
/// BER = Q(sqrt(2 * Eb/No))
/// A unit-energy matched filter yields means ±sqrt(Eb) and noise variance N0/2.
/// Crossing the zero threshold is therefore this Gaussian tail. Gray QPSK has
/// two orthogonal decisions with the same energy per bit.
#[doc(alias = "BER")]
#[must_use]
pub fn ber_bpsk(eb_no_linear: f64) -> f64 {
    q_function((2.0 * eb_no_linear).sqrt())
}

/// BER for M-PSK (M >= 4, Gray coded).
///
/// BER ≈ (2/k) * Q(sqrt(2*k*Eb/No) * sin(π/M)), where k = log2(M).
/// Each adjacent decision boundary is sqrt(Es)*sin(π/M) from the symbol.
/// Count two boundary crossings and one Gray-bit error per nearest neighbor.
/// The higher-order approximation is useful at low BER; BPSK is handled exactly
/// and M=4 reduces to the ideal QPSK bit curve. Orders are not validated.
#[doc(alias = "BER")]
#[must_use]
pub fn ber_mpsk(eb_no_linear: f64, m: u32) -> f64 {
    if m == 2 {
        return ber_bpsk(eb_no_linear);
    }
    let k = (m as f64).log2();
    let sin_term = (PI / m as f64).sin();
    (2.0 / k) * q_function((2.0 * k * eb_no_linear).sqrt() * sin_term)
}

/// Approximate BER for square M-QAM (Gray coded, M = 4, 16, 64, 256, …).
///
/// BER ≈ (4/k) * (1 - 1/√M) * Q(sqrt(3*k*Eb/No / (M-1))).
/// For M=4 (QPSK), this reduces to the QPSK formula.
/// With axis spacing 2d, averaging squared levels gives Es = 2*d²*(M-1)/3.
/// The coefficient counts nearest boundaries; dividing by k assumes one bit
/// error per adjacent decision. Non-square orders such as 32 are evaluated but
/// lie outside this derivation. See `docs/physics-communications.md`.
#[doc(alias = "BER")]
#[must_use]
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

/// BER for any supported modulation type.
#[doc(alias = "BER")]
#[must_use]
pub fn ber(eb_no_linear: f64, modulation: &Modulation) -> f64 {
    match modulation {
        Modulation::Bpsk => ber_bpsk(eb_no_linear),
        Modulation::Qpsk => ber_bpsk(eb_no_linear),
        Modulation::Mpsk(m) => ber_mpsk(eb_no_linear, *m),
        Modulation::Mqam(m) => ber_mqam(eb_no_linear, *m),
        // This selects the coherent precoded-MSK model, not every MSK detector.
        Modulation::Msk => ber_bpsk(eb_no_linear),
    }
}

/// BER from Eb/No in dB.
#[doc(alias = "BER")]
#[doc(alias = "Eb/N0")]
#[must_use]
pub fn ber_from_db(eb_no_db: f64, modulation: &Modulation) -> f64 {
    let eb_no_linear = 10.0_f64.powf(eb_no_db / 10.0);
    ber(eb_no_linear, modulation)
}

/// Link margin in dB: actual Eb/No minus required Eb/No for a target BER.
///
/// Positive = link closes with headroom. Negative = link does not close.
#[doc(alias = "link margin")]
#[must_use]
pub fn link_margin_db(
    actual_eb_no_db: f64,
    target_ber: f64,
    modulation: &Modulation,
) -> Option<f64> {
    let required = required_eb_no_db(target_ber, modulation)?;
    Some(actual_eb_no_db - required)
}

/// Natural logarithm of uncoded BER from finite Eb/No in dB.
///
/// Uses the same coherent AWGN models as [`ber_from_db`], but evaluates the
/// Gaussian logarithm directly even when linear BER underflows. BPSK, QPSK,
/// and coherent precoded MSK use the ideal bit curve. Higher-order PSK/QAM
/// remain low-BER nearest-neighbor approximations, not decoder predictions.
///
/// # Errors
/// Returns `InvalidInput` for nonfinite Eb/No, `UnsupportedRegime` unless
/// M-PSK has power-of-two M>=2 or QAM has square power-of-two M>=4, and
/// `NumericalFailure` if the logarithm cannot be represented as finite f64
/// or the numerical evaluation fails. No FEC accuracy is implied.
pub fn log_ber_from_db(eb_no_db: f64, modulation: &Modulation) -> Result<f64, ModelError> {
    validate_constellation(modulation)?;
    log_ber_unchecked_constellation(eb_no_db, modulation)
}

fn validate_constellation(modulation: &Modulation) -> Result<(), ModelError> {
    match modulation {
        Modulation::Mpsk(m) if *m < 2 || !m.is_power_of_two() => {
            Err(ModelError::UnsupportedRegime("power-of-two PSK order >= 2"))
        }
        Modulation::Mqam(m) if *m < 4 || !m.is_power_of_two() || m.trailing_zeros() % 2 != 0 => {
            Err(ModelError::UnsupportedRegime(
                "square power-of-two QAM order >= 4",
            ))
        }
        _ => Ok(()),
    }
}

// The legacy linear APIs deliberately retain their unvalidated constellation
// behavior. Keep their coefficient/argument geometry available to the inverse.
fn log_ber_unchecked_constellation(
    eb_no_db: f64,
    modulation: &Modulation,
) -> Result<f64, ModelError> {
    if !eb_no_db.is_finite() {
        return Err(ModelError::InvalidInput("eb_no_db"));
    }
    let (coefficient, scale) = match modulation {
        Modulation::Mpsk(m) if *m != 2 => {
            let k = (*m as f64).log2();
            (2.0 / k, (2.0 * k).sqrt() * (PI / *m as f64).sin())
        }
        Modulation::Mqam(m) if *m != 4 => {
            let k = (*m as f64).log2();
            (
                4.0 / k * (1.0 - 1.0 / (*m as f64).sqrt()),
                (3.0 * k / (*m as f64 - 1.0)).sqrt(),
            )
        }
        _ => (1.0, std::f64::consts::SQRT_2),
    };
    if !coefficient.is_finite() || coefficient <= 0.0 || !scale.is_finite() || scale <= 0.0 {
        return Err(ModelError::NumericalFailure("BER geometry"));
    }
    let argument = (eb_no_db * (std::f64::consts::LN_10 / 20.0) + scale.ln()).exp();
    let result = coefficient.ln() + log_q_positive(argument)?;
    if result.is_finite() {
        Ok(result)
    } else {
        Err(ModelError::NumericalFailure("BER logarithm"))
    }
}

/// Required Eb/No in dB for a finite, strictly negative natural-log BER target.
///
/// Supports targets below linear f64 underflow without exponentiating them.
/// The inclusive supported search bracket remains [-5, 50] dB. Constellation
/// restrictions and physical approximations are those of [`log_ber_from_db`].
///
/// # Errors
/// Returns `InvalidInput` for nonfinite or nonnegative log targets,
/// `UnsupportedRegime` for an invalid constellation or an unbracketed target,
/// and `NumericalFailure` if evaluation or bisection fails.
pub fn required_eb_no_from_log_ber(
    log_target_ber: f64,
    modulation: &Modulation,
) -> Result<f64, ModelError> {
    if !log_target_ber.is_finite() || log_target_ber >= 0.0 {
        return Err(ModelError::InvalidInput("log_target_ber"));
    }
    validate_constellation(modulation)?;
    invert_log_ber(log_target_ber, modulation)
}

fn invert_log_ber(log_target: f64, modulation: &Modulation) -> Result<f64, ModelError> {
    let mut lo = -5.0;
    let mut hi = 50.0;
    let at_lo = log_ber_unchecked_constellation(lo, modulation)?;
    let at_hi = log_ber_unchecked_constellation(hi, modulation)?;
    if log_target > at_lo || log_target < at_hi {
        return Err(ModelError::UnsupportedRegime(
            "BER target outside [-5, 50] dB",
        ));
    }
    if log_target == at_lo {
        return Ok(lo);
    }
    if log_target == at_hi {
        return Ok(hi);
    }
    for _ in 0..200 {
        let mid = (lo + hi) * 0.5;
        if mid == lo || mid == hi {
            return Ok(mid);
        }
        let value = log_ber_unchecked_constellation(mid, modulation)?;
        if value == log_target {
            return Ok(mid);
        }
        if value > log_target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Err(ModelError::NumericalFailure("BER inverse convergence"))
}

/// Required Eb/No (dB) for a target BER, found by log-domain bisection.
///
/// Preserves the legacy inclusive [-5, 50] dB bracket and `None` rejection for
/// nonfinite/nonpositive targets, nonfinite endpoint BER, or unbracketed targets.
/// Constellations are not validated here (including the legacy non-square QAM
/// presets); use [`required_eb_no_from_log_ber`] for a checked model. The search
/// terminates at an exact log residual or f64 bracket resolution, with at
/// most 200 iterations.
#[doc(alias = "Eb/N0")]
#[must_use]
pub fn required_eb_no_db(target_ber: f64, modulation: &Modulation) -> Option<f64> {
    if !target_ber.is_finite() || target_ber <= 0.0 {
        return None;
    }
    let at_lo = ber_from_db(-5.0, modulation);
    let at_hi = ber_from_db(50.0, modulation);
    if !at_lo.is_finite() || !at_hi.is_finite() || target_ber > at_lo || target_ber < at_hi {
        return None;
    }
    if target_ber == at_lo {
        return Some(-5.0);
    }
    if target_ber == at_hi {
        return Some(50.0);
    }
    invert_log_ber(target_ber.ln(), modulation).ok()
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
    fn required_eb_no_rejects_invalid_or_unreachable_targets() {
        let modulation = Modulation::Qpsk;
        for target in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 0.0, -1e-5, 0.5] {
            assert_eq!(required_eb_no_db(target, &modulation), None);
        }
        let target = 1e-5;
        let required = required_eb_no_db(target, &modulation).unwrap();
        assert!(required.is_finite());
        assert!(((ber_from_db(required, &modulation) - target) / target).abs() < 1e-6);
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
        assert!(
            ber_64 > ber_16,
            "64-QAM should have higher BER than 16-QAM at same Eb/No"
        );
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
        assert!(
            eb_no > 13.0 && eb_no < 16.0,
            "Expected ~14.4 dB, got {}",
            eb_no
        );
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
        assert!(
            ber_8psk > ber_qpsk,
            "8-PSK should have higher BER than QPSK"
        );
    }
}
