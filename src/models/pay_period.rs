//! Pay period and public holiday models.
//!
//! This module contains the [`PayPeriod`] and [`PublicHoliday`] types used to define
//! the calculation context for pay calculations.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Represents a public holiday within a pay period.
///
/// Public holidays affect penalty rates and are tracked per region
/// to support state-specific holidays in Australia.
///
/// # Example
///
/// ```
/// use award_engine::models::PublicHoliday;
/// use chrono::NaiveDate;
///
/// let holiday = PublicHoliday {
///     date: NaiveDate::from_ymd_opt(2026, 1, 26).unwrap(),
///     name: "Australia Day".to_string(),
///     region: "national".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicHoliday {
    /// The date of the public holiday.
    pub date: NaiveDate,
    /// The name of the public holiday (e.g., "Australia Day").
    pub name: String,
    /// The region where this holiday applies (e.g., "national", "VIC", "NSW").
    pub region: String,
}

/// Represents a pay period with its date range and associated public holidays.
///
/// A pay period defines the time window for pay calculations and includes
/// information about any public holidays that fall within the period.
///
/// # Example
///
/// ```
/// use award_engine::models::{PayPeriod, PublicHoliday};
/// use chrono::NaiveDate;
///
/// let pay_period = PayPeriod {
///     start_date: NaiveDate::from_ymd_opt(2026, 1, 13).unwrap(),
///     end_date: NaiveDate::from_ymd_opt(2026, 1, 26).unwrap(),
///     public_holidays: vec![
///         PublicHoliday {
///             date: NaiveDate::from_ymd_opt(2026, 1, 26).unwrap(),
///             name: "Australia Day".to_string(),
///             region: "national".to_string(),
///         }
///     ],
/// };
///
/// assert!(pay_period.contains_date(NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()));
/// assert!(pay_period.is_public_holiday(NaiveDate::from_ymd_opt(2026, 1, 26).unwrap()));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayPeriod {
    /// The start date of the pay period (inclusive).
    pub start_date: NaiveDate,
    /// The end date of the pay period (inclusive).
    pub end_date: NaiveDate,
    /// Public holidays that fall within this pay period.
    pub public_holidays: Vec<PublicHoliday>,
}

impl PayPeriod {
    /// Checks if a given date falls within this pay period.
    ///
    /// The check is inclusive of both start and end dates.
    ///
    /// # Arguments
    ///
    /// * `date` - The date to check.
    ///
    /// # Returns
    ///
    /// `true` if the date is within the pay period (inclusive), `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use award_engine::models::PayPeriod;
    /// use chrono::NaiveDate;
    ///
    /// let period = PayPeriod {
    ///     start_date: NaiveDate::from_ymd_opt(2026, 1, 13).unwrap(),
    ///     end_date: NaiveDate::from_ymd_opt(2026, 1, 26).unwrap(),
    ///     public_holidays: vec![],
    /// };
    ///
    /// assert!(period.contains_date(NaiveDate::from_ymd_opt(2026, 1, 13).unwrap())); // start date
    /// assert!(period.contains_date(NaiveDate::from_ymd_opt(2026, 1, 20).unwrap())); // middle
    /// assert!(period.contains_date(NaiveDate::from_ymd_opt(2026, 1, 26).unwrap())); // end date
    /// assert!(!period.contains_date(NaiveDate::from_ymd_opt(2026, 1, 12).unwrap())); // before
    /// assert!(!period.contains_date(NaiveDate::from_ymd_opt(2026, 1, 27).unwrap())); // after
    /// ```
    pub fn contains_date(&self, date: NaiveDate) -> bool {
        date >= self.start_date && date <= self.end_date
    }

    /// Checks if a given date is a public holiday within this pay period.
    ///
    /// # Arguments
    ///
    /// * `date` - The date to check.
    ///
    /// # Returns
    ///
    /// `true` if the date matches any public holiday in the pay period, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use award_engine::models::{PayPeriod, PublicHoliday};
    /// use chrono::NaiveDate;
    ///
    /// let period = PayPeriod {
    ///     start_date: NaiveDate::from_ymd_opt(2026, 1, 13).unwrap(),
    ///     end_date: NaiveDate::from_ymd_opt(2026, 1, 26).unwrap(),
    ///     public_holidays: vec![
    ///         PublicHoliday {
    ///             date: NaiveDate::from_ymd_opt(2026, 1, 26).unwrap(),
    ///             name: "Australia Day".to_string(),
    ///             region: "national".to_string(),
    ///         }
    ///     ],
    /// };
    ///
    /// assert!(period.is_public_holiday(NaiveDate::from_ymd_opt(2026, 1, 26).unwrap()));
    /// assert!(!period.is_public_holiday(NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()));
    /// ```
    pub fn is_public_holiday(&self, date: NaiveDate) -> bool {
        self.public_holidays.iter().any(|h| h.date == date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_pay_period_with_holiday() -> PayPeriod {
        PayPeriod {
            start_date: NaiveDate::from_ymd_opt(2026, 1, 13).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 1, 26).unwrap(),
            public_holidays: vec![PublicHoliday {
                date: NaiveDate::from_ymd_opt(2026, 1, 26).unwrap(),
                name: "Australia Day".to_string(),
                region: "national".to_string(),
            }],
        }
    }

    fn create_pay_period_no_holidays() -> PayPeriod {
        PayPeriod {
            start_date: NaiveDate::from_ymd_opt(2026, 1, 13).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 1, 26).unwrap(),
            public_holidays: vec![],
        }
    }

