//! Sourced DVB-S2 normal-frame performance profiles, not BER curves.
//!
//! Values are factual reference points from ETSI EN 302 307-1 V1.4.1 (2014-11),
//! section 6, Table 13. No decoder or implementation code is reproduced.
//! See `docs/production-models.md` for conditions, examples, and migration.

use crate::ModelError;

/// Source and operating conditions shared by every profile in this module.
#[derive(Debug, Clone, Copy)]
pub struct ReferenceConditions {
    /// Exact source revision and table.
    pub source: &'static str,
    /// Publisher-hosted source document.
    pub source_url: &'static str,
    /// Coded bits in each normal FECFRAME.
    pub fecframe_bits: u32,
    /// Useful 188-byte transport-stream packet error ratio after FEC.
    pub packet_error_ratio: f64,
    /// Fixed-point LDPC decoder iterations used for the reference simulation.
    pub ldpc_iterations: u32,
    /// Channel and synchronization assumptions, not hardware guarantees.
    pub assumptions: &'static str,
    /// Definition of the tabulated rate efficiency.
    pub efficiency_basis: &'static str,
}

/// Normal-frame, no-pilots reference conditions; short frames are unsupported.
pub const REFERENCE: ReferenceConditions = ReferenceConditions {
    source: "ETSI EN 302 307-1 V1.4.1 (2014-11), section 6, Table 13",
    source_url: "https://www.etsi.org/deliver/etsi_en/302300_302399/30230701/01.04.01_60/en_30230701v010401p.pdf",
    fecframe_bits: 64_800,
    packet_error_ratio: 1e-7,
    ldpc_iterations: 50,
    assumptions: "AWGN; perfect carrier and synchronization recovery; no phase noise",
    efficiency_basis: "system information bits per transmitted symbol; normal frames; no pilots",
};

/// A closed set of sourced normal-frame DVB-S2 performance profiles.
///
/// APSK names identify actual APSK reference modes, not QAM substitutes. These
/// profiles deliberately expose no arbitrary BER target or analytical BER curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// QPSK, nominal LDPC rate 1/2.
    QpskR12,
    /// QPSK, nominal LDPC rate 3/4.
    QpskR34,
    /// 8PSK, nominal LDPC rate 2/3.
    Psk8R23,
    /// 16APSK, nominal LDPC rate 3/4.
    Apsk16R34,
    /// 32APSK, nominal LDPC rate 5/6.
    Apsk32R56,
}

impl Mode {
    /// Ideal required Es/N0 in dB at the fixed [`REFERENCE`] conditions.
    #[must_use]
    pub const fn required_es_no_db(self) -> f64 {
        match self {
            Self::QpskR12 => 1.00,
            Self::QpskR34 => 4.03,
            Self::Psk8R23 => 6.62,
            Self::Apsk16R34 => 10.21,
            Self::Apsk32R56 => 14.28,
        }
    }

    /// System information bits per symbol, including reference framing overhead.
    ///
    /// Not bits per occupied Hz or net application throughput. The table is for
    /// normal frames without pilots; it is not simply log2(M) times LDPC rate.
    #[must_use]
    pub const fn information_bits_per_symbol(self) -> f64 {
        match self {
            Self::QpskR12 => 0.988_858,
            Self::QpskR34 => 1.487_473,
            Self::Psk8R23 => 1.980_636,
            Self::Apsk16R34 => 2.966_728,
            Self::Apsk32R56 => 4.119_540,
        }
    }

    /// Margin against the reference threshold and a nonnegative implementation
    /// allowance, all in dB, at an explicit positive symbol rate (symbols/s).
    pub fn margin_db(
        self,
        c_over_no_db_hz: f64,
        symbol_rate_sps: f64,
        implementation_allowance_db: f64,
    ) -> Result<f64, ModelError> {
        validate_link(
            c_over_no_db_hz,
            symbol_rate_sps,
            implementation_allowance_db,
        )?;
        let margin = c_over_no_db_hz
            - 10.0 * symbol_rate_sps.log10()
            - self.required_es_no_db()
            - implementation_allowance_db;
        if !margin.is_finite() {
            return Err(ModelError::NumericalFailure("DVB-S2 margin"));
        }
        Ok(margin)
    }

    /// Reference system information rate in bit/s at a positive symbol rate.
    ///
    /// Application encapsulation and additional overhead are not subtracted.
    pub fn information_rate_bps(self, symbol_rate_sps: f64) -> Result<f64, ModelError> {
        if !symbol_rate_sps.is_finite() || symbol_rate_sps <= 0.0 {
            return Err(ModelError::InvalidInput("symbol_rate_sps"));
        }
        let rate = symbol_rate_sps * self.information_bits_per_symbol();
        if !rate.is_finite() {
            return Err(ModelError::NumericalFailure("DVB-S2 information rate"));
        }
        Ok(rate)
    }
}

fn validate_link(c_no: f64, rs: f64, allowance: f64) -> Result<(), ModelError> {
    if !c_no.is_finite() {
        return Err(ModelError::InvalidInput("c_over_no_db_hz"));
    }
    if !rs.is_finite() || rs <= 0.0 {
        return Err(ModelError::InvalidInput("symbol_rate_sps"));
    }
    if !allowance.is_finite() || allowance < 0.0 {
        return Err(ModelError::InvalidInput("implementation_allowance_db"));
    }
    Ok(())
}

/// Select the highest-efficiency candidate that meets the fixed reference
/// criterion at one common symbol rate. Ties retain the first candidate.
///
/// Returns `Ok(None)` when no candidate closes, including an empty set. This
/// stateless calculation does not model ACM feedback, hysteresis, or pilots.
pub fn select_mode(
    candidates: &[Mode],
    c_over_no_db_hz: f64,
    symbol_rate_sps: f64,
    implementation_allowance_db: f64,
) -> Result<Option<Mode>, ModelError> {
    validate_link(
        c_over_no_db_hz,
        symbol_rate_sps,
        implementation_allowance_db,
    )?;
    let mut best: Option<Mode> = None;
    for &candidate in candidates {
        if candidate.margin_db(
            c_over_no_db_hz,
            symbol_rate_sps,
            implementation_allowance_db,
        )? >= 0.0
            && best.map_or(true, |previous| {
                candidate.information_bits_per_symbol() > previous.information_bits_per_symbol()
            })
        {
            best = Some(candidate);
        }
    }
    Ok(best)
}

/// Symbol rate for a raised-cosine occupied allocation in Hz and roll-off [0, 1].
///
/// This does not infer receiver equivalent noise bandwidth.
pub fn symbol_rate_from_occupied_bandwidth(
    occupied_bandwidth_hz: f64,
    rolloff: f64,
) -> Result<f64, ModelError> {
    if !occupied_bandwidth_hz.is_finite() || occupied_bandwidth_hz <= 0.0 {
        return Err(ModelError::InvalidInput("occupied_bandwidth_hz"));
    }
    if !rolloff.is_finite() || !(0.0..=1.0).contains(&rolloff) {
        return Err(ModelError::InvalidInput("rolloff"));
    }
    Ok(occupied_bandwidth_hz / (1.0 + rolloff))
}
