//! Errors at the boundary of validated planning models.

use std::fmt;

/// An invalid input, unsupported regime, or failed numerical evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelError {
    /// The named input violates the function's documented domain.
    InvalidInput(&'static str),
    /// Valid physical inputs are outside the implemented approximation's regime.
    UnsupportedRegime(&'static str),
    /// Floating-point evaluation did not produce a usable finite result.
    NumericalFailure(&'static str),
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(name) => write!(f, "invalid input: {name}"),
            Self::UnsupportedRegime(reason) => write!(f, "unsupported regime: {reason}"),
            Self::NumericalFailure(reason) => write!(f, "numerical failure: {reason}"),
        }
    }
}

impl std::error::Error for ModelError {}
