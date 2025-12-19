use rfconversions::frequency::frequency_to_wavelength;
use std::f64::consts::PI;

/// Path Loss (FSPL)
/// if you are modeling orbital mechanics, you may calculate slant range yourself and pass in the distance here
///
/// https://www.dsprelated.com/showarticle/62.php
/// this is a great article that explains the math behind path loss, and how propagation in free space is not freqeuncy dependent
/// """
/// It is apparent from careful study of the Friis Transmission Equation that
/// the often-mentioned "frequency-dependent propagation loss"
/// of radio waves is really an antenna effect and not a wave propagation effect.
/// The propagation of a radio wave or photon through free space is unaffected
/// by its frequency. Stick antennas, that is, dipole or monopole antennas,
/// are larger at lower frequencies and therefore have more effective area for energy collection.
/// Reflective antennas, that is dish antennas, of a given size have higher gain with decreasing wavelength.
/// Systems that use stick antennas therefore benefit from lower frequencies
/// while dish-antenna systems will attain higher system gain at higher frequencies.
/// """
pub struct PathLoss {
    pub frequency: f64,
    pub distance: f64,
}

impl PathLoss {
    pub fn calculate(&self) -> f64 {
        let wavelength: f64 = frequency_to_wavelength(self.frequency);
        let distance_wavelength_ratio: f64 = self.distance / wavelength;

        // (4 * PI * distance / wavelength).powf(2.0) in decibels
        let free_space_path_loss: f64 = 20.0 * f64::log10(4.0 * PI * distance_wavelength_ratio);

        free_space_path_loss
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn short_range_indoors() {
        let base: f64 = 10.0;

        // 60 GHz Wi-Fi, known as WiGig (IEEE 802.11ad/ay)
        let frequency: f64 = 60.0 * base.powf(9.0);
        let distance: f64 = 5.0 * base.powf(1.0); // 5 m

        let free_space_path_loss: f64 = PathLoss {
            frequency,
            distance,
        }
        .calculate();

        let rounded_to_4_decimal_places: f64 = (free_space_path_loss * 10000.0).round() / 10000.0;

        assert_eq!(101.9902, rounded_to_4_decimal_places);
    }

    #[test]
    fn leo_starlink_min() {
        let base: f64 = 10.0;
        let frequency: f64 = 14.0 * base.powf(9.0);

        // Starlink, min per wikipedia, directly above
        let distance: f64 = 340.0 * base.powf(3.0);

        let free_space_path_loss: f64 = PathLoss {
            frequency,
            distance,
        }
        .calculate();

        let rounded_to_4_decimal_places: f64 = (free_space_path_loss * 10000.0).round() / 10000.0;

        assert_eq!(165.9999, rounded_to_4_decimal_places);
    }

    #[test]
    fn meo_o3b_uplink() {
        let base: f64 = 10.0;

        // around 18 GHz for downlink (receive) and 28 GHz for uplink (transmit)
        let frequency: f64 = 28.0 * base.powf(9.0);

        // O3B Ka Uplink, for example, directly above
        let distance: f64 = 8.062 * base.powf(6.0);

        let free_space_path_loss: f64 = PathLoss {
            frequency,
            distance,
        }
        .calculate();
        assert_eq!(199.51979972506842, free_space_path_loss);
    }

    #[test]
    fn geo_galileo() {
        let base: f64 = 10.0;
        let frequency: f64 = 28.0 * base.powf(9.0);

        // GEO, for example, directly above
        let distance: f64 = 35.786 * base.powf(6.0);

        let free_space_path_loss: f64 = PathLoss {
            frequency,
            distance,
        }
        .calculate();
        assert_eq!(212.46520700065133, free_space_path_loss);
    }
}
