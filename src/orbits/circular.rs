//! Circular orbit speed and period calculations.

use crate::constants::GRAVITATIONAL_CONSTANT;

/// Orbital speed for a circular orbit (m/s).
#[doc(alias = "orbit")]
#[must_use]
pub fn calculate_circular_orbit_speed(mass_of_body: f64, distance_from_center_of_body: f64) -> f64 {
    let orbital_speed: f64 =
        (GRAVITATIONAL_CONSTANT * mass_of_body / distance_from_center_of_body).sqrt();

    orbital_speed
}

/// Orbital period for a circular orbit (seconds).
#[doc(alias = "orbit")]
#[must_use]
pub fn calculate_circular_orbit_period(
    mass_of_body: f64,
    distance_from_center_of_body: f64,
) -> f64 {
    let inner_term: f64 =
        distance_from_center_of_body.powf(3.0) / (GRAVITATIONAL_CONSTANT * mass_of_body);

    let orbital_period: f64 = 2.0 * std::f64::consts::PI * inner_term.sqrt();

    orbital_period
}

#[cfg(test)]
mod tests {
    use crate::constants::MASS_OF_EARTH;
    use crate::constants::RADIUS_OF_EARTH;

    #[test]
    fn leo_earth() {
        let base: f64 = 10.0;
        let altitude: f64 = 1.0 * base.powf(6.0);
        let distance_from_center_of_body: f64 = altitude + RADIUS_OF_EARTH;

        let orbital_speed: f64 =
            super::calculate_circular_orbit_speed(MASS_OF_EARTH, distance_from_center_of_body);

        assert_eq!(7353.592432681345, orbital_speed);
    }

    #[test]
    fn leo_earth_period() {
        let base: f64 = 10.0;
        let altitude: f64 = 1.0 * base.powf(6.0);
        let distance_from_center_of_body: f64 = altitude + RADIUS_OF_EARTH;

        let orbital_period: f64 =
            super::calculate_circular_orbit_period(MASS_OF_EARTH, distance_from_center_of_body);

        assert_eq!(6298.058985889903, orbital_period);

        let orbital_period_minutes = orbital_period / 60.0;
        assert_eq!(104.96764976483172, orbital_period_minutes);
    }

    #[test]
    fn leo_earth_period_higher() {
        let base: f64 = 10.0;
        let altitude: f64 = 2.0 * base.powf(6.0);
        let distance_from_center_of_body: f64 = altitude + RADIUS_OF_EARTH;

        let orbital_period: f64 =
            super::calculate_circular_orbit_period(MASS_OF_EARTH, distance_from_center_of_body);

        assert_eq!(7622.248787682895, orbital_period);

        let orbital_period_minutes = orbital_period / 60.0;
        assert_eq!(127.03747979471493, orbital_period_minutes);
    }
}
