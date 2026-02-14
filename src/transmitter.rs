pub struct Transmitter {
    pub output_power: f64, // dBm
    pub gain: f64,         // dB
    pub bandwidth: f64,    // Hz
}

impl Transmitter {
    pub fn eirp_dbm(&self) -> f64 {
        self.output_power + self.gain
    }

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
