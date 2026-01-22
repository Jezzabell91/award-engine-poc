//! Shift model and related types.
//!
//! This module defines the Shift and Break structs for representing
//! work shifts and breaks in the award interpretation system.

use chrono::{Datelike, NaiveDate, NaiveDateTime, Weekday};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Represents a break taken during a shift.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Break {
    /// The start time of the break.
    pub start_time: NaiveDateTime,
    /// The end time of the break.
    pub end_time: NaiveDateTime,
    /// Whether the break is paid (true) or unpaid (false).
    pub is_paid: bool,
}

impl Break {
    /// Returns the duration of the break in minutes.
    fn duration_minutes(&self) -> i64 {
        (self.end_time - self.start_time).num_minutes()
    }
}

/// Represents a work shift with timing information and breaks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Shift {
    /// Unique identifier for the shift.
    pub id: String,
    /// The date of the shift (used for determining day type).
    pub date: NaiveDate,
    /// The start time of the shift.
    pub start_time: NaiveDateTime,
    /// The end time of the shift.
    pub end_time: NaiveDateTime,
    /// Breaks taken during the shift.
    #[serde(default)]
    pub breaks: Vec<Break>,
}

impl Shift {
    /// Calculates the total worked hours for the shift.
    ///
    /// This method calculates the total duration of the shift and subtracts
    /// any unpaid breaks. Paid breaks are NOT subtracted from the total.
    ///
    /// # Returns
    ///
    /// The number of worked hours as a Decimal.
    ///
    /// # Examples
    ///
    /// ```
    /// use award_engine::models::{Shift, Break};
    /// use chrono::{NaiveDate, NaiveDateTime};
    /// use rust_decimal::Decimal;
    ///
    /// let shift = Shift {
    ///     id: "shift_001".to_string(),
    ///     date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
    ///     start_time: NaiveDateTime::parse_from_str("2026-01-15 09:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
    ///     end_time: NaiveDateTime::parse_from_str("2026-01-15 17:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
    ///     breaks: vec![],
    /// };
    /// assert_eq!(shift.worked_hours(), Decimal::new(80, 1)); // 8.0 hours
    /// ```
    pub fn worked_hours(&self) -> Decimal {
        // Calculate total shift duration in minutes
        let total_minutes = (self.end_time - self.start_time).num_minutes();

        // Calculate total unpaid break minutes
        let unpaid_break_minutes: i64 = self
            .breaks
            .iter()
            .filter(|b| !b.is_paid)
            .map(|b| b.duration_minutes())
            .sum();

        // Worked minutes = total - unpaid breaks
        let worked_minutes = total_minutes - unpaid_break_minutes;

        // Convert minutes to hours as Decimal
        Decimal::new(worked_minutes, 0) / Decimal::new(60, 0)
    }

