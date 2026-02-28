//! Shannon-capacity PHY rate calculation.

use core::fmt;
use std::fmt::{Display, Formatter};

/// Shannon-capacity PHY rate for a given bandwidth and SNR.
pub struct PhyRate {
    /// Channel bandwidth in Hz.
    pub bandwidth: f64,
    /// Signal-to-noise ratio (linear, not dB).
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
            &self.bandwidth.to_string(),
            &self.snr.to_string(),
            &self.mbps().to_string()
        )
    }
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
