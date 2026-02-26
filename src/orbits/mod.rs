//! Orbital mechanics calculations (circular orbits, slant range).

use crate::constants::GRAVITATIONAL_CONSTANT;

pub mod circular;
pub mod slant_range;

/// Standard gravitational parameter μ = G·M for a body of given mass.
#[doc(alias = "orbit")]
#[must_use]
pub fn calculate_standard_gravitational_parameter(mass_of_bodies: f64) -> f64 {
    GRAVITATIONAL_CONSTANT * mass_of_bodies
}
