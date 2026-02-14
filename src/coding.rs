//! Forward Error Correction (FEC) coding gain and coded modulation.
//!
//! Provides a `CodedModulation` struct that combines a modulation scheme with
//! an FEC code rate and coding gain, enabling throughput and coded performance
//! calculations.
//!
//! ## Coding Gain
//!
//! Coding gain is the reduction in required Eb/No (at a given BER) achieved by
//! adding FEC redundancy. It comes at the cost of reduced throughput (by the
//! code rate R) and increased bandwidth or complexity.
//!
//! ```text
//! required_Eb/No_coded = required_Eb/No_uncoded - coding_gain
//! ```
//!
//! ## Common Coding Gains (at BER = 1e-5)
//!
//! | Code | Rate | Coding Gain |
//! |------|------|-------------|
//! | Convolutional (K=7) | 1/2 | ~5.0 dB |
//! | Convolutional (K=7) | 3/4 | ~3.5 dB |
//! | Turbo | 1/2 | ~7.5 dB |
//! | Turbo | 3/4 | ~5.5 dB |
//! | LDPC (DVB-S2) | 1/2 | ~8.0 dB |
//! | LDPC (DVB-S2) | 3/4 | ~6.5 dB |
//! | LDPC (DVB-S2) | 9/10 | ~5.0 dB |
//!
//! ## References
//!
//! - Proakis, J. (1995). *Digital Communications* (3rd ed.). McGraw-Hill.
//! - ETSI EN 302 307 — DVB-S2 standard (LDPC + BCH coding)
//! - Berrou, Glavieux, Thitimajshima (1993). "Near Shannon limit error-correcting coding and decoding: Turbo-codes"

use crate::ber;
use crate::modulation::Modulation;

// ---------------------------------------------------------------------------
// Coding gain constants (approximate, at BER ≈ 1e-5 unless noted)
// ---------------------------------------------------------------------------

/// Convolutional code, rate 1/2, constraint length K=7
pub const CODING_GAIN_CONV_R12_K7: f64 = 5.0;
/// Convolutional code, rate 3/4, constraint length K=7
pub const CODING_GAIN_CONV_R34_K7: f64 = 3.5;
/// Turbo code, rate 1/2
pub const CODING_GAIN_TURBO_R12: f64 = 7.5;
/// Turbo code, rate 3/4
pub const CODING_GAIN_TURBO_R34: f64 = 5.5;
/// LDPC (DVB-S2), rate 1/2
pub const CODING_GAIN_LDPC_R12: f64 = 8.0;
/// LDPC (DVB-S2), rate 2/3
pub const CODING_GAIN_LDPC_R23: f64 = 7.0;
/// LDPC (DVB-S2), rate 3/4
pub const CODING_GAIN_LDPC_R34: f64 = 6.5;
/// LDPC (DVB-S2), rate 5/6
pub const CODING_GAIN_LDPC_R56: f64 = 5.5;
/// LDPC (DVB-S2), rate 9/10
pub const CODING_GAIN_LDPC_R910: f64 = 5.0;

// ---------------------------------------------------------------------------
// FEC type enum
// ---------------------------------------------------------------------------

/// Common FEC code families with typical coding gains
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FecCode {
    /// No FEC (uncoded)
    Uncoded,
    /// Convolutional code, constraint length K=7
    Convolutional { rate: f64 },
    /// Turbo code
    Turbo { rate: f64 },
    /// LDPC code (e.g. DVB-S2)
    Ldpc { rate: f64 },
    /// Custom FEC with explicit code rate and coding gain
    Custom { rate: f64, coding_gain_db: f64 },
}

impl FecCode {
    /// Code rate R (ratio of information bits to total bits)
    pub fn rate(&self) -> f64 {
        match self {
            FecCode::Uncoded => 1.0,
            FecCode::Convolutional { rate } => *rate,
            FecCode::Turbo { rate } => *rate,
            FecCode::Ldpc { rate } => *rate,
            FecCode::Custom { rate, .. } => *rate,
        }
    }

