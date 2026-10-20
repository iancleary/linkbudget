//! Shannon capacity and second-order finite-blocklength AWGN planning.
//!
//! For Gaussian signal plus independent Gaussian noise, the entropy difference
//! is `0.5*log2(1 + SNR)` per real degree of freedom. A band-limited channel has
//! `2*B` such degrees per second, giving `B*log2(1 + SNR)` bit/s.
//! See `docs/physics-rf-link.md` for the derivation, assumptions, and tests.

use crate::{ber::inverse_q_function, ModelError};
use core::fmt;
use std::fmt::{Display, Formatter};

/// Ideal flat-AWGN channel capacity for a given bandwidth and SNR.
///
/// This is an average-power capacity bound with complete signal capture.
/// It does not include a finite constellation, decoder loss, or packet overhead.
pub struct PhyRate {
    /// Channel bandwidth in Hz; must be finite and positive.
    pub bandwidth: f64,
    /// Signal-to-noise ratio (linear, not dB) over the same channel bandwidth.
    /// Must be finite and nonnegative; use `C/(N0*bandwidth)`.
    pub snr: f64,
}

impl PhyRate {
    /// PHY rate in bits per second (Shannon capacity).
    #[must_use]
    pub fn bps(&self) -> f64 {
        self.bandwidth * (1.0 + self.snr).log2()
    }

    /// PHY rate in megabits per second.
    #[must_use]
    pub fn mbps(&self) -> f64 {
        self.bps() / 1_000_000.0
    }

    /// PHY rate in gigabits per second.
    #[must_use]
    pub fn gbps(&self) -> f64 {
        self.bps() / 1_000_000_000.0
    }
}

impl Display for PhyRate {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "Bandwidth {} Hz\nSNR {} (linear)\nPHY Rate {} Mbps",
            self.bandwidth,
            self.snr,
            self.mbps()
        )
    }
}

/// Second-order AWGN normal-approximation rate in bits per complex channel use.
///
/// For linear SNR `rho`, evaluates `C - sqrt(V/n) * Q_inverse(epsilon)` with
/// `C = log2(1+rho)` and `V = (1-(1+rho)^(-2)) * log2(e)^2`.
/// One complex use groups two real AWGN uses. `n` counts complex uses, not
/// information bits; `epsilon` is block error probability, not bit error rate.
/// There is no logarithmic/third-order correction.
///
/// Based on Polyanskiy, Poor, and Verdu (2010), Theorem 54, equations 291–294,
/// DOI: <https://doi.org/10.1109/TIT.2010.2043769>, normalized to complex uses.
/// This is an unconstrained Gaussian-input ideal AWGN planning approximation
/// in the maximal/equal codeword-power regime, not a finite-constellation
/// decoder model, achieved rate, rigorous upper/lower bound, or decoder BLER.
///
/// Requires finite positive SNR, `n >= 100`, and `1e-9 <= epsilon < 0.5`.
/// These are conservative product guardrails, not published accuracy guarantees;
/// the approximation is not universally accurate within them. The result is
/// not bits/s or modem throughput. Any conversion needs an explicitly justified
/// complex-use rate; noise bandwidth is not automatically that rate.
/// [`PhyRate`] and [`crate::LinkBudget::phy_rate`] retain Shannon semantics.
/// See `docs/production-models.md` for source normalization and evidence.
///
/// # Errors
/// Returns [`ModelError::InvalidInput`] for nonfinite/nonpositive SNR, zero
/// blocklength, or probability outside (0,1); [`ModelError::UnsupportedRegime`]
/// outside the product guardrails or when the estimated rate is nonpositive
/// (no silent clamp); [`ModelError::NumericalFailure`] for nonfinite arithmetic
/// or failed inverse-Q evaluation.
pub fn awgn_finite_blocklength_rate_bits_per_complex_use(
    snr_linear: f64,
    blocklength_complex_uses: u32,
    block_error_probability: f64,
) -> Result<f64, ModelError> {
    if !snr_linear.is_finite() || snr_linear <= 0.0 {
        return Err(ModelError::InvalidInput("snr_linear"));
    }
    if blocklength_complex_uses == 0 {
        return Err(ModelError::InvalidInput("blocklength_complex_uses"));
    }
    if !block_error_probability.is_finite()
        || block_error_probability <= 0.0
        || block_error_probability >= 1.0
    {
        return Err(ModelError::InvalidInput("block_error_probability"));
    }
    if blocklength_complex_uses < 100 {
        return Err(ModelError::UnsupportedRegime(
            "requires at least 100 complex uses",
        ));
    }
    if !(1e-9..0.5).contains(&block_error_probability) {
        return Err(ModelError::UnsupportedRegime(
            "requires 1e-9 <= block error probability < 0.5",
        ));
    }

    let log_one_plus_snr = snr_linear.ln_1p();
    let capacity = log_one_plus_snr * std::f64::consts::LOG2_E;
    // exp_m1 avoids small-SNR cancellation; no squaring of 1+rho can overflow.
    let dispersion = -(-2.0 * log_one_plus_snr).exp_m1() * std::f64::consts::LOG2_E.powi(2);
    let rate = capacity
        - (dispersion.sqrt() / f64::from(blocklength_complex_uses).sqrt())
            * inverse_q_function(block_error_probability)?;
    if !capacity.is_finite() || !dispersion.is_finite() || !rate.is_finite() {
        return Err(ModelError::NumericalFailure("AWGN finite-blocklength rate"));
    }
    if rate <= 0.0 {
        return Err(ModelError::UnsupportedRegime(
            "nonpositive second-order rate",
        ));
    }
    Ok(rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phy_rate() {
        let phy_rate = PhyRate {
            bandwidth: 20_000_000.0,
            snr: 15.0, // to have a clean 2^4
        };
        assert_eq!(phy_rate.bps(), 80_000_000.0);
        assert_eq!(phy_rate.mbps(), 80.0);
        assert_eq!(phy_rate.gbps(), 0.08);
    }

    #[test]
    fn zero_snr_gives_zero_capacity() {
        let phy = PhyRate {
            bandwidth: 1_000_000.0,
            snr: 0.0,
        };
        assert_eq!(phy.bps(), 0.0);
    }

    #[test]
    fn snr_1_gives_bandwidth() {
        // SNR=1 (0 dB) → C = BW * log2(2) = BW
        let bw = 10_000_000.0;
        let phy = PhyRate {
            bandwidth: bw,
            snr: 1.0,
        };
        assert!((phy.bps() - bw).abs() < 1e-6);
    }

    #[test]
    fn high_snr_satellite() {
        // Ka-band satellite: 500 MHz BW, 20 dB SNR (100 linear)
        let phy = PhyRate {
            bandwidth: 500_000_000.0,
            snr: 100.0,
        };
        // log2(101) ≈ 6.658 → ~3.33 Gbps
        assert!((phy.gbps() - 3.329).abs() < 0.01);
    }

    #[test]
    fn display_format() {
        let phy = PhyRate {
            bandwidth: 20_000_000.0,
            snr: 15.0,
        };
        let s = format!("{phy}");
        assert!(s.contains("20000000"));
        assert!(s.contains("15"));
        assert!(s.contains("80"));
    }

    #[test]
    fn mbps_gbps_consistency() {
        let phy = PhyRate {
            bandwidth: 1_000_000_000.0,
            snr: 3.0,
        };
        assert!((phy.mbps() - phy.gbps() * 1000.0).abs() < 1e-6);
        assert!((phy.bps() - phy.mbps() * 1_000_000.0).abs() < 1e-3);
    }
}
