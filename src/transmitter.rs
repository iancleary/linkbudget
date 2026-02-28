//! Transmitter model for link budget calculations.

/// A radio transmitter with output power, antenna gain, and bandwidth.
#[doc(alias = "EIRP")]
pub struct Transmitter {
    /// Output power in dBm.
    pub output_power: f64,
    /// Antenna gain in dB.
    pub gain: f64,
    /// Bandwidth in Hz.
    pub bandwidth: f64,
}

impl Transmitter {
    /// Effective Isotropic Radiated Power in dBm.
    #[doc(alias = "EIRP")]
    #[must_use]
    pub fn eirp_dbm(&self) -> f64 {
        self.output_power + self.gain
    }

    /// Effective Isotropic Radiated Power in dBW.
    #[doc(alias = "EIRP")]
    #[must_use]
    pub fn eirp_dbw(&self) -> f64 {
        self.eirp_dbm() - 30.0
    }
}

#[cfg(test)]
mod tests {
    use crate::transmitter::Transmitter;

    #[test]
    fn eirp_dbm() {
        let tx = Transmitter {
            output_power: 30.0, // 1 W in dBm
            gain: 10.0,
            bandwidth: 100.0e6,
        };

        assert!((tx.eirp_dbm() - 40.0).abs() < 1e-10);
    }

    #[test]
    fn eirp_dbw() {
        let tx = Transmitter {
            output_power: 30.0,
            gain: 10.0,
            bandwidth: 100.0e6,
        };

        assert!((tx.eirp_dbw() - 10.0).abs() < 1e-10);
    }

    #[test]
    fn eirp_dbm_dbw_consistency() {
        let tx = Transmitter {
            output_power: 43.0, // 20W
            gain: 35.0,         // High-gain dish
            bandwidth: 36.0e6,
        };
        assert!((tx.eirp_dbm() - tx.eirp_dbw() - 30.0).abs() < 1e-10);
    }

    #[test]
    fn zero_gain_eirp_equals_power() {
        let tx = Transmitter {
            output_power: 20.0,
            gain: 0.0,
            bandwidth: 1.0e6,
        };
        assert!((tx.eirp_dbm() - 20.0).abs() < 1e-10);
    }

    #[test]
    fn negative_gain_lossy_antenna() {
        // Small antenna with negative gain (loss)
        let tx = Transmitter {
            output_power: 10.0,
            gain: -3.0,
            bandwidth: 1.0e6,
        };
        assert!((tx.eirp_dbm() - 7.0).abs() < 1e-10);
    }

    #[test]
    fn high_power_ka_band() {
        // Ka-band TWTA: 130W (51.1 dBm) + 42 dBi antenna
        let tx = Transmitter {
            output_power: 51.14,
            gain: 42.0,
            bandwidth: 500.0e6,
        };
        // EIRP ≈ 93.14 dBm = 63.14 dBW
        assert!((tx.eirp_dbm() - 93.14).abs() < 0.01);
        assert!((tx.eirp_dbw() - 63.14).abs() < 0.01);
    }
}
