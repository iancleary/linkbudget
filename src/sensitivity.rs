//! Receiver sensitivity calculator.
//!
//! Estimates input power for a target BER at the 290 K noise-figure reference.
//! The BER helpers use an uncoded threshold and ignore their code-rate argument;
//! they do not predict the effect of an FEC decoder. Derivations and limitations
//! are in `docs/physics-communications.md`.
//!
//! ## Roll-Off Factor (α)
//!
//! The roll-off factor α (alpha) is a parameter of the raised-cosine pulse shaping
//! filter used in digital communications to minimize intersymbol interference (ISI).
//! It determines the **excess bandwidth** beyond the theoretical Nyquist minimum:
//!
//! - **Occupied bandwidth:** `BW = Rs × (1 + α)` where Rs is the symbol rate
//! - **Nyquist minimum bandwidth:** `BW_min = Rs` (α = 0, brick-wall filter — impractical)
//! - **Excess bandwidth:** `BW_excess = α × Rs`
//!
//! Common values:
//! - α = 0.20 — DVB-S2, tight spectral efficiency
//! - α = 0.25 — some satellite systems
//! - α = 0.35 — legacy DVB-S, many satcom modems
//! - α = 0.50 — relaxed filtering, simpler implementation
//! - α = 1.00 — maximum excess bandwidth, no flat part of the raised cosine
//!
//! ### Impact on sensitivity
//!
//! For ideal root-raised-cosine TX/RX filters, equivalent noise bandwidth in the
//! full RF-width convention is Rs regardless of α: the squared receive response
//! is raised cosine, whose normalized area is Rs. More generally, substituting
//! `SNR_required = (Eb/N0)_required * Rb / Bn` into `C_min = N0 * Bn * SNR_required`
//! cancels Bn. Ideal sensitivity depends on required energy, not bandwidth labels.
//!
//! Roll-off still changes spectral allocation and practical filter design. This
//! module's bandpass helper assumes a noise-width increase by `1 + α` at fixed
//! useful decision signal and required decision SNR. It therefore adds
//! `10*log10(1 + α)` dB as an explicit loss model. That value is neither a universal
//! non-matched-receiver penalty nor a worst-case bound.
//!
//! Real mismatch can change signal energy, ISI, and noise together; those effects
//! require a receiver model. Merely changing Bn in a consistent SNR conversion
//! does not create the bandpass helper's penalty.
//!
//! ## References
//!
//! - [MathWorks raised-cosine filtering](https://www.mathworks.com/help/comm/ug/raised-cosine-filtering.html)
//! - [MathWorks energy-ratio conversions](https://www.mathworks.com/help/comm/ref/convertsnr.html)

use crate::ber;
use crate::modulation::Modulation;

/// Thermal noise floor in dBm for a given bandwidth.
///
/// `N = -174 dBm/Hz + 10·log10(BW_Hz) + NF_dB`
/// This is input-referred `k*T0*F*Bn`, with a 290 K source and noise factor F.
/// The constant rounds -173.98 dBm/Hz to -174; BW is equivalent noise bandwidth.
#[must_use]
pub fn noise_floor_dbm(bandwidth_hz: f64, noise_figure_db: f64) -> f64 {
    -174.0 + 10.0 * bandwidth_hz.log10() + noise_figure_db
}

/// Receiver sensitivity in dBm assuming a **matched filter** (root-raised-cosine).
///
/// Since `Eb/N0 = C/(Rb*N0)`, the required power is `N0*Rb*(Eb/N0)_required`.
/// With a 290 K source, `N0 = k*T0*F`. Noise bandwidth cancels when required SNR
/// and integrated noise use the same bandwidth. See `docs/physics-communications.md`.
/// The current threshold is uncoded, so a nonunit code rate does not establish
/// coded-link sensitivity; the function ignores this parameter.
///
/// ```text
/// Sensitivity = -174 + NF + Eb/No_required + 10·log10(Rb) + impl_loss
/// ```
///
/// # Arguments
/// * `modulation` - Modulation scheme
/// * `info_bit_rate_bps` - Information (payload) bit rate Rb
/// * `code_rate` - Accepted for compatibility but ignored; no FEC gain is applied
/// * `noise_figure_db` - Receiver noise figure in dB
/// * `target_ber` - Required BER (e.g. 1e-6)
/// * `implementation_loss_db` - Additional loss margin (modem imperfections, etc.)
///
/// # Returns
/// Minimum input power in dBm to achieve the target BER.
#[must_use]
pub fn sensitivity_matched_filter_dbm(
    modulation: &Modulation,
    info_bit_rate_bps: f64,
    _code_rate: f64,
    noise_figure_db: f64,
    target_ber: f64,
    implementation_loss_db: f64,
) -> Option<f64> {
    let required_eb_no_db = ber::required_eb_no_db(target_ber, modulation)?;

    // C_min = k*T0*F*Rb*(Eb/N0)_required; -174 rounds k*T0 at 290 K.
    // This uses an uncoded BER threshold; _code_rate does not model a decoder.
    let sensitivity = -174.0
        + noise_figure_db
        + required_eb_no_db
        + 10.0 * info_bit_rate_bps.log10()
        + implementation_loss_db;

    tracing::debug!(
        required_eb_no_db,
        noise_figure_db,
        info_bit_rate_bps,
        implementation_loss_db,
        sensitivity_dbm = sensitivity,
        "Matched filter sensitivity"
    );

    Some(sensitivity)
}

