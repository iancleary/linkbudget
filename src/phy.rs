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
}
