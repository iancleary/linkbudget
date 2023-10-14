use crate::constants::GRAVITATIONAL_CONSTANT;

pub fn circular_orbital_speed(mass_of_body: f64, distance_from_center_of_body: f64) -> f64 {
    // F = G*M*m/ r^2 = mv^2/r

    // G*M = mv^2/r

    // v^2 = G*M/r
    // v = sqrt(G*M/r)

    let orbital_speed: f64 =
        (GRAVITATIONAL_CONSTANT * mass_of_body / distance_from_center_of_body).sqrt();

    orbital_speed
}

#[cfg(test)]
mod tests {
    use crate::constants::RADIUS_OF_EARTH;

    #[test]
    fn leo_earth() {
        let earth_mass: f64 = 5.972 * 10.0f64.powf(24.0);

        let base: f64 = 10.0;
        let altitude: f64 = 1.0 * base.powf(6.0);
        let distance_from_center_of_body: f64 = altitude + RADIUS_OF_EARTH;

        let orbital_speed: f64 =
            super::circular_orbital_speed(earth_mass, distance_from_center_of_body);
        assert_eq!(7353.592432681345, orbital_speed);
    }
}
