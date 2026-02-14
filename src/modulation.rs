//! Modulation types and their parameters.
//!
//! Defines modulation schemes (BPSK, QPSK, M-PSK, M-QAM) and provides
//! conversions between symbol rate, bit rate, and bandwidth.

/// Supported modulation types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Modulation {
    /// Binary Phase Shift Keying (M=2, k=1)
    Bpsk,
    /// Quadrature Phase Shift Keying (M=4, k=2)
    Qpsk,
    /// M-ary Phase Shift Keying
    Mpsk(u32),
    /// M-ary Quadrature Amplitude Modulation
    Mqam(u32),
    /// Minimum Shift Keying (M=2, k=1, continuous phase)
    Msk,
}

impl Modulation {
    /// Modulation order M
    pub fn order(&self) -> u32 {
        match self {
            Modulation::Bpsk => 2,
            Modulation::Qpsk => 4,
            Modulation::Mpsk(m) => *m,
            Modulation::Mqam(m) => *m,
            Modulation::Msk => 2,
        }
    }

    /// Bits per symbol: k = log2(M)
    pub fn bits_per_symbol(&self) -> f64 {
        (self.order() as f64).log2()
    }

    /// Symbol rate from information bit rate and code rate
    /// Rs = Rb / (k * R) where R is FEC code rate
    pub fn symbol_rate(&self, info_bit_rate_bps: f64, code_rate: f64) -> f64 {
        let coded_bit_rate = info_bit_rate_bps / code_rate;
        coded_bit_rate / self.bits_per_symbol()
    }

    /// Occupied bandwidth from symbol rate and roll-off factor (alpha)
    /// BW = Rs * (1 + alpha) for raised-cosine pulse shaping
    pub fn occupied_bandwidth(&self, symbol_rate: f64, rolloff: f64) -> f64 {
        symbol_rate * (1.0 + rolloff)
    }

    /// Null-to-null bandwidth (no pulse shaping)
    /// For most schemes this is 2 * Rs; for MSK it's 1.5 * Rs
    pub fn null_bandwidth(&self, symbol_rate: f64) -> f64 {
        match self {
            Modulation::Msk => 1.5 * symbol_rate,
            _ => 2.0 * symbol_rate,
        }
    }

    /// Spectral efficiency in bits/s/Hz (ideal, no roll-off)
    /// eta = k * R
    pub fn spectral_efficiency(&self, code_rate: f64) -> f64 {
        self.bits_per_symbol() * code_rate
    }
}

impl std::fmt::Display for Modulation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Modulation::Bpsk => write!(f, "BPSK"),
            Modulation::Qpsk => write!(f, "QPSK"),
            Modulation::Mpsk(m) => write!(f, "{}-PSK", m),
            Modulation::Mqam(m) => write!(f, "{}-QAM", m),
            Modulation::Msk => write!(f, "MSK"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bpsk_order_and_bits() {
        let m = Modulation::Bpsk;
        assert_eq!(m.order(), 2);
        assert!((m.bits_per_symbol() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn qpsk_order_and_bits() {
        let m = Modulation::Qpsk;
        assert_eq!(m.order(), 4);
        assert!((m.bits_per_symbol() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn qam64_order_and_bits() {
        let m = Modulation::Mqam(64);
        assert_eq!(m.order(), 64);
        assert!((m.bits_per_symbol() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn symbol_rate_from_bit_rate() {
        // 10 Mbps, QPSK (k=2), rate 1/2 FEC
        // Rc = 10e6 / 0.5 = 20e6 coded bps
        // Rs = 20e6 / 2 = 10e6 symbols/s
        let m = Modulation::Qpsk;
        let rs = m.symbol_rate(10e6, 0.5);
        assert!((rs - 10e6).abs() < 1.0);
    }

    #[test]
    fn occupied_bandwidth_with_rolloff() {
        let m = Modulation::Qpsk;
        let rs = 10e6;
        let bw = m.occupied_bandwidth(rs, 0.35);
        assert!((bw - 13.5e6).abs() < 1.0);
    }

    #[test]
    fn spectral_efficiency_qpsk_rate_half() {
        let m = Modulation::Qpsk;
        let eta = m.spectral_efficiency(0.5);
        assert!((eta - 1.0).abs() < 1e-10);
    }

    #[test]
    fn msk_null_bandwidth() {
        let m = Modulation::Msk;
        let bw = m.null_bandwidth(1e6);
        assert!((bw - 1.5e6).abs() < 1.0);
    }
}
