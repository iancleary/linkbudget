use crate::constants::GRAVITATIONAL_CONSTANT;

pub fn calculate_circular_orbit_speed(mass_of_body: f64, distance_from_center_of_body: f64) -> f64 {
    // F = G*M*m/ r^2 = mv^2/r

    // G*M = mv^2/r

    // v^2 = G*M/r
    // v = sqrt(G*M/r)

    let orbital_speed: f64 =
        (GRAVITATIONAL_CONSTANT * mass_of_body / distance_from_center_of_body).sqrt();

    // m/s
    orbital_speed
}

pub fn calculate_circular_orbit_period(mass_of_body: f64, distance_from_center_of_body: f64) -> f64 {
    // T = 2*pi*sqrt(r^3/G*M)
    // Returns the orbital period in seconds

    // numerator: m^3
    // denominator: N*m^2/kg^2 * kg = N*m^2/kg

    // flip denominator
    // 1/(N*m^2/kg) = kg/(N*m^2)

    // sum numerator and flipped denominator
    // m^3 * kg / (N*m^2) = m^3 * kg / (kg*(m/s^2)*m^2) = m^3 / ((m^3) * 1/s^2) = s ^ 2

    // Return unit is seconds
    // sqrt(s^2) = s


    let inner_term: f64 = distance_from_center_of_body.powf(3.0) / (GRAVITATIONAL_CONSTANT * mass_of_body);

    let orbital_period: f64 = 2.0 * std::f64::consts::PI * inner_term.sqrt();

    // seconds
    orbital_period
}

#[cfg(test)]
mod tests {
    use crate::constants::RADIUS_OF_EARTH;
    use crate::constants::MASS_OF_EARTH;

    #[test]
    fn leo_earth() {
        let base: f64 = 10.0;
        let altitude: f64 = 1.0 * base.powf(6.0);
        let distance_from_center_of_body: f64 = altitude + RADIUS_OF_EARTH;

        let orbital_speed: f64 =
            super::calculate_circular_orbit_speed(MASS_OF_EARTH, distance_from_center_of_body);

        // m/s
        assert_eq!(7353.592432681345, orbital_speed);
    }

    #[test]
    fn leo_earth_period() {
        let base: f64 = 10.0;
        let altitude: f64 = 1.0 * base.powf(6.0); // 1_000 km altitude
        let distance_from_center_of_body: f64 = altitude + RADIUS_OF_EARTH;

        let orbital_period: f64 = super::calculate_circular_orbit_period(MASS_OF_EARTH, distance_from_center_of_body);

        // seconds
        assert_eq!(6298.058985889903, orbital_period);

        let orbital_period_minutes = orbital_period / 60.0;

        // minutes
        assert_eq!(104.96764976483172, orbital_period_minutes);
    }

    #[test]
    fn leo_earth_period_higher() {
        let base: f64 = 10.0;
        let altitude: f64 = 2.0 * base.powf(6.0); // 2_000 km altitude
        let distance_from_center_of_body: f64 = altitude + RADIUS_OF_EARTH;

        let orbital_period: f64 = super::calculate_circular_orbit_period(MASS_OF_EARTH, distance_from_center_of_body);

        // seconds
        assert_eq!(7622.248787682895, orbital_period);

        let orbital_period_minutes = orbital_period / 60.0;

        // minutes
        assert_eq!(127.03747979471493, orbital_period_minutes);
    }

}
