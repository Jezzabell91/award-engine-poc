//! Day detection and shift segmentation logic.
//!
//! This module provides utilities for determining the day type (weekday, Saturday, Sunday)
//! for any datetime and for splitting shifts at midnight boundaries for correct penalty
//! rate application.

use chrono::{Datelike, NaiveDateTime, Weekday};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::models::Shift;

/// Represents the type of day for penalty rate calculation.
///
/// Used to determine which penalty rates apply to hours worked.
/// Per Aged Care Award 2010 clause 23, different rates apply for
/// Saturday and Sunday work.
///
/// # Example
///
/// ```
/// use award_engine::calculation::DayType;
///
/// let day_type = DayType::Saturday;
/// assert_eq!(format!("{:?}", day_type), "Saturday");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DayType {
    /// Monday through Friday - ordinary time rates apply.
    Weekday,
    /// Saturday - 150% for non-casuals, 175% for casuals (clause 23.1, 23.2(a)).
    Saturday,
    /// Sunday - 175% for non-casuals, 200% for casuals (clause 23.1, 23.2(b)).
    Sunday,
}

impl std::fmt::Display for DayType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DayType::Weekday => write!(f, "Weekday"),
            DayType::Saturday => write!(f, "Saturday"),
            DayType::Sunday => write!(f, "Sunday"),
        }
    }
}

/// Determines the day type for a given datetime.
///
/// Returns the appropriate [`DayType`] based on the day of the week
/// of the provided datetime.
///
/// # Arguments
///
/// * `datetime` - The datetime to check
///
/// # Returns
///
/// The [`DayType`] for the given datetime:
/// - [`DayType::Weekday`] for Monday through Friday
/// - [`DayType::Saturday`] for Saturday
/// - [`DayType::Sunday`] for Sunday
///
/// # Example
///
/// ```
/// use award_engine::calculation::get_day_type;
/// use award_engine::calculation::DayType;
/// use chrono::NaiveDateTime;
///
/// // 2026-01-17 is a Saturday
/// let saturday = NaiveDateTime::parse_from_str("2026-01-17 15:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
/// assert_eq!(get_day_type(saturday), DayType::Saturday);
///
/// // 2026-01-18 is a Sunday
/// let sunday = NaiveDateTime::parse_from_str("2026-01-18 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
/// assert_eq!(get_day_type(sunday), DayType::Sunday);
///
/// // 2026-01-12 is a Monday
/// let monday = NaiveDateTime::parse_from_str("2026-01-12 09:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
/// assert_eq!(get_day_type(monday), DayType::Weekday);
/// ```
pub fn get_day_type(datetime: NaiveDateTime) -> DayType {
    match datetime.weekday() {
        Weekday::Sat => DayType::Saturday,
        Weekday::Sun => DayType::Sunday,
        _ => DayType::Weekday,
    }
}

/// Represents a segment of a shift within a single day.
///
/// When a shift crosses midnight, it is split into multiple segments,
/// each belonging to a single day. This allows correct penalty rates
/// to be applied to each portion of the shift.
///
/// # Example
///
/// ```
/// use award_engine::calculation::{ShiftSegment, DayType};
/// use chrono::NaiveDateTime;
/// use rust_decimal::Decimal;
///
/// let segment = ShiftSegment {
///     start_time: NaiveDateTime::parse_from_str("2026-01-17 22:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
///     end_time: NaiveDateTime::parse_from_str("2026-01-18 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
///     day_type: DayType::Saturday,
///     hours: Decimal::new(20, 1), // 2.0 hours
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShiftSegment {
    /// The start time of this segment.
    pub start_time: NaiveDateTime,
    /// The end time of this segment.
    pub end_time: NaiveDateTime,
    /// The day type for this segment (determines penalty rates).
    pub day_type: DayType,
    /// The number of worked hours in this segment.
    pub hours: Decimal,
}

