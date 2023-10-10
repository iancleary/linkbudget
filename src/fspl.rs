use crate::frequency::frequency_to_wavelength;
use std::f64::consts::PI;

pub fn calculate_slant_range(elevation_angle_degrees: f64, altitude: f64, body_radius: f64) -> f64 {
    let elevation_angle_radians: f64 =
        crate::conversions::degrees_to_radians(elevation_angle_degrees);
    let total_radius: f64 = altitude + body_radius;

    let total_radius_ratio: f64 = total_radius / body_radius;

    let radius_ratio_squared: f64 = total_radius_ratio * total_radius_ratio;

    let inner_term: f64 = f64::sqrt(
        radius_ratio_squared
            - f64::cos(elevation_angle_radians) * f64::cos(elevation_angle_radians),
    );

    body_radius * (inner_term - f64::sin(elevation_angle_radians))
}

pub fn calculate_free_space_path_loss(frequency: f64, distance: f64) -> f64 {
    let wavelength: f64 = frequency_to_wavelength(frequency);
    let distance_wavelength_ratio: f64 = distance / wavelength;

    // (4 * PI * distance / wavelength).powf(2.0) in decibels
    let free_space_path_loss: f64 = 20.0 * f64::log10(4.0 * PI * distance_wavelength_ratio);

    free_space_path_loss
}

#[cfg(test)]
mod tests {
    use crate::fspl::calculate_slant_range;

    #[test]
    fn straight_above() {
        let elevation_angle_degrees: f64 = 90.0;
        let base: f64 = 10.0;
        let altitude: f64 = base.powf(6.0);
        let body_radius: f64 = 6371000.0;

        let slant_range: f64 =
            calculate_slant_range(elevation_angle_degrees, altitude, body_radius);
        assert_eq!(altitude, slant_range);
    }

    #[test]
    fn thirty_five_degrees() {
        let elevation_angle_degrees: f64 = 35.0;
        let base: f64 = 10.0;
        let altitude: f64 = base.powf(6.0);
        let body_radius: f64 = 6371000.0;

        let slant_range: f64 =
            calculate_slant_range(elevation_angle_degrees, altitude, body_radius);
        assert_eq!(1.551086307581479 * altitude, slant_range);
    }

    #[test]
    fn horizon() {
        let elevation_angle_degrees: f64 = 0.0;
        let base: f64 = 10.0;
        let altitude: f64 = base.powf(6.0);
        let body_radius: f64 = 6371000.0;

        let slant_range: f64 =
            calculate_slant_range(elevation_angle_degrees, altitude, body_radius);
        assert_eq!(3.707020366817534 * altitude, slant_range);
    }

    use crate::fspl::calculate_free_space_path_loss;

    #[test]
    fn leo() {
        let base: f64 = 10.0;
        let frequency: f64 = 27.5 * base.powf(9.0);

        let distance: f64 = 1.0 * base.powf(6.0);

        let free_space_path_loss: f64 = calculate_free_space_path_loss(frequency, distance);
        assert_eq!(181.23443709848863, free_space_path_loss);
    }

    #[test]
    fn leo_slant_range() {
        let base: f64 = 10.0;
        let frequency: f64 = 27.5 * base.powf(9.0);

        let altitude: f64 = 1.0 * base.powf(6.0);
        let elevation_angle_degrees: f64 = 35.0;

        let slant_range: f64 = calculate_slant_range(elevation_angle_degrees, altitude, 6371000.0);

        let free_space_path_loss: f64 = calculate_free_space_path_loss(frequency, slant_range);
        assert_eq!(185.04715637988681, free_space_path_loss);
    }
}
