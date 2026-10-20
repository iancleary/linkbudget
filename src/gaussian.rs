//! Shared Gaussian tails, independently implemented from DLMF 7.6.1,
//! 7.11.2 and 8.9.2. No third-party implementation code is used.

use crate::ModelError;
use std::f64::consts::{LN_2, PI, SQRT_2};

const MAX_ITERATIONS: usize = 10_000;
const LOG_SQRT_PI: f64 = 0.572_364_942_924_700_1;

// Positive erfc is Q(1/2, x²). Return its logarithm before exponentiation,
// so callers can retain probabilities below the subnormal range.
pub(crate) fn log_erfc_positive(x: f64) -> Result<f64, ModelError> {
    if x < 1.5 {
        // DLMF 7.6.1: erf(x) = 2/sqrt(pi) sum (-1)^n x^(2n+1)/(n!(2n+1)).
        let mut power = x;
        let mut sum = x;
        for n in 1..=MAX_ITERATIONS {
            power *= -x * x / n as f64;
            let term = power / (2 * n + 1) as f64;
            sum += term;
            if term.abs() <= f64::EPSILON * sum.abs() {
                return Ok((-(2.0 / PI.sqrt()) * sum).ln_1p());
            }
        }
    } else {
        let squared = x * x;
        if !squared.is_finite() {
            return Err(ModelError::NumericalFailure("Gaussian tail exponent"));
        }
        // DLMF 8.9.2, normalized by Gamma(1/2): the fraction has
        // b_n = x² + 1/2 + 2n and a_n = -n(n-1/2).
        // For x >= 1.5 these denominators stay positive; no tiny-value
        // substitutions are needed in the modified Lentz recurrence.
        let mut b = squared + 0.5;
        let mut c = b;
        let mut d = 0.0;
        let mut fraction = b;
        for n in 1..=MAX_ITERATIONS {
            let a = -(n as f64) * (n as f64 - 0.5);
            b += 2.0;
            c = b + a / c;
            d = 1.0 / (b + a * d);
            let delta = c * d;
            fraction *= delta;
            if (delta - 1.0).abs() <= 2.0 * f64::EPSILON {
                let result = -squared + x.ln() - LOG_SQRT_PI - fraction.ln();
                if result.is_finite() {
                    return Ok(result);
                }
                return Err(ModelError::NumericalFailure("Gaussian tail logarithm"));
            }
        }
    }
    Err(ModelError::NumericalFailure("Gaussian tail convergence"))
}

pub(crate) fn log_q_positive(x: f64) -> Result<f64, ModelError> {
    Ok(log_erfc_positive(x / SQRT_2)? - LN_2)
}