/// Receiver sensitivity in dBm with an assumed bandpass noise-width penalty.
///
/// Assumes noise increases in proportion to width `Rs × (1 + α)`, while useful
/// decision signal and required decision SNR remain fixed. This adds the width
/// ratio in dB. It is not a general mismatch calculation or a worst-case bound;
/// changing bandwidth in a consistent energy/SNR conversion instead cancels.
///
/// ```text
/// Sensitivity = -174 + NF + Eb/No_required + 10·log10(Rb) + 10·log10(1+α) + impl_loss
/// ```
///
/// # Arguments
/// * `modulation` - Modulation scheme
/// * `info_bit_rate_bps` - Information (payload) bit rate Rb
/// * `code_rate` - Passed to the matched-filter helper, which ignores it
/// * `noise_figure_db` - Receiver noise figure in dB
/// * `target_ber` - Required BER (e.g. 1e-6)
/// * `implementation_loss_db` - Additional loss margin
/// * `rolloff` - Raised-cosine roll-off factor α (0.0 to 1.0)
///
/// # Returns
/// Minimum input power in dBm to achieve the target BER.
#[must_use]
pub fn sensitivity_bandpass_dbm(
    modulation: &Modulation,
    info_bit_rate_bps: f64,
    code_rate: f64,
    noise_figure_db: f64,
    target_ber: f64,
    implementation_loss_db: f64,
    rolloff: f64,
) -> Option<f64> {
    let matched = sensitivity_matched_filter_dbm(
        modulation,
        info_bit_rate_bps,
        code_rate,
        noise_figure_db,
        target_ber,
        implementation_loss_db,
    )?;

    // Assumed fixed-decision-SNR loss, not an automatic consequence of roll-off.
    let rolloff_penalty_db = 10.0 * (1.0 + rolloff).log10();

    Some(matched + rolloff_penalty_db)
}

/// Legacy wrapper — calls [`sensitivity_matched_filter_dbm`].
///
/// The `rolloff` parameter is accepted but ignored (matched filter assumption).
/// Prefer [`sensitivity_matched_filter_dbm`] or [`sensitivity_bandpass_dbm`] directly.
#[must_use]
pub fn sensitivity_dbm(
    modulation: &Modulation,
    info_bit_rate_bps: f64,
    code_rate: f64,
    noise_figure_db: f64,
    target_ber: f64,
    implementation_loss_db: f64,
    _rolloff: f64,
) -> Option<f64> {
    sensitivity_matched_filter_dbm(
        modulation,
        info_bit_rate_bps,
        code_rate,
        noise_figure_db,
        target_ber,
        implementation_loss_db,
    )
}

/// Assumed roll-off noise-width penalty in dB.
///
/// Divides assumed noise widths `Rs × (1 + α)` and `Rs`, then takes 10*log10.
/// Applying this to sensitivity assumes unchanged useful decision signal and
/// required decision SNR; see [`sensitivity_bandpass_dbm`] for the model limits.
///
/// ```text
/// penalty = 10·log10(1 + α)
/// ```
///
/// | α    | Penalty |
/// |------|---------|
/// | 0.00 | 0.00 dB |
/// | 0.20 | 0.79 dB |
/// | 0.25 | 0.97 dB |
/// | 0.35 | 1.30 dB |
/// | 0.50 | 1.76 dB |
/// | 1.00 | 3.01 dB |
#[must_use]
pub fn rolloff_penalty_db(rolloff: f64) -> f64 {
    10.0 * (1.0 + rolloff).log10()
}