    /// PP-001: contains_date within period
    #[test]
    fn test_contains_date_within_period() {
        let period = create_pay_period_no_holidays();
        let test_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        assert!(period.contains_date(test_date));
    }

    /// PP-002: contains_date outside period
    #[test]
    fn test_contains_date_outside_period() {
        let period = create_pay_period_no_holidays();
        let test_date = NaiveDate::from_ymd_opt(2026, 1, 27).unwrap();
        assert!(!period.contains_date(test_date));
    }

    /// PP-003: is_public_holiday returns true
    #[test]
    fn test_is_public_holiday_returns_true() {
        let period = create_pay_period_with_holiday();
        let test_date = NaiveDate::from_ymd_opt(2026, 1, 26).unwrap();
        assert!(period.is_public_holiday(test_date));
    }

    /// PP-004: is_public_holiday returns false
    #[test]
    fn test_is_public_holiday_returns_false() {
        let period = create_pay_period_no_holidays();
        let test_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
        assert!(!period.is_public_holiday(test_date));
    }

    #[test]
    fn test_contains_date_on_start_date() {
        let period = create_pay_period_no_holidays();
        assert!(period.contains_date(period.start_date));
    }

    #[test]
    fn test_contains_date_on_end_date() {
        let period = create_pay_period_no_holidays();
        assert!(period.contains_date(period.end_date));
    }

    #[test]
    fn test_contains_date_before_start() {
        let period = create_pay_period_no_holidays();
        let test_date = NaiveDate::from_ymd_opt(2026, 1, 12).unwrap();
        assert!(!period.contains_date(test_date));
    }

    #[test]
    fn test_serialize_pay_period() {
        let period = create_pay_period_with_holiday();
        let json = serde_json::to_string(&period).unwrap();
        assert!(json.contains("\"start_date\":\"2026-01-13\""));
        assert!(json.contains("\"end_date\":\"2026-01-26\""));
        assert!(json.contains("\"name\":\"Australia Day\""));
    }

    #[test]
    fn test_deserialize_pay_period() {
        let json = r#"{
            "start_date": "2026-01-13",
            "end_date": "2026-01-26",
            "public_holidays": [
                {
                    "date": "2026-01-26",
                    "name": "Australia Day",
                    "region": "national"
                }
            ]
        }"#;
        let period: PayPeriod = serde_json::from_str(json).unwrap();
        assert_eq!(
            period.start_date,
            NaiveDate::from_ymd_opt(2026, 1, 13).unwrap()
        );
        assert_eq!(
            period.end_date,
            NaiveDate::from_ymd_opt(2026, 1, 26).unwrap()
        );
        assert_eq!(period.public_holidays.len(), 1);
        assert_eq!(period.public_holidays[0].name, "Australia Day");
    }

    #[test]
    fn test_serialize_public_holiday() {
        let holiday = PublicHoliday {
            date: NaiveDate::from_ymd_opt(2026, 1, 26).unwrap(),
            name: "Australia Day".to_string(),
            region: "national".to_string(),
        };
        let json = serde_json::to_string(&holiday).unwrap();
        assert!(json.contains("\"date\":\"2026-01-26\""));
        assert!(json.contains("\"name\":\"Australia Day\""));
        assert!(json.contains("\"region\":\"national\""));
    }

    #[test]
    fn test_deserialize_public_holiday() {
        let json = r#"{
            "date": "2026-12-25",
            "name": "Christmas Day",
            "region": "national"
        }"#;
        let holiday: PublicHoliday = serde_json::from_str(json).unwrap();
        assert_eq!(holiday.date, NaiveDate::from_ymd_opt(2026, 12, 25).unwrap());
        assert_eq!(holiday.name, "Christmas Day");
        assert_eq!(holiday.region, "national");
    }

    #[test]
    fn test_multiple_public_holidays() {
        let period = PayPeriod {
            start_date: NaiveDate::from_ymd_opt(2026, 12, 20).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2027, 1, 3).unwrap(),
            public_holidays: vec![
                PublicHoliday {
                    date: NaiveDate::from_ymd_opt(2026, 12, 25).unwrap(),
                    name: "Christmas Day".to_string(),
                    region: "national".to_string(),
                },
                PublicHoliday {
                    date: NaiveDate::from_ymd_opt(2026, 12, 26).unwrap(),
                    name: "Boxing Day".to_string(),
                    region: "national".to_string(),
                },
                PublicHoliday {
                    date: NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
                    name: "New Year's Day".to_string(),
                    region: "national".to_string(),
                },
            ],
        };

        assert!(period.is_public_holiday(NaiveDate::from_ymd_opt(2026, 12, 25).unwrap()));
        assert!(period.is_public_holiday(NaiveDate::from_ymd_opt(2026, 12, 26).unwrap()));
        assert!(period.is_public_holiday(NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()));
        assert!(!period.is_public_holiday(NaiveDate::from_ymd_opt(2026, 12, 24).unwrap()));
    }
}
