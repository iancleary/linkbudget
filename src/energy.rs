//! Energy-per-bit metrics: Eb/No, Es/No, Ec/No, C/No conversions.
//!
//! Reference: <https://www.dsprelated.com/showarticle/168.php>
//!
//! ## Definitions
//! - **C/No** (or C/N₀): Carrier power to noise spectral density ratio (dB·Hz)
//! - **Es/No**: Energy per symbol to noise spectral density ratio (dB)
//! - **Ec/No**: Energy per coded bit to noise spectral density ratio (dB)
//! - **Eb/No**: Energy per information bit to noise spectral density ratio (dB)
//! - **SNR** (C/N): Carrier to noise ratio in a given bandwidth (dB)
//!
//! ## Key Relationships (all in dB)
//! - C/No = SNR + 10·log10(BW)
//! - Es/No = C/No - 10·log10(Rs)
//! - Ec/No = Es/No - 10·log10(k)    where k = bits per symbol
//! - Eb/No = Ec/No + 10·log10(R)     where R = code rate (Eb/No = Ec/No / R in linear)
//!   equivalently: Eb/No = Es/No - 10·log10(k) + 10·log10(R)
//!   or: Eb/No = C/No - 10·log10(Rb)

use crate::modulation::Modulation;

/// Convert SNR (C/N) in dB to C/No in dB·Hz
/// C/No = SNR_dB + 10·log10(noise_bandwidth_hz)
pub fn snr_to_c_over_no(snr_db: f64, noise_bandwidth_hz: f64) -> f64 {
    snr_db + 10.0 * noise_bandwidth_hz.log10()
}

/// Convert C/No in dB·Hz to SNR (C/N) in dB
/// SNR = C/No - 10·log10(noise_bandwidth_hz)
pub fn c_over_no_to_snr(c_over_no_db_hz: f64, noise_bandwidth_hz: f64) -> f64 {
    c_over_no_db_hz - 10.0 * noise_bandwidth_hz.log10()
}

/// Convert C/No to Es/No given symbol rate
/// Es/No = C/No - 10·log10(Rs)
pub fn c_over_no_to_es_over_no(c_over_no_db_hz: f64, symbol_rate: f64) -> f64 {
    c_over_no_db_hz - 10.0 * symbol_rate.log10()
}

/// Convert Es/No to Eb/No given modulation and code rate
/// Eb/No = Es/No - 10·log10(k) + 10·log10(R)
///       = Es/No - 10·log10(k/R)
///
/// Note: Eb > Ec because Eb = Ec/R and R < 1, so dividing by R increases energy.
/// In dB: Eb/No = Ec/No - 10·log10(R) ... but Ec/No = Es/No - 10·log10(k)
/// so Eb/No = Es/No - 10·log10(k) - 10·log10(R)
/// Wait — let's be precise:
///   Rb = Rs · k · R  (info bit rate = symbol rate × bits/symbol × code rate)
///   Eb = C / Rb = C / (Rs · k · R)
///   Eb/No = C/No / (Rs · k · R) = (C/No) - 10log10(Rs·k·R) [in dB]
///   Eb/No = Es/No - 10log10(k·R)
///   Eb/No = Es/No - 10log10(k) - 10log10(R)
pub fn es_over_no_to_eb_over_no(es_over_no_db: f64, modulation: &Modulation, code_rate: f64) -> f64 {
    let k = modulation.bits_per_symbol();
    es_over_no_db - 10.0 * k.log10() - 10.0 * code_rate.log10()
}

/// Convert Eb/No to Es/No
/// Es/No = Eb/No + 10·log10(k) + 10·log10(R)
pub fn eb_over_no_to_es_over_no(eb_over_no_db: f64, modulation: &Modulation, code_rate: f64) -> f64 {
    let k = modulation.bits_per_symbol();
    eb_over_no_db + 10.0 * k.log10() + 10.0 * code_rate.log10()
}

/// Convert C/No directly to Eb/No given information bit rate
/// Eb/No = C/No - 10·log10(Rb)
pub fn c_over_no_to_eb_over_no(c_over_no_db_hz: f64, info_bit_rate_bps: f64) -> f64 {
    c_over_no_db_hz - 10.0 * info_bit_rate_bps.log10()
}

/// Convert Eb/No to C/No given information bit rate
/// C/No = Eb/No + 10·log10(Rb)
pub fn eb_over_no_to_c_over_no(eb_over_no_db: f64, info_bit_rate_bps: f64) -> f64 {
    eb_over_no_db + 10.0 * info_bit_rate_bps.log10()
}

/// Es/No to Ec/No: Ec/No = Es/No - 10·log10(k)
pub fn es_over_no_to_ec_over_no(es_over_no_db: f64, modulation: &Modulation) -> f64 {
    es_over_no_db - 10.0 * modulation.bits_per_symbol().log10()
}

/// Ec/No to Eb/No: Eb/No = Ec/No - 10·log10(R)
pub fn ec_over_no_to_eb_over_no(ec_over_no_db: f64, code_rate: f64) -> f64 {
    ec_over_no_db - 10.0 * code_rate.log10()
}

