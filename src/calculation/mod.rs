//! Calculation logic for the Award Interpretation Engine.
//!
//! This module contains all the calculation functions for determining pay,
//! including base rate lookup, casual loading, ordinary hours calculations,
//! day detection for weekend penalty rates, Saturday penalty rates, and Sunday penalty rates.

mod base_rate;
mod casual_loading;
mod day_detection;
mod ordinary_hours;
mod saturday_penalty;
mod sunday_penalty;

pub use base_rate::{BaseRateLookupResult, get_base_rate};
pub use casual_loading::{CasualLoadingResult, apply_casual_loading, casual_loading_multiplier};
pub use day_detection::{DayType, ShiftSegment, get_day_type, segment_by_day};
pub use ordinary_hours::{OrdinaryHoursResult, calculate_ordinary_hours};
pub use saturday_penalty::{SaturdayPayResult, calculate_saturday_pay};
pub use sunday_penalty::{SundayPayResult, calculate_sunday_pay};
