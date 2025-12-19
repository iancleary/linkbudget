pub const SPEED_OF_LIGHT: f64 = 299792458.0;
pub const RADIUS_OF_EARTH: f64 = 6371000.0;
// 5.972 * 10.0f64.powf(24.0);
pub const MASS_OF_EARTH: f64 = 5.972e24;
pub const RADIUS_OF_MOON: f64 = 1737400.0;
pub const MASS_OF_MOON: f64 = 7.34767309e22;
pub const RADIUS_OF_SUN: f64 = 695500000.0;
pub const MASS_OF_SUN: f64 = 1.98847e30;
pub const RADIUS_OF_MARS: f64 = 3389500.0;
pub const MASS_OF_MARS: f64 = 6.4165e23;

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

    #[test]
    fn radius_of_earth() {
        use super::RADIUS_OF_EARTH;

        assert_eq!(6371000.0, RADIUS_OF_EARTH);
    }

    #[test]
    fn mass_of_earth() {
        use super::MASS_OF_EARTH;

        assert_eq!(5.972e24, MASS_OF_EARTH);
    }

    #[test]
    fn radius_of_moon() {
        use super::RADIUS_OF_MOON;

        assert_eq!(1737400.0, RADIUS_OF_MOON);
    }

    #[test]
    fn mass_of_moon() {
        use super::MASS_OF_MOON;

        assert_eq!(7.34767309e22, MASS_OF_MOON);
    }

    #[test]
    fn radius_of_sun() {
        use super::RADIUS_OF_SUN;

        assert_eq!(695500000.0, RADIUS_OF_SUN);
    }

    #[test]
    fn mass_of_sun() {
        use super::MASS_OF_SUN;

        assert_eq!(1.98847e30, MASS_OF_SUN);
    }

    #[test]
    fn radius_of_mars() {
        use super::RADIUS_OF_MARS;

        assert_eq!(3389500.0, RADIUS_OF_MARS);
    }

    #[test]
    fn mass_of_mars() {
        use super::MASS_OF_MARS;

        assert_eq!(6.4165e23, MASS_OF_MARS);
    }
}
