//! Award Interpretation Engine for Australian Awards
//!
//! This crate provides functionality for interpreting the Aged Care Award 2010 (MA000018)
//! and calculating pay based on shifts, employee classifications, and award rules.

#![warn(missing_docs)]

pub mod calculation;
pub mod config;
pub mod error;
pub mod models;
