//! Orbital mechanics calculations (circular orbits, slant range).
//!
//! Derivations, units, and model limits are in `docs/physics-geometry-and-errors.md`.

use crate::constants::GRAVITATIONAL_CONSTANT;

pub mod circular;
pub mod slant_range;

/// Standard gravitational parameter μ = G·M for a body of given mass.
///
/// A positive mass in kg gives μ in m³/s². The result inherits the precision
/// of the supplied mass and the measured gravitational constant.
#[doc(alias = "orbit")]
#[must_use]
pub fn calculate_standard_gravitational_parameter(mass_of_bodies: f64) -> f64 {
    GRAVITATIONAL_CONSTANT * mass_of_bodies
}
