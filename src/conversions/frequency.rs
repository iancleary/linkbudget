pub fn frequency_to_wavelength(frequency: f64) -> f64 {
    crate::constants::SPEED_OF_LIGHT / frequency
}

#[cfg(test)]
mod tests {

    #[test]
    fn one_gigahertz() {
        let base: f64 = 10.0;
        let frequency: f64 = 1.0 * base.powf(9.0);

        let wavelength: f64 = super::frequency_to_wavelength(frequency);

        assert_eq!(0.299792458, wavelength);
    }

    #[test]
    fn twenty_seven_point_five_gigahertz() {
        let base: f64 = 10.0;
        let frequency: f64 = 27.5 * base.powf(9.0);

        let wavelength: f64 = super::frequency_to_wavelength(frequency);

        assert_eq!(0.010901543927272727, wavelength);
    }

    #[test]
    fn thirty_gigahertz() {
        let base: f64 = 10.0;
        let frequency: f64 = 30.0 * base.powf(9.0);

        let wavelength: f64 = super::frequency_to_wavelength(frequency);

        assert_eq!(0.009993081933333333, wavelength);
    }
}