    /// Returns the day of the week for the shift.
    ///
    /// # Returns
    ///
    /// The weekday (Monday through Sunday) of the shift date.
    ///
    /// # Examples
    ///
    /// ```
    /// use award_engine::models::Shift;
    /// use chrono::{NaiveDate, NaiveDateTime, Weekday};
    ///
    /// let shift = Shift {
    ///     id: "shift_001".to_string(),
    ///     date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(), // Thursday
    ///     start_time: NaiveDateTime::parse_from_str("2026-01-15 09:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
    ///     end_time: NaiveDateTime::parse_from_str("2026-01-15 17:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
    ///     breaks: vec![],
    /// };
    /// assert_eq!(shift.day_of_week(), Weekday::Thu);
    /// ```
    pub fn day_of_week(&self) -> Weekday {
        self.date.weekday()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_datetime(date_str: &str, time_str: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(&format!("{} {}", date_str, time_str), "%Y-%m-%d %H:%M:%S")
            .unwrap()
    }

    fn make_date(date_str: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap()
    }

    /// SH-001: 8 hour shift no breaks
    #[test]
    fn test_8_hour_shift_no_breaks() {
        let shift = Shift {
            id: "SH-001".to_string(),
            date: make_date("2026-01-15"),
            start_time: make_datetime("2026-01-15", "09:00:00"),
            end_time: make_datetime("2026-01-15", "17:00:00"),
            breaks: vec![],
        };

        assert_eq!(shift.worked_hours(), Decimal::new(80, 1)); // 8.0
    }

    /// SH-002: 8.5 hour shift with 30min unpaid break
    #[test]
    fn test_8_5_hour_shift_with_30min_unpaid_break() {
        let shift = Shift {
            id: "SH-002".to_string(),
            date: make_date("2026-01-15"),
            start_time: make_datetime("2026-01-15", "09:00:00"),
            end_time: make_datetime("2026-01-15", "17:30:00"),
            breaks: vec![Break {
                start_time: make_datetime("2026-01-15", "12:00:00"),
                end_time: make_datetime("2026-01-15", "12:30:00"),
                is_paid: false,
            }],
        };

        assert_eq!(shift.worked_hours(), Decimal::new(80, 1)); // 8.0
    }

    /// SH-003: 8.5 hour shift with 30min paid break
    #[test]
    fn test_8_5_hour_shift_with_30min_paid_break() {
        let shift = Shift {
            id: "SH-003".to_string(),
            date: make_date("2026-01-15"),
            start_time: make_datetime("2026-01-15", "09:00:00"),
            end_time: make_datetime("2026-01-15", "17:30:00"),
            breaks: vec![Break {
                start_time: make_datetime("2026-01-15", "12:00:00"),
                end_time: make_datetime("2026-01-15", "12:30:00"),
                is_paid: true,
            }],
        };

        assert_eq!(shift.worked_hours(), Decimal::new(85, 1)); // 8.5
    }

    /// SH-004: overnight shift
    #[test]
    fn test_overnight_shift() {
        let shift = Shift {
            id: "SH-004".to_string(),
            date: make_date("2026-01-15"),
            start_time: make_datetime("2026-01-15", "22:00:00"),
            end_time: make_datetime("2026-01-16", "06:00:00"),
            breaks: vec![],
        };

        assert_eq!(shift.worked_hours(), Decimal::new(80, 1)); // 8.0
    }

    /// SH-005: zero duration shift
    #[test]
    fn test_zero_duration_shift() {
        let shift = Shift {
            id: "SH-005".to_string(),
            date: make_date("2026-01-15"),
            start_time: make_datetime("2026-01-15", "09:00:00"),
            end_time: make_datetime("2026-01-15", "09:00:00"),
            breaks: vec![],
        };

        assert_eq!(shift.worked_hours(), Decimal::new(0, 0)); // 0.0
    }

    #[test]
    fn test_day_of_week() {
        // 2026-01-15 is a Thursday
        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-15"),
            start_time: make_datetime("2026-01-15", "09:00:00"),
            end_time: make_datetime("2026-01-15", "17:00:00"),
            breaks: vec![],
        };
        assert_eq!(shift.day_of_week(), Weekday::Thu);

        // 2026-01-17 is a Saturday
        let saturday_shift = Shift {
            id: "shift_002".to_string(),
            date: make_date("2026-01-17"),
            start_time: make_datetime("2026-01-17", "09:00:00"),
            end_time: make_datetime("2026-01-17", "17:00:00"),
            breaks: vec![],
        };
        assert_eq!(saturday_shift.day_of_week(), Weekday::Sat);

        // 2026-01-18 is a Sunday
        let sunday_shift = Shift {
            id: "shift_003".to_string(),
            date: make_date("2026-01-18"),
            start_time: make_datetime("2026-01-18", "09:00:00"),
            end_time: make_datetime("2026-01-18", "17:00:00"),
            breaks: vec![],
        };
        assert_eq!(sunday_shift.day_of_week(), Weekday::Sun);
    }

    #[test]
    fn test_shift_serialization() {
        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-15"),
            start_time: make_datetime("2026-01-15", "09:00:00"),
            end_time: make_datetime("2026-01-15", "17:00:00"),
            breaks: vec![Break {
                start_time: make_datetime("2026-01-15", "12:00:00"),
                end_time: make_datetime("2026-01-15", "12:30:00"),
                is_paid: false,
            }],
        };

        let json = serde_json::to_string(&shift).unwrap();
        let deserialized: Shift = serde_json::from_str(&json).unwrap();
        assert_eq!(shift, deserialized);
    }

    #[test]
    fn test_shift_deserialization() {
        let json = r#"{
            "id": "shift_001",
            "date": "2026-01-15",
            "start_time": "2026-01-15T09:00:00",
            "end_time": "2026-01-15T17:00:00",
            "breaks": [
                {
                    "start_time": "2026-01-15T12:00:00",
                    "end_time": "2026-01-15T12:30:00",
                    "is_paid": false
                }
            ]
        }"#;

        let shift: Shift = serde_json::from_str(json).unwrap();
        assert_eq!(shift.id, "shift_001");
        assert_eq!(shift.breaks.len(), 1);
        assert!(!shift.breaks[0].is_paid);
    }

    #[test]
    fn test_multiple_breaks() {
        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-15"),
            start_time: make_datetime("2026-01-15", "08:00:00"),
            end_time: make_datetime("2026-01-15", "18:00:00"), // 10 hours total
            breaks: vec![
                Break {
                    start_time: make_datetime("2026-01-15", "10:00:00"),
                    end_time: make_datetime("2026-01-15", "10:15:00"), // 15 min paid
                    is_paid: true,
                },
                Break {
                    start_time: make_datetime("2026-01-15", "12:00:00"),
                    end_time: make_datetime("2026-01-15", "12:30:00"), // 30 min unpaid
                    is_paid: false,
                },
                Break {
                    start_time: make_datetime("2026-01-15", "15:00:00"),
                    end_time: make_datetime("2026-01-15", "15:15:00"), // 15 min unpaid
                    is_paid: false,
                },
            ],
        };

        // 10 hours - 45 min unpaid = 9.25 hours
        // (600 minutes - 45 minutes) / 60 = 555 / 60 = 9.25
        assert_eq!(shift.worked_hours(), Decimal::new(925, 2)); // 9.25
    }
}
