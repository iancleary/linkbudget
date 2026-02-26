//! A link budget toolbox for satellite communications.
//!
//! This crate provides tools for calculating link budgets, including
//! path loss, receiver sensitivity, BER curves, modulation schemes,
//! EVM, orbital mechanics, and Doppler analysis.
#![warn(missing_docs)]

pub mod ber;
mod budget;
#[allow(missing_docs, dead_code)]
pub mod cli;
pub mod coding;
pub mod constants;
pub mod doppler;
pub mod energy;
pub mod evm;
#[allow(missing_docs, dead_code)]
pub(crate) mod file_operations;
pub mod modulation;
#[allow(missing_docs, dead_code)]
pub(crate) mod open;
pub mod orbits;
pub mod path_loss;
pub mod pfd;
pub mod phy;
#[allow(missing_docs, dead_code)]
pub(crate) mod plot;
pub mod quantization;
pub mod receiver;
pub mod sensitivity;
pub mod transmitter;

pub use ber::{ber, ber_from_db, link_margin_db, required_eb_no_db};
pub use budget::LinkBudget;
pub use coding::{CodedModulation, FecCode};
pub use doppler::*;
pub use energy::*;
pub use evm::*;
pub use modulation::Modulation;
pub use path_loss::PathLoss;
pub use pfd::*;
pub use quantization::*;
pub use receiver::Receiver;
pub use sensitivity::{
    noise_floor_dbm, rolloff_penalty_db, sensitivity_bandpass_dbm, sensitivity_dbm,
    sensitivity_from_snr_dbm, sensitivity_matched_filter_dbm,
};
pub use transmitter::Transmitter;