/// Comprehensive conversion: SNR to Eb/No
/// Given SNR in a bandwidth, modulation, symbol rate, and code rate
pub fn snr_to_eb_over_no(
    snr_db: f64,
    noise_bandwidth_hz: f64,
    modulation: &Modulation,
    symbol_rate: f64,
    code_rate: f64,
) -> f64 {
    let c_no = snr_to_c_over_no(snr_db, noise_bandwidth_hz);
    let es_no = c_over_no_to_es_over_no(c_no, symbol_rate);
    es_over_no_to_eb_over_no(es_no, modulation, code_rate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modulation::Modulation;

    #[test]
    fn snr_to_c_over_no_1mhz() {
        // SNR = 10 dB in 1 MHz bandwidth
        // C/No = 10 + 10*log10(1e6) = 10 + 60 = 70 dB·Hz
        let c_no = snr_to_c_over_no(10.0, 1e6);
        assert!((c_no - 70.0).abs() < 1e-10);
    }

    #[test]
    fn c_over_no_roundtrip() {
        let snr = 15.0;
        let bw = 36e6;
        let c_no = snr_to_c_over_no(snr, bw);
        let snr_back = c_over_no_to_snr(c_no, bw);
        assert!((snr - snr_back).abs() < 1e-10);
    }

    #[test]
    fn c_over_no_to_eb_over_no_direct() {
        // C/No = 70 dB·Hz, Rb = 1 Mbps
        // Eb/No = 70 - 10*log10(1e6) = 70 - 60 = 10 dB
        let eb_no = c_over_no_to_eb_over_no(70.0, 1e6);
        assert!((eb_no - 10.0).abs() < 1e-10);
    }

    #[test]
    fn eb_no_roundtrip_via_c_no() {
        let eb_no = 9.6;
        let rb = 5e6;
        let c_no = eb_over_no_to_c_over_no(eb_no, rb);
        let eb_no_back = c_over_no_to_eb_over_no(c_no, rb);
        assert!((eb_no - eb_no_back).abs() < 1e-10);
    }

    #[test]
    fn es_no_to_eb_no_qpsk_rate_half() {
        // QPSK: k=2, code rate R=1/2
        // Eb/No = Es/No - 10log10(2) - 10log10(0.5)
        //       = Es/No - 3.01 + 3.01 = Es/No
        let mod_qpsk = Modulation::Qpsk;
        let es_no = 10.0;
        let eb_no = es_over_no_to_eb_over_no(es_no, &mod_qpsk, 0.5);
        assert!((eb_no - es_no).abs() < 0.01, "For QPSK rate-1/2, Eb/No ≈ Es/No");
    }

    #[test]
    fn es_no_to_eb_no_bpsk_rate_one() {
        // BPSK: k=1, R=1 (uncoded)
        // Eb/No = Es/No - 10log10(1) - 10log10(1) = Es/No
        let m = Modulation::Bpsk;
        let es_no = 12.0;
        let eb_no = es_over_no_to_eb_over_no(es_no, &m, 1.0);
        assert!((eb_no - 12.0).abs() < 1e-10);
    }

    #[test]
    fn eb_no_es_no_roundtrip() {
        let m = Modulation::Mqam(16);
        let code_rate = 0.75;
        let eb_no = 8.0;
        let es_no = eb_over_no_to_es_over_no(eb_no, &m, code_rate);
        let eb_no_back = es_over_no_to_eb_over_no(es_no, &m, code_rate);
        assert!((eb_no - eb_no_back).abs() < 1e-10);
    }

    #[test]
    fn ec_no_chain() {
        // QPSK (k=2), rate 3/4
        // Es/No = 12 dB
        // Ec/No = 12 - 10log10(2) = 12 - 3.01 = 8.99 dB
        // Eb/No = 8.99 - 10log10(0.75) = 8.99 + 1.25 = 10.24 dB
        let m = Modulation::Qpsk;
        let es_no = 12.0;
        let ec_no = es_over_no_to_ec_over_no(es_no, &m);
        let eb_no = ec_over_no_to_eb_over_no(ec_no, 0.75);

        let expected_ec = 12.0 - 10.0 * 2.0_f64.log10();
        assert!((ec_no - expected_ec).abs() < 0.01);

        let expected_eb = expected_ec - 10.0 * 0.75_f64.log10();
        assert!((eb_no - expected_eb).abs() < 0.01);
    }

    #[test]
    fn full_chain_snr_to_eb_no() {
        // SNR = 20 dB in 10 MHz noise BW
        // QPSK, Rs = 5 Msps, R = 3/4
        // C/No = 20 + 70 = 90 dB·Hz
        // Es/No = 90 - 10log10(5e6) = 90 - 66.99 = 23.01 dB
        // Eb/No = 23.01 - 10log10(2) - 10log10(0.75) = 23.01 - 3.01 + 1.25 = 21.25 dB
        let m = Modulation::Qpsk;
        let eb_no = snr_to_eb_over_no(20.0, 10e6, &m, 5e6, 0.75);

        let expected = 20.0 + 10.0 * 10e6_f64.log10()
            - 10.0 * 5e6_f64.log10()
            - 10.0 * 2.0_f64.log10()
            - 10.0 * 0.75_f64.log10();
        assert!((eb_no - expected).abs() < 0.01);
    }
}
