#[derive(Debug)]
pub struct FreeSpacePathLoss {
    pub frequency: f64, // in GHz
    pub distance: f64,  // in meters
}

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
}
