use crate::constants::GRAVITATIONAL_CONSTANT;

pub mod circular;
pub mod slant_range;

pub fn calculate_standard_gravitational_parameter(mass_of_bodies: f64) -> f64 {
    GRAVITATIONAL_CONSTANT * mass_of_bodies
}