/// Segments a shift by day boundaries.
///
/// Splits a shift at midnight boundaries, creating separate segments for each
/// calendar day the shift spans. Each segment records its start/end times,
/// day type, and hours worked within that day.
///
/// # Arguments
///
/// * `shift` - The shift to segment
///
/// # Returns
///
/// A vector of [`ShiftSegment`]s, ordered chronologically. The sum of all
/// segment hours equals the shift's total worked hours (excluding unpaid breaks).
///
/// # Behavior
///
/// - A shift entirely within one day returns a single segment
/// - A shift crossing midnight returns two segments (before and after midnight)
/// - Segments are ordered chronologically
/// - Each segment's day_type matches the day it falls on
/// - Unpaid breaks are NOT considered in segmentation (they are handled at shift level)
///
/// # Example
///
/// ```
/// use award_engine::calculation::{segment_by_day, DayType};
/// use award_engine::models::Shift;
/// use chrono::{NaiveDate, NaiveDateTime};
/// use rust_decimal::Decimal;
///
/// // A shift crossing Saturday to Sunday
/// let shift = Shift {
///     id: "shift_001".to_string(),
///     date: NaiveDate::from_ymd_opt(2026, 1, 17).unwrap(),
///     start_time: NaiveDateTime::parse_from_str("2026-01-17 22:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
///     end_time: NaiveDateTime::parse_from_str("2026-01-18 06:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
///     breaks: vec![],
/// };
///
/// let segments = segment_by_day(&shift);
/// assert_eq!(segments.len(), 2);
/// assert_eq!(segments[0].day_type, DayType::Saturday);
/// assert_eq!(segments[0].hours, Decimal::new(20, 1)); // 2.0 hours
/// assert_eq!(segments[1].day_type, DayType::Sunday);
/// assert_eq!(segments[1].hours, Decimal::new(60, 1)); // 6.0 hours
/// ```
pub fn segment_by_day(shift: &Shift) -> Vec<ShiftSegment> {
    let mut segments = Vec::new();
    let mut current_start = shift.start_time;
    let shift_end = shift.end_time;

    // If shift doesn't cross midnight, return single segment
    if current_start.date() == shift_end.date() || current_start == shift_end {
        let hours = calculate_hours(current_start, shift_end);
        if hours > Decimal::ZERO {
            segments.push(ShiftSegment {
                start_time: current_start,
                end_time: shift_end,
                day_type: get_day_type(current_start),
                hours,
            });
        }
        return segments;
    }

    // Handle shifts crossing one or more midnights
    while current_start < shift_end {
        // Calculate midnight at the end of the current day
        let next_midnight = (current_start.date() + chrono::Duration::days(1))
            .and_hms_opt(0, 0, 0)
            .expect("Valid midnight time");

        // Segment ends at either midnight or shift end, whichever is first
        let segment_end = if next_midnight <= shift_end {
            next_midnight
        } else {
            shift_end
        };

        let hours = calculate_hours(current_start, segment_end);
        if hours > Decimal::ZERO {
            segments.push(ShiftSegment {
                start_time: current_start,
                end_time: segment_end,
                day_type: get_day_type(current_start),
                hours,
            });
        }

        current_start = segment_end;
    }

    segments
}

