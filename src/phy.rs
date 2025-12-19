use core::fmt;
use std::fmt::{Display, Formatter};

pub struct PhyRate {
    pub bandwidth: f64, // Hz
    pub snr: f64,       // linear
}

impl PhyRate {
    pub fn bps(&self) -> f64 {
        // PHY Rate in bps
        self.bandwidth * (1.0 + self.snr).log2()
    }

    pub fn mbps(&self) -> f64 {
        // PHY Rate in Mbps
        self.bps() / 1_000_000.0
    }

    pub fn gbps(&self) -> f64 {
        // PHY Rate in Gbps
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
