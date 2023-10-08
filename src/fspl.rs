#[derive(Debug)]
pub struct FreeSpacePathLoss {
    pub frequency: f64, // in GHz
    pub distance: f64,  // in meters
}

pub fn calculate_slant_range(elevation_angle_degrees: f64, altitude: f64, body_radius: f64) -> f64 {
    let elevation_angle_radians = crate::conversions::degrees_to_radians(elevation_angle_degrees);
    let total_radius = altitude + body_radius;

    let radius_ratio = total_radius / body_radius;

    let radius_ratio_squared = radius_ratio * radius_ratio;

    let inner_term = f64::sqrt(
        radius_ratio_squared
            - f64::cos(elevation_angle_radians) * f64::cos(elevation_angle_radians)
            - f64::sin(elevation_angle_radians),
    );

    altitude * inner_term
}
