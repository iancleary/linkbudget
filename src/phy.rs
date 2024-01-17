use crate::utils::print::print_row;

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

    pub fn print(&self) {
        print_row("Bandwidth", &self.bandwidth.to_string(), "Hz");
        print_row("SNR", &self.snr.to_string(), "linear");
        print_row("Shannon Capacity", "B * log2(1 + SNR)", "");
        print_row("PHY Rate", &self.bps().to_string(), "bps");
        print_row("PHY Rate", &self.mbps().to_string(), "Mbps");
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
