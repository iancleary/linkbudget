pub const SPEED_OF_LIGHT: f64 = 299792458.0;
pub const RADIUS_OF_EARTH: f64 = 6371000.0;

// https://en.wikipedia.org/wiki/Gravitational_constant
// 6.67430(15)×10−11 m3⋅kg−1⋅s−2
pub const GRAVITATIONAL_CONSTANT: f64 = 0.0000000000667430;

#[cfg(test)]
mod tests {

    #[test]
    fn speed_of_light() {
        use super::SPEED_OF_LIGHT;

        assert_eq!(299792458.0, SPEED_OF_LIGHT);
    }

    #[test]
    fn gravitational_constant() {
        use super::GRAVITATIONAL_CONSTANT;

        const BASE_TEN: f64 = 10.0;
        const POWER_OF_NEGATIVE_ELEVEN: f64 = -11.0;
        let expected: f64 = 6.67430 * BASE_TEN.powf(POWER_OF_NEGATIVE_ELEVEN);
        assert_eq!(expected, GRAVITATIONAL_CONSTANT);
    }
}