/// Calculates the number of hours between two datetimes.
///
/// # Arguments
///
/// * `start` - The start datetime
/// * `end` - The end datetime
///
/// # Returns
///
/// The number of hours as a [`Decimal`].
fn calculate_hours(start: NaiveDateTime, end: NaiveDateTime) -> Decimal {
    let duration_minutes = (end - start).num_minutes();
    Decimal::new(duration_minutes, 0) / Decimal::new(60, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::str::FromStr;

    fn make_datetime(date_str: &str, time_str: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(&format!("{} {}", date_str, time_str), "%Y-%m-%d %H:%M:%S")
            .unwrap()
    }

    fn make_date(date_str: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap()
    }

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    // ==========================================================================
    // DD-001: Monday is Weekday
    // ==========================================================================
    #[test]
    fn test_dd_001_monday_is_weekday() {
        // 2026-01-12 is a Monday
        let datetime = make_datetime("2026-01-12", "09:00:00");
        assert_eq!(get_day_type(datetime), DayType::Weekday);
    }

    // ==========================================================================
    // DD-002: Saturday is Saturday
    // ==========================================================================
    #[test]
    fn test_dd_002_saturday_is_saturday() {
        // 2026-01-17 is a Saturday
        let datetime = make_datetime("2026-01-17", "15:00:00");
        assert_eq!(get_day_type(datetime), DayType::Saturday);
    }

    // ==========================================================================
    // DD-003: Sunday is Sunday
    // ==========================================================================
    #[test]
    fn test_dd_003_sunday_is_sunday() {
        // 2026-01-18 is a Sunday
        let datetime = make_datetime("2026-01-18", "08:00:00");
        assert_eq!(get_day_type(datetime), DayType::Sunday);
    }

    // ==========================================================================
    // DD-004: Saturday 23:59 is Saturday
    // ==========================================================================
    #[test]
    fn test_dd_004_saturday_2359_is_saturday() {
        // 2026-01-17 23:59 is still Saturday
        let datetime = make_datetime("2026-01-17", "23:59:00");
        assert_eq!(get_day_type(datetime), DayType::Saturday);
    }

    // ==========================================================================
    // DD-005: Sunday 00:00 is Sunday
    // ==========================================================================
    #[test]
    fn test_dd_005_sunday_0000_is_sunday() {
        // 2026-01-18 00:00 is Sunday
        let datetime = make_datetime("2026-01-18", "00:00:00");
        assert_eq!(get_day_type(datetime), DayType::Sunday);
    }

    // ==========================================================================
    // DD-006: Weekday shift returns single segment
    // ==========================================================================
    #[test]
    fn test_dd_006_weekday_shift_single_segment() {
        // Wednesday 09:00 to 17:00 (2026-01-14 is a Wednesday)
        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-14"),
            start_time: make_datetime("2026-01-14", "09:00:00"),
            end_time: make_datetime("2026-01-14", "17:00:00"),
            breaks: vec![],
        };

        let segments = segment_by_day(&shift);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].day_type, DayType::Weekday);
        assert_eq!(segments[0].hours, dec("8.0"));
    }

    // ==========================================================================
    // DD-007: Overnight shift returns two segments
    // ==========================================================================
    #[test]
    fn test_dd_007_overnight_shift_two_segments() {
        // Saturday 22:00 to Sunday 06:00
        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-17"),
            start_time: make_datetime("2026-01-17", "22:00:00"),
            end_time: make_datetime("2026-01-18", "06:00:00"),
            breaks: vec![],
        };

        let segments = segment_by_day(&shift);
        assert_eq!(segments.len(), 2);

        // First segment: Saturday 22:00 to 00:00 (2 hours)
        assert_eq!(segments[0].day_type, DayType::Saturday);
        assert_eq!(segments[0].hours, dec("2.0"));
        assert_eq!(
            segments[0].start_time,
            make_datetime("2026-01-17", "22:00:00")
        );
        assert_eq!(
            segments[0].end_time,
            make_datetime("2026-01-18", "00:00:00")
        );

        // Second segment: Sunday 00:00 to 06:00 (6 hours)
        assert_eq!(segments[1].day_type, DayType::Sunday);
        assert_eq!(segments[1].hours, dec("6.0"));
        assert_eq!(
            segments[1].start_time,
            make_datetime("2026-01-18", "00:00:00")
        );
        assert_eq!(
            segments[1].end_time,
            make_datetime("2026-01-18", "06:00:00")
        );
    }

    // ==========================================================================
    // Additional tests for all weekdays
    // ==========================================================================
    #[test]
    fn test_tuesday_is_weekday() {
        // 2026-01-13 is a Tuesday
        let datetime = make_datetime("2026-01-13", "10:00:00");
        assert_eq!(get_day_type(datetime), DayType::Weekday);
    }

    #[test]
    fn test_wednesday_is_weekday() {
        // 2026-01-14 is a Wednesday
        let datetime = make_datetime("2026-01-14", "10:00:00");
        assert_eq!(get_day_type(datetime), DayType::Weekday);
    }

    #[test]
    fn test_thursday_is_weekday() {
        // 2026-01-15 is a Thursday
        let datetime = make_datetime("2026-01-15", "10:00:00");
        assert_eq!(get_day_type(datetime), DayType::Weekday);
    }

    #[test]
    fn test_friday_is_weekday() {
        // 2026-01-16 is a Friday
        let datetime = make_datetime("2026-01-16", "10:00:00");
        assert_eq!(get_day_type(datetime), DayType::Weekday);
    }

    // ==========================================================================
    // Tests for segment_by_day edge cases
    // ==========================================================================
    #[test]
    fn test_saturday_only_shift() {
        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-17"),
            start_time: make_datetime("2026-01-17", "09:00:00"),
            end_time: make_datetime("2026-01-17", "17:00:00"),
            breaks: vec![],
        };

        let segments = segment_by_day(&shift);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].day_type, DayType::Saturday);
        assert_eq!(segments[0].hours, dec("8.0"));
    }

    #[test]
    fn test_sunday_only_shift() {
        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-18"),
            start_time: make_datetime("2026-01-18", "08:00:00"),
            end_time: make_datetime("2026-01-18", "16:00:00"),
            breaks: vec![],
        };

        let segments = segment_by_day(&shift);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].day_type, DayType::Sunday);
        assert_eq!(segments[0].hours, dec("8.0"));
    }

    #[test]
    fn test_friday_to_saturday_overnight() {
        // Friday 22:00 to Saturday 06:00
        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-16"),
            start_time: make_datetime("2026-01-16", "22:00:00"),
            end_time: make_datetime("2026-01-17", "06:00:00"),
            breaks: vec![],
        };

        let segments = segment_by_day(&shift);
        assert_eq!(segments.len(), 2);

        // First segment: Friday 22:00 to 00:00 (2 hours)
        assert_eq!(segments[0].day_type, DayType::Weekday);
        assert_eq!(segments[0].hours, dec("2.0"));

        // Second segment: Saturday 00:00 to 06:00 (6 hours)
        assert_eq!(segments[1].day_type, DayType::Saturday);
        assert_eq!(segments[1].hours, dec("6.0"));
    }

    #[test]
    fn test_sunday_to_monday_overnight() {
        // Sunday 22:00 to Monday 06:00
        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-18"),
            start_time: make_datetime("2026-01-18", "22:00:00"),
            end_time: make_datetime("2026-01-19", "06:00:00"),
            breaks: vec![],
        };

        let segments = segment_by_day(&shift);
        assert_eq!(segments.len(), 2);

        // First segment: Sunday 22:00 to 00:00 (2 hours)
        assert_eq!(segments[0].day_type, DayType::Sunday);
        assert_eq!(segments[0].hours, dec("2.0"));

        // Second segment: Monday 00:00 to 06:00 (6 hours)
        assert_eq!(segments[1].day_type, DayType::Weekday);
        assert_eq!(segments[1].hours, dec("6.0"));
    }

    #[test]
    fn test_segment_hours_sum_equals_shift_worked_hours() {
        // Verify that the sum of segment hours equals the total worked hours
        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-17"),
            start_time: make_datetime("2026-01-17", "22:00:00"),
            end_time: make_datetime("2026-01-18", "06:00:00"),
            breaks: vec![],
        };

        let segments = segment_by_day(&shift);
        let segment_total: Decimal = segments.iter().map(|s| s.hours).sum();
        assert_eq!(segment_total, shift.worked_hours());
    }

    #[test]
    fn test_segments_ordered_chronologically() {
        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-17"),
            start_time: make_datetime("2026-01-17", "22:00:00"),
            end_time: make_datetime("2026-01-18", "06:00:00"),
            breaks: vec![],
        };

        let segments = segment_by_day(&shift);
        for i in 1..segments.len() {
            assert!(segments[i - 1].end_time <= segments[i].start_time);
        }
    }

    #[test]
    fn test_no_segment_crosses_midnight() {
        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-17"),
            start_time: make_datetime("2026-01-17", "22:00:00"),
            end_time: make_datetime("2026-01-18", "06:00:00"),
            breaks: vec![],
        };

        let segments = segment_by_day(&shift);
        for segment in &segments {
            assert_eq!(
                segment.start_time.date(),
                segment
                    .end_time
                    .date()
                    .pred_opt()
                    .map(|d| {
                        if segment.end_time.time()
                            == chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap()
                        {
                            d
                        } else {
                            segment.end_time.date()
                        }
                    })
                    .unwrap_or(segment.end_time.date())
                    .max(segment.start_time.date()),
                "Segment should not cross midnight: {:?}",
                segment
            );
        }
    }

    #[test]
    fn test_zero_duration_shift() {
        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-17"),
            start_time: make_datetime("2026-01-17", "09:00:00"),
            end_time: make_datetime("2026-01-17", "09:00:00"),
            breaks: vec![],
        };

        let segments = segment_by_day(&shift);
        assert!(segments.is_empty());
    }

    #[test]
    fn test_day_type_display() {
        assert_eq!(format!("{}", DayType::Weekday), "Weekday");
        assert_eq!(format!("{}", DayType::Saturday), "Saturday");
        assert_eq!(format!("{}", DayType::Sunday), "Sunday");
    }

    #[test]
    fn test_day_type_serialization() {
        let saturday = DayType::Saturday;
        let json = serde_json::to_string(&saturday).unwrap();
        assert_eq!(json, "\"saturday\"");

        let deserialized: DayType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, DayType::Saturday);
    }

    #[test]
    fn test_shift_segment_serialization() {
        let segment = ShiftSegment {
            start_time: make_datetime("2026-01-17", "22:00:00"),
            end_time: make_datetime("2026-01-18", "00:00:00"),
            day_type: DayType::Saturday,
            hours: dec("2.0"),
        };

        let json = serde_json::to_string(&segment).unwrap();
        assert!(json.contains("\"day_type\":\"saturday\""));
        assert!(json.contains("\"hours\":\"2.0\""));

        let deserialized: ShiftSegment = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, segment);
    }
}
