//! ADC quantization noise and ENOB calculations.
//!
//! These use the full-scale sine model and rounded coefficients from
//! `10 * log10((3/2) * 2^(2N))`. The derivation and measurement assumptions
//! are in `docs/physics-geometry-and-errors.md`.

/// Quantization SNR for an ideal ADC.
///
/// Returns SNR in dB for a given number of bits.
/// Assumes uniform quantization error, no clipping or other impairments, and
/// total quantization noise over the Nyquist bandwidth. Practical use requires
/// a positive resolution; the function does not validate the physical model.
#[doc(alias = "SNR")]
#[must_use]
pub fn quantization_snr_db(bits: u32) -> f64 {
    6.02 * bits as f64 + 1.76
}

/// Effective number of bits (ENOB) from measured SNR in dB.
///
/// Inverts the ideal full-scale sine formula. Conventional ADC ENOB uses SINAD
/// (noise plus distortion); noise-only SNR gives an equivalent noise resolution.
/// This helper does not correct input backoff or measurement bandwidth.
#[must_use]
pub fn enob_from_snr(snr_db: f64) -> f64 {
    (snr_db - 1.76) / 6.02
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snr_8_bit() {
        let snr = quantization_snr_db(8);
        assert!((snr - 49.92).abs() < 1e-10);
    }

    #[test]
    fn snr_12_bit() {
        let snr = quantization_snr_db(12);
        assert!((snr - 74.0).abs() < 1e-10);
    }

    #[test]
    fn snr_16_bit() {
        let snr = quantization_snr_db(16);
        assert!((snr - 98.08).abs() < 1e-10);
    }

    #[test]
    fn enob_roundtrip() {
        let bits = 12;
        let snr = quantization_snr_db(bits);
        let enob = enob_from_snr(snr);
        assert!((enob - bits as f64).abs() < 1e-10);
    }
}