    /// Approximate coding gain in dB at BER ≈ 1e-5.
    ///
    /// For convolutional, turbo, and LDPC codes, the gain is interpolated
    /// between known rate/gain pairs. For custom codes, the explicit gain is used.
    pub fn coding_gain_db(&self) -> f64 {
        match self {
            FecCode::Uncoded => 0.0,
            FecCode::Convolutional { rate } => {
                // Interpolate between rate 1/2 (5.0 dB) and rate 3/4 (3.5 dB)
                lerp_gain(*rate, 0.5, CODING_GAIN_CONV_R12_K7, 0.75, CODING_GAIN_CONV_R34_K7)
            }
            FecCode::Turbo { rate } => {
                lerp_gain(*rate, 0.5, CODING_GAIN_TURBO_R12, 0.75, CODING_GAIN_TURBO_R34)
            }
            FecCode::Ldpc { rate } => {
                // Piecewise interpolation across DVB-S2 rate points
                if *rate <= 0.5 {
                    CODING_GAIN_LDPC_R12
                } else if *rate <= 2.0 / 3.0 {
                    lerp_gain(*rate, 0.5, CODING_GAIN_LDPC_R12, 2.0 / 3.0, CODING_GAIN_LDPC_R23)
                } else if *rate <= 0.75 {
                    lerp_gain(*rate, 2.0 / 3.0, CODING_GAIN_LDPC_R23, 0.75, CODING_GAIN_LDPC_R34)
                } else if *rate <= 5.0 / 6.0 {
                    lerp_gain(*rate, 0.75, CODING_GAIN_LDPC_R34, 5.0 / 6.0, CODING_GAIN_LDPC_R56)
                } else {
                    lerp_gain(*rate, 5.0 / 6.0, CODING_GAIN_LDPC_R56, 0.9, CODING_GAIN_LDPC_R910)
                }
            }
            FecCode::Custom { coding_gain_db, .. } => *coding_gain_db,
        }
    }
}

/// Linear interpolation of coding gain between two rate/gain points.
/// Higher code rate → less redundancy → less coding gain.
fn lerp_gain(rate: f64, r1: f64, g1: f64, r2: f64, g2: f64) -> f64 {
    if (r2 - r1).abs() < 1e-10 {
        return g1;
    }
    let t = ((rate - r1) / (r2 - r1)).clamp(0.0, 1.0);
    g1 + t * (g2 - g1)
}

impl std::fmt::Display for FecCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FecCode::Uncoded => write!(f, "Uncoded"),
            FecCode::Convolutional { rate } => write!(f, "Convolutional (R={})", rate),
            FecCode::Turbo { rate } => write!(f, "Turbo (R={})", rate),
            FecCode::Ldpc { rate } => write!(f, "LDPC (R={})", rate),
            FecCode::Custom { rate, coding_gain_db } => {
                write!(f, "Custom (R={}, gain={} dB)", rate, coding_gain_db)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// CodedModulation
// ---------------------------------------------------------------------------

/// A modulation scheme combined with FEC coding.
///
/// This struct bridges the gap between raw modulation performance and
/// practical system throughput.
#[derive(Debug, Clone)]
pub struct CodedModulation {
    pub modulation: Modulation,
    pub fec: FecCode,
}

impl CodedModulation {
    pub fn new(modulation: Modulation, fec: FecCode) -> Self {
        Self { modulation, fec }
    }

    /// Code rate R
    pub fn code_rate(&self) -> f64 {
        self.fec.rate()
    }

    /// Coding gain in dB
    pub fn coding_gain_db(&self) -> f64 {
        self.fec.coding_gain_db()
    }

    /// Spectral efficiency in bits/s/Hz
    /// η = k × R where k = bits_per_symbol
    pub fn spectral_efficiency(&self) -> f64 {
        self.modulation.spectral_efficiency(self.fec.rate())
    }

    /// Throughput in bits/s for a given channel bandwidth
    /// throughput = BW × η = BW × k × R
    pub fn throughput_bps(&self, bandwidth_hz: f64) -> f64 {
        bandwidth_hz * self.spectral_efficiency()
    }

    /// Required Eb/No (dB) for a target BER, accounting for coding gain.
    ///
    /// ```text
    /// required_Eb/No = uncoded_required - coding_gain
    /// ```
    pub fn required_eb_no_db(&self, target_ber: f64) -> Option<f64> {
        let uncoded = ber::required_eb_no_db(target_ber, &self.modulation)?;
        Some(uncoded - self.fec.coding_gain_db())
    }

    /// BER for a given Eb/No (dB), accounting for coding gain.
    ///
    /// The effective Eb/No seen by the decoder is increased by the coding gain.
    pub fn ber_from_db(&self, eb_no_db: f64) -> f64 {
        let effective_eb_no_db = eb_no_db + self.fec.coding_gain_db();
        ber::ber_from_db(effective_eb_no_db, &self.modulation)
    }

    /// Link margin in dB: actual Eb/No minus required Eb/No for target BER.
    ///
    /// Positive margin means the link closes with headroom.
    /// Negative margin means the link does not close.
    pub fn link_margin_db(&self, actual_eb_no_db: f64, target_ber: f64) -> Option<f64> {
        let required = self.required_eb_no_db(target_ber)?;
        Some(actual_eb_no_db - required)
    }

    /// Symbol rate for a given information bit rate
    pub fn symbol_rate(&self, info_bit_rate_bps: f64) -> f64 {
        self.modulation.symbol_rate(info_bit_rate_bps, self.fec.rate())
    }
}

impl std::fmt::Display for CodedModulation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} + {}", self.modulation, self.fec)
    }
}

