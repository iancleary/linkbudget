/// A transmitter in a link budget, characterized by output power, antenna gain, and bandwidth.
///
/// # Examples
///
/// ```
/// use linkbudget::transmitter::Transmitter;
///
/// let tx = Transmitter {
///     output_power: 30.0,  // 1 W in dBm
///     gain: 36.0,          // 36 dBi dish antenna
///     bandwidth: 36.0e6,   // 36 MHz transponder
/// };
///
/// assert!((tx.eirp_dbm() - 66.0).abs() < 1e-10);
/// assert!((tx.eirp_dbw() - 36.0).abs() < 1e-10);
/// ```
pub struct Transmitter {
    /// Transmitter output power in dBm (at the antenna port, after any line losses)
    pub output_power: f64,
    /// Antenna gain in dBi
    pub gain: f64,
    /// Signal bandwidth in Hz
    pub bandwidth: f64,
}

impl Transmitter {
    /// Effective Isotropic Radiated Power in dBm.
    ///
    /// EIRP = output power + antenna gain. This is the power that would need to
    /// be radiated by an isotropic antenna to produce the same signal strength
    /// in the direction of maximum gain.
    pub fn eirp_dbm(&self) -> f64 {
        self.output_power + self.gain
    }

    /// Effective Isotropic Radiated Power in dBW (EIRP_dBm − 30).
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
