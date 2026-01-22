//! Calculation logic for the Award Interpretation Engine.
//!
//! This module contains all the calculation functions for determining pay,
//! including base rate lookup, casual loading, ordinary hours calculations,
//! day detection for weekend penalty rates, Saturday penalty rates, Sunday penalty rates,
//! overnight shift calculations that span multiple days, daily overtime detection,
//! weekday overtime rate calculation, weekend overtime rate calculation, and
//! laundry allowance calculation.

mod base_rate;
mod casual_loading;
mod daily_overtime;
mod day_detection;
mod laundry_allowance;
mod ordinary_hours;
mod overnight_shift;
mod overtime_audit;
mod saturday_penalty;
mod sunday_penalty;
mod weekday_overtime;
mod weekend_overtime;

pub use base_rate::{BaseRateLookupResult, get_base_rate};
pub use casual_loading::{CasualLoadingResult, apply_casual_loading, casual_loading_multiplier};
pub use daily_overtime::{
    DEFAULT_DAILY_OVERTIME_THRESHOLD, DailyOvertimeDetection, detect_daily_overtime,
};
pub use day_detection::{DayType, ShiftSegment, get_day_type, segment_by_day};
pub use ordinary_hours::{OrdinaryHoursResult, calculate_ordinary_hours};
pub use overnight_shift::{OvernightShiftResult, calculate_overnight_shift};
pub use saturday_penalty::{SaturdayPayResult, calculate_saturday_pay};
pub use sunday_penalty::{SundayPayResult, calculate_sunday_pay};
pub use weekday_overtime::{
    WEEKDAY_OT_TIER_1_THRESHOLD, WeekdayOvertimeResult, calculate_weekday_overtime,
};
pub use weekend_overtime::{WeekendOvertimeResult, calculate_weekend_overtime};
pub use laundry_allowance::{
    LAUNDRY_ALLOWANCE_CLAUSE, LAUNDRY_ALLOWANCE_TAG, LaundryAllowanceResult,
    calculate_laundry_allowance,
};