// ---------------------------------------------------------------------------
// Convenience constructors for common DVB-S2 ModCods
// ---------------------------------------------------------------------------

/// DVB-S2 QPSK rate 1/2 (LDPC)
pub fn dvbs2_qpsk_r12() -> CodedModulation {
    CodedModulation::new(Modulation::Qpsk, FecCode::Ldpc { rate: 0.5 })
}

/// DVB-S2 QPSK rate 3/4 (LDPC)
pub fn dvbs2_qpsk_r34() -> CodedModulation {
    CodedModulation::new(Modulation::Qpsk, FecCode::Ldpc { rate: 0.75 })
}

/// DVB-S2 8-PSK rate 2/3 (LDPC)
pub fn dvbs2_8psk_r23() -> CodedModulation {
    CodedModulation::new(Modulation::Mpsk(8), FecCode::Ldpc { rate: 2.0 / 3.0 })
}

/// DVB-S2 16-APSK rate 3/4 (LDPC) — modeled as 16-QAM for BER approximation
pub fn dvbs2_16apsk_r34() -> CodedModulation {
    CodedModulation::new(Modulation::Mqam(16), FecCode::Ldpc { rate: 0.75 })
}

/// DVB-S2 32-APSK rate 5/6 (LDPC) — modeled as 32-QAM for BER approximation
pub fn dvbs2_32apsk_r56() -> CodedModulation {
    CodedModulation::new(Modulation::Mqam(32), FecCode::Ldpc { rate: 5.0 / 6.0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uncoded_zero_gain() {
        let fec = FecCode::Uncoded;
        assert_eq!(fec.rate(), 1.0);
        assert_eq!(fec.coding_gain_db(), 0.0);
    }

    #[test]
    fn conv_r12_gain() {
        let fec = FecCode::Convolutional { rate: 0.5 };
        assert!((fec.coding_gain_db() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn conv_r34_gain() {
        let fec = FecCode::Convolutional { rate: 0.75 };
        assert!((fec.coding_gain_db() - 3.5).abs() < 1e-10);
    }

    #[test]
    fn turbo_r12_gain() {
        let fec = FecCode::Turbo { rate: 0.5 };
        assert!((fec.coding_gain_db() - 7.5).abs() < 1e-10);
    }

    #[test]
    fn ldpc_r12_gain() {
        let fec = FecCode::Ldpc { rate: 0.5 };
        assert!((fec.coding_gain_db() - 8.0).abs() < 1e-10);
    }

    #[test]
    fn ldpc_r34_gain() {
        let fec = FecCode::Ldpc { rate: 0.75 };
        assert!((fec.coding_gain_db() - 6.5).abs() < 1e-10);
    }

    #[test]
    fn ldpc_r910_gain() {
        let fec = FecCode::Ldpc { rate: 0.9 };
        assert!((fec.coding_gain_db() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn ldpc_interpolated_gain() {
        // Rate 0.625 (between 1/2 and 2/3) should interpolate
        let fec = FecCode::Ldpc { rate: 0.625 };
        let gain = fec.coding_gain_db();
        assert!(gain > 7.0 && gain < 8.0,
            "Expected interpolated gain between 7.0 and 8.0, got {}", gain);
    }

    #[test]
    fn custom_fec() {
        let fec = FecCode::Custom { rate: 0.8, coding_gain_db: 6.0 };
        assert_eq!(fec.rate(), 0.8);
        assert_eq!(fec.coding_gain_db(), 6.0);
    }

    #[test]
    fn coded_modulation_spectral_efficiency() {
        // QPSK (k=2) rate 3/4 → η = 2 × 0.75 = 1.5 bits/s/Hz
        let cm = CodedModulation::new(Modulation::Qpsk, FecCode::Ldpc { rate: 0.75 });
        assert!((cm.spectral_efficiency() - 1.5).abs() < 1e-10);
    }

    #[test]
    fn coded_modulation_throughput() {
        // QPSK rate 3/4, 36 MHz channel → 36e6 × 1.5 = 54 Mbps
        let cm = dvbs2_qpsk_r34();
        let tp = cm.throughput_bps(36e6);
        assert!((tp - 54e6).abs() < 1.0);
    }

    #[test]
    fn coded_requires_less_eb_no() {
        let uncoded = CodedModulation::new(Modulation::Qpsk, FecCode::Uncoded);
        let coded = dvbs2_qpsk_r12();

        let req_uncoded = uncoded.required_eb_no_db(1e-5).unwrap();
        let req_coded = coded.required_eb_no_db(1e-5).unwrap();

        assert!(req_coded < req_uncoded,
            "Coded should require less Eb/No: coded={:.1}, uncoded={:.1}", req_coded, req_uncoded);
        let diff = req_uncoded - req_coded;
        assert!((diff - 8.0).abs() < 0.5,
            "LDPC R=1/2 coding gain should be ~8 dB, got {:.1} dB", diff);
    }

    #[test]
    fn coded_ber_better_than_uncoded() {
        let uncoded = CodedModulation::new(Modulation::Qpsk, FecCode::Uncoded);
        let coded = dvbs2_qpsk_r34();

        let eb_no = 5.0; // dB
        let ber_uncoded = uncoded.ber_from_db(eb_no);
        let ber_coded = coded.ber_from_db(eb_no);

        assert!(ber_coded < ber_uncoded,
            "Coded BER should be lower: coded={:.2e}, uncoded={:.2e}", ber_coded, ber_uncoded);
    }

    #[test]
    fn link_margin_positive() {
        let cm = dvbs2_qpsk_r34();
        let margin = cm.link_margin_db(10.0, 1e-5).unwrap();
        assert!(margin > 0.0, "10 dB Eb/No should close with QPSK R=3/4 at BER=1e-5");
    }

    #[test]
    fn link_margin_negative() {
        let cm = CodedModulation::new(Modulation::Mqam(64), FecCode::Uncoded);
        let margin = cm.link_margin_db(5.0, 1e-6).unwrap();
        assert!(margin < 0.0, "5 dB Eb/No should NOT close for uncoded 64-QAM at BER=1e-6");
    }

    #[test]
    fn dvbs2_presets_display() {
        assert_eq!(format!("{}", dvbs2_qpsk_r12()), "QPSK + LDPC (R=0.5)");
        assert_eq!(format!("{}", dvbs2_8psk_r23()), "8-PSK + LDPC (R=0.6666666666666666)");
    }

    #[test]
    fn symbol_rate_coded() {
        // QPSK (k=2), rate 3/4, 54 Mbps info rate
        // Rc = 54e6 / 0.75 = 72e6 coded bps
        // Rs = 72e6 / 2 = 36e6 symbols/s
        let cm = dvbs2_qpsk_r34();
        let rs = cm.symbol_rate(54e6);
        assert!((rs - 36e6).abs() < 1.0);
    }
}
