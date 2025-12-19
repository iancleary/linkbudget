// this is a 1-D geometric calculation of slant range
// further work is needed to account for the lat/long of the transmitter and receiver
// for the full 3D calculation, TLEs would be needed for the satellite, lat/long/altitude for the other point
// this doesn't also allow from point to point communications (ground <-> ground, ground <-> satellite, satellite <-> satellite, satellite <-> plane/etc.)

pub struct SlantRange {
    pub elevation_angle_degrees: f64,
    pub altitude: f64,
    pub body_radius: f64,
}

impl SlantRange {
    pub fn calculate(&self) -> f64 {
        let elevation_angle_radians: f64 =
            crate::conversions::angle::degrees_to_radians(self.elevation_angle_degrees);
        let total_radius: f64 = self.altitude + self.body_radius;

        let total_radius_ratio: f64 = total_radius / self.body_radius;

        let radius_ratio_squared: f64 = total_radius_ratio * total_radius_ratio;

        let inner_term: f64 = f64::sqrt(
            radius_ratio_squared
                - f64::cos(elevation_angle_radians) * f64::cos(elevation_angle_radians),
        );

        self.body_radius * (inner_term - f64::sin(elevation_angle_radians))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_above() {
        let elevation_angle_degrees: f64 = 90.0;
        let base: f64 = 10.0;
        let altitude: f64 = base.powf(6.0);
        let body_radius: f64 = 6371000.0;

        let slant_range: f64 = SlantRange {
            elevation_angle_degrees,
            altitude,
            body_radius,
        }
        .calculate();
        assert_eq!(altitude, slant_range);
    }

    #[test]
    fn thirty_five_degrees() {
        let elevation_angle_degrees: f64 = 35.0;
        let base: f64 = 10.0;
        let altitude: f64 = base.powf(6.0);
        let body_radius: f64 = 6371000.0;

        let slant_range: f64 = SlantRange {
            elevation_angle_degrees,
            altitude,
            body_radius,
        }
        .calculate();
        assert_eq!(1.551086307581479 * altitude, slant_range);
    }

    #[test]
    fn horizon() {
        let elevation_angle_degrees: f64 = 0.0;
        let base: f64 = 10.0;
        let altitude: f64 = base.powf(6.0);
        let body_radius: f64 = 6371000.0;

        let slant_range: f64 = SlantRange {
            elevation_angle_degrees,
            altitude,
            body_radius,
        }
        .calculate();
        assert_eq!(3.707020366817534 * altitude, slant_range);
    }

    #[test]
    fn leo_slant_range() {
        let base: f64 = 10.0;

        // starlink, min per wikipedia
        let altitude: f64 = 340.0 * base.powf(3.0); // 340,000 m
        let elevation_angle_degrees: f64 = 35.0;

        let slant_range: f64 = SlantRange {
            elevation_angle_degrees,
            altitude,
            body_radius: 6371000.0,
        }
        .calculate();

        let rounded_to_4_decimal_places: f64 = (slant_range * 10000.0).round() / 10000.0;
        assert_eq!(564922.5345, rounded_to_4_decimal_places);
    }

    #[test]
    fn leo_slant_range_2() {
        let base: f64 = 10.0;

        // starlink, min per wikipedia
        let altitude: f64 = 550.0 * base.powf(3.0); // 550,000 m
        let elevation_angle_degrees: f64 = 35.0;

        let slant_range: f64 = SlantRange {
            elevation_angle_degrees,
            altitude,
            body_radius: 6371000.0,
        }
        .calculate();

        let rounded_to_4_decimal_places: f64 = (slant_range * 10000.0).round() / 10000.0;
        assert_eq!(891531.9238, rounded_to_4_decimal_places);
    }

    #[test]
    fn meo_slant_range() {
        let base: f64 = 10.0;

        // O3B Ka Uplink, for example
        let altitude: f64 = 8.062 * base.powf(6.0);

        let elevation_angle_degrees: f64 = 50.0;

        let slant_range: f64 = SlantRange {
            elevation_angle_degrees,
            altitude,
            body_radius: 6371000.0,
        }
        .calculate();

        let rounded_to_4_decimal_places: f64 = (slant_range * 10000.0).round() / 10000.0;
        assert_eq!(8959358.4203, rounded_to_4_decimal_places);
    }

    #[test]
    fn geo_slant_range() {
        let base: f64 = 10.0;

        // GEO, for example
        let altitude: f64 = 35.786 * base.powf(6.0);

        let elevation_angle_degrees: f64 = 80.0;

        let slant_range: f64 = SlantRange {
            elevation_angle_degrees,
            altitude,
            body_radius: 6371000.0,
        }
        .calculate();

        let rounded_to_4_decimal_places: f64 = (slant_range * 10000.0).round() / 10000.0;
        assert_eq!(35868271.0040, rounded_to_4_decimal_places);
    }
}
