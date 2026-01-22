//! Calculation logic for the Award Interpretation Engine.
//!
//! This module contains all the calculation functions for determining pay,
//! including base rate lookup, casual loading, and ordinary hours calculations.

mod base_rate;
mod casual_loading;
mod ordinary_hours;

pub use base_rate::{BaseRateLookupResult, get_base_rate};
pub use casual_loading::{CasualLoadingResult, apply_casual_loading, casual_loading_multiplier};
pub use ordinary_hours::{OrdinaryHoursResult, calculate_ordinary_hours};
