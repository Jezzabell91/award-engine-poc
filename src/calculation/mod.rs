//! Calculation logic for the Award Interpretation Engine.
//!
//! This module contains all the calculation functions for determining pay,
//! including base rate lookup, casual loading, and ordinary hours calculations.

mod base_rate;

pub use base_rate::{get_base_rate, BaseRateLookupResult};
