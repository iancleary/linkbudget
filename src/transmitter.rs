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
}