/// Simplified sensitivity: just noise floor + required SNR.
///
/// For quick estimates when you know the required SNR directly.
/// The SNR threshold must use the same equivalent noise bandwidth as the noise
/// floor. A power threshold is noise power multiplied by required linear SNR.
#[must_use]
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
    fn sensitivity_matched_bpsk_1mbps() {
        // BPSK, 1 Mbps, rate 1 (uncoded), NF=3 dB, BER=1e-5, 0 dB impl loss
        // Required Eb/No ≈ 9.6 dB
        // Sensitivity ≈ -174 + 3 + 9.6 + 60 = -101.4 dBm
        let sens =
            sensitivity_matched_filter_dbm(&Modulation::Bpsk, 1e6, 1.0, 3.0, 1e-5, 0.0).unwrap();
        assert!(
            sens > -103.0 && sens < -100.0,
            "Expected ~-101.4 dBm, got {}",
            sens
        );
    }

    #[test]
    fn sensitivity_bandpass_worse_than_matched() {
        let matched =
            sensitivity_matched_filter_dbm(&Modulation::Qpsk, 10e6, 0.75, 3.0, 1e-6, 0.0).unwrap();
        let bandpass =
            sensitivity_bandpass_dbm(&Modulation::Qpsk, 10e6, 0.75, 3.0, 1e-6, 0.0, 0.35).unwrap();
        // Bandpass should be worse (higher power needed) by ~1.3 dB
        assert!(
            bandpass > matched,
            "Bandpass sensitivity should be worse than matched filter"
        );
        let diff = bandpass - matched;
        assert!(
            (diff - 1.303).abs() < 0.01,
            "Expected ~1.3 dB penalty for α=0.35, got {:.3} dB",
            diff
        );
    }

    #[test]
    fn rolloff_zero_no_penalty() {
        let penalty = rolloff_penalty_db(0.0);
        assert!((penalty - 0.0).abs() < 1e-10);
    }

    #[test]
    fn rolloff_one_3db_penalty() {
        // α=1.0 → 10·log10(2) ≈ 3.01 dB
        let penalty = rolloff_penalty_db(1.0);
        assert!((penalty - 3.0103).abs() < 0.001);
    }

    #[test]
    fn rolloff_035_penalty() {
        // α=0.35 → 10·log10(1.35) ≈ 1.303 dB
        let penalty = rolloff_penalty_db(0.35);
        assert!((penalty - 1.303).abs() < 0.01);
    }

    #[test]
    fn rolloff_penalty_table() {
        // Verify the table in the doc comment
        let cases = vec![
            (0.00, 0.00),
            (0.20, 0.79),
            (0.25, 0.97),
            (0.35, 1.30),
            (0.50, 1.76),
            (1.00, 3.01),
        ];
        for (alpha, expected) in cases {
            let penalty = rolloff_penalty_db(alpha);
            assert!(
                (penalty - expected).abs() < 0.01,
                "α={}: expected {:.2} dB, got {:.2} dB",
                alpha,
                expected,
                penalty
            );
        }
    }

    #[test]
    fn sensitivity_legacy_wrapper() {
        // Legacy wrapper should match matched-filter result
        let legacy = sensitivity_dbm(&Modulation::Bpsk, 1e6, 1.0, 3.0, 1e-5, 0.0, 0.35).unwrap();
        let matched =
            sensitivity_matched_filter_dbm(&Modulation::Bpsk, 1e6, 1.0, 3.0, 1e-5, 0.0).unwrap();
        assert!((legacy - matched).abs() < 1e-10);
    }

    #[test]
    fn sensitivity_qpsk_10mbps() {
        let sens =
            sensitivity_matched_filter_dbm(&Modulation::Qpsk, 10e6, 0.75, 5.0, 1e-6, 2.0).unwrap();
        assert!(
            sens > -100.0 && sens < -75.0,
            "Expected sensitivity in -100 to -75 dBm range, got {}",
            sens
        );
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
        let sens_1m =
            sensitivity_matched_filter_dbm(&Modulation::Bpsk, 1e6, 1.0, 3.0, 1e-5, 0.0).unwrap();
        let sens_10m =
            sensitivity_matched_filter_dbm(&Modulation::Bpsk, 10e6, 1.0, 3.0, 1e-5, 0.0).unwrap();
        assert!(
            sens_10m > sens_1m,
            "Higher bit rate should require more power"
        );
        assert!(((sens_10m - sens_1m) - 10.0).abs() < 0.5);
    }
}
