//! Weekend overtime rate calculation functionality.
//!
//! This module provides functions for calculating overtime pay on weekend days
//! (Saturday and Sunday) as per the Aged Care Award 2010 clause 25.1(a)(i)(B).
//!
//! ## Rate Structure
//!
//! **Weekend overtime is NOT tiered (unlike weekday overtime):**
//! - All weekend overtime hours: 200% for non-casuals, 250% for casuals (2.0 × 1.25)
//!
//! This differs from weekday overtime where the first 2 hours are at a lower rate.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::calculation::DayType;
use crate::config::AwardConfig;
use crate::models::{AuditStep, Employee, EmploymentType, PayCategory, PayLine};

/// The result of weekend overtime calculation.
///
/// Contains the pay line for weekend overtime and the audit step
/// documenting the calculation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeekendOvertimeResult {
    /// Pay line for weekend overtime (may be None if no overtime hours).
    pub pay_line: Option<PayLine>,
    /// Audit step recording the calculation.
    pub audit_step: Option<AuditStep>,
}

/// Calculates weekend overtime pay at flat 200% (or 250% for casuals).
///
/// Weekend overtime is calculated differently from weekday overtime:
/// - **All hours** are at 200% for non-casuals, 250% for casuals
/// - There is NO tiered rate (unlike weekday overtime)
///
/// # Arguments
///
/// * `overtime_hours` - The total overtime hours to be paid
/// * `base_rate` - The base hourly rate (before any loading)
/// * `employee` - The employee receiving overtime pay
/// * `config` - The award configuration containing overtime multipliers
/// * `day_type` - The type of weekend day (Saturday or Sunday)
/// * `date` - The date of the shift for pay line records
/// * `shift_id` - The shift ID for pay line records
/// * `step_number` - The step number for audit trail sequencing
///
/// # Returns
///
/// A [`WeekendOvertimeResult`] containing:
/// - `pay_line`: Optional pay line (None if overtime_hours <= 0)
/// - `audit_step`: Optional audit step (None if overtime_hours <= 0)
///
/// # Award Reference
///
/// - Clause 25.1(a)(i)(B): Weekend overtime rates
///
/// # Examples
///
/// ## 2 hours Saturday overtime (full-time)
///
/// ```
/// use award_engine::calculation::{calculate_weekend_overtime, DayType};
/// use award_engine::config::ConfigLoader;
/// use award_engine::models::{Employee, EmploymentType, PayCategory};
/// use chrono::NaiveDate;
/// use rust_decimal::Decimal;
/// use std::str::FromStr;
///
/// let config = ConfigLoader::load("config/ma000018").unwrap().config().clone();
/// let employee = Employee {
///     id: "emp_001".to_string(),
///     employment_type: EmploymentType::FullTime,
///     classification_code: "dce_level_3".to_string(),
///     date_of_birth: NaiveDate::from_ymd_opt(1990, 1, 15).unwrap(),
///     employment_start_date: NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
///     base_hourly_rate: None,
///     tags: vec![],
/// };
/// let date = NaiveDate::from_ymd_opt(2026, 1, 17).unwrap(); // Saturday
///
/// let result = calculate_weekend_overtime(
///     Decimal::from_str("2.0").unwrap(),
///     Decimal::from_str("28.54").unwrap(),
///     &employee,
///     &config,
///     DayType::Saturday,
///     date,
///     "shift_001",
///     1,
/// );
///
/// assert!(result.pay_line.is_some());
/// let pay_line = result.pay_line.unwrap();
/// assert_eq!(pay_line.category, PayCategory::Overtime200);
/// // 2h × ($28.54 × 2.0) = 2h × $57.08 = $114.16
/// assert_eq!(pay_line.amount, Decimal::from_str("114.16").unwrap());
/// ```
#[allow(clippy::too_many_arguments)]
pub fn calculate_weekend_overtime(
    overtime_hours: Decimal,
    base_rate: Decimal,
    employee: &Employee,
    config: &AwardConfig,
    day_type: DayType,
    date: NaiveDate,
    shift_id: &str,
    step_number: u32,
) -> WeekendOvertimeResult {
    // If no overtime, return empty result
    if overtime_hours <= Decimal::ZERO {
        return WeekendOvertimeResult {
            pay_line: None,
            audit_step: None,
        };
    }

    // Get weekend overtime rates from config
    let weekend_overtime = &config.penalties().overtime.weekend;

    // Determine multiplier based on day type and employment type
    let multiplier = match day_type {
        DayType::Saturday => match employee.employment_type {
            EmploymentType::FullTime => weekend_overtime.saturday.full_time,
            EmploymentType::PartTime => weekend_overtime.saturday.part_time,
            EmploymentType::Casual => weekend_overtime.saturday.casual,
        },
        DayType::Sunday => match employee.employment_type {
            EmploymentType::FullTime => weekend_overtime.sunday.full_time,
            EmploymentType::PartTime => weekend_overtime.sunday.part_time,
            EmploymentType::Casual => weekend_overtime.sunday.casual,
        },
        DayType::Weekday => {
            // Weekend overtime should not be called for weekdays
            // but handle gracefully by returning empty result
            return WeekendOvertimeResult {
                pay_line: None,
                audit_step: None,
            };
        }
    };

    let employment_type_str = match employee.employment_type {
        EmploymentType::FullTime => "full_time",
        EmploymentType::PartTime => "part_time",
        EmploymentType::Casual => "casual",
    };

    let day_type_str = match day_type {
        DayType::Saturday => "Saturday",
        DayType::Sunday => "Sunday",
        DayType::Weekday => "Weekday",
    };

    let rate = base_rate * multiplier;
    let amount = overtime_hours * rate;

    let reasoning = if employee.is_casual() {
        format!(
            "{} overtime: {} hours at {}% ({}% × 1.25 casual loading): {} hours × ${} = ${}",
            day_type_str,
            overtime_hours.normalize(),
            (multiplier * Decimal::from(100)).normalize(),
            Decimal::from(200),
            overtime_hours.normalize(),
            rate.normalize(),
            amount.normalize()
        )
    } else {
        format!(
            "{} overtime: {} hours at {}%: {} hours × ${} = ${}",
            day_type_str,
            overtime_hours.normalize(),
            (multiplier * Decimal::from(100)).normalize(),
            overtime_hours.normalize(),
            rate.normalize(),
            amount.normalize()
        )
    };

    let audit_step = AuditStep {
        step_number,
        rule_id: "weekend_overtime".to_string(),
        rule_name: format!("{} Overtime", day_type_str),
        clause_ref: weekend_overtime.clause.clone(),
        input: serde_json::json!({
            "hours": overtime_hours.normalize().to_string(),
            "base_rate": base_rate.normalize().to_string(),
            "employment_type": employment_type_str,
            "day_type": day_type_str
        }),
        output: serde_json::json!({
            "multiplier": multiplier.normalize().to_string(),
            "rate": rate.normalize().to_string(),
            "amount": amount.normalize().to_string()
        }),
        reasoning,
    };

    let pay_line = PayLine {
        date,
        shift_id: shift_id.to_string(),
        category: PayCategory::Overtime200,
        hours: overtime_hours,
        rate,
        amount,
        clause_ref: weekend_overtime.clause.clone(),
    };

    WeekendOvertimeResult {
        pay_line: Some(pay_line),
        audit_step: Some(audit_step),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigLoader;
    use std::str::FromStr;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn create_test_employee(employment_type: EmploymentType) -> Employee {
        Employee {
            id: "emp_001".to_string(),
            employment_type,
            classification_code: "dce_level_3".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1990, 1, 15).unwrap(),
            employment_start_date: NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
            base_hourly_rate: None,
            tags: vec![],
        }
    }

    fn saturday_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 1, 17).unwrap() // Saturday
    }

    fn sunday_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 1, 18).unwrap() // Sunday
    }

    fn load_config() -> AwardConfig {
        ConfigLoader::load("config/ma000018")
            .unwrap()
            .config()
            .clone()
    }

    // ==========================================================================
    // SATOT-001: fulltime 10h Saturday - 2h overtime
    // Expected: Ordinary 8h @ 1.50 = 342.48, OT 2h @ 2.0 = 114.16
    // ==========================================================================
    #[test]
    fn test_satot_001_fulltime_10h_saturday_2h_overtime() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");
        let overtime_hours = dec("2.0");

        let result = calculate_weekend_overtime(
            overtime_hours,
            base_rate,
            &employee,
            &config,
            DayType::Saturday,
            saturday_date(),
            "shift_001",
            1,
        );

        assert!(result.pay_line.is_some());
        let pay_line = result.pay_line.unwrap();

        assert_eq!(pay_line.category, PayCategory::Overtime200);
        assert_eq!(pay_line.hours, dec("2.0"));
        // 2h × ($28.54 × 2.0) = 2h × $57.08 = $114.16
        assert_eq!(pay_line.rate, dec("57.08"));
        assert_eq!(pay_line.amount, dec("114.16"));
        assert_eq!(pay_line.clause_ref, "25.1(a)(i)(B)");
    }

    // ==========================================================================
    // SATOT-002: casual 10h Saturday - 2h overtime @ 250%
    // Expected: OT 2h @ 2.5 = 142.70
    // ==========================================================================
    #[test]
    fn test_satot_002_casual_10h_saturday_2h_overtime() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::Casual);
        let base_rate = dec("28.54");
        let overtime_hours = dec("2.0");

        let result = calculate_weekend_overtime(
            overtime_hours,
            base_rate,
            &employee,
            &config,
            DayType::Saturday,
            saturday_date(),
            "shift_001",
            1,
        );

        assert!(result.pay_line.is_some());
        let pay_line = result.pay_line.unwrap();

        assert_eq!(pay_line.category, PayCategory::Overtime200);
        assert_eq!(pay_line.hours, dec("2.0"));
        // 2h × ($28.54 × 2.5) = 2h × $71.35 = $142.70
        assert_eq!(pay_line.rate, dec("71.35"));
        assert_eq!(pay_line.amount, dec("142.70"));
    }

    // ==========================================================================
    // SUNOT-001: fulltime 10h Sunday - 2h overtime @ 200%
    // Expected: OT 2h @ 2.0 = 114.16
    // ==========================================================================
    #[test]
    fn test_sunot_001_fulltime_10h_sunday_2h_overtime() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");
        let overtime_hours = dec("2.0");

        let result = calculate_weekend_overtime(
            overtime_hours,
            base_rate,
            &employee,
            &config,
            DayType::Sunday,
            sunday_date(),
            "shift_001",
            1,
        );

        assert!(result.pay_line.is_some());
        let pay_line = result.pay_line.unwrap();

        assert_eq!(pay_line.category, PayCategory::Overtime200);
        assert_eq!(pay_line.hours, dec("2.0"));
        // 2h × ($28.54 × 2.0) = 2h × $57.08 = $114.16
        assert_eq!(pay_line.rate, dec("57.08"));
        assert_eq!(pay_line.amount, dec("114.16"));
        assert_eq!(pay_line.clause_ref, "25.1(a)(i)(B)");
    }

    // ==========================================================================
    // SUNOT-002: casual 10h Sunday - 2h overtime @ 250%
    // Expected: OT 2h @ 2.5 = 142.70
    // ==========================================================================
    #[test]
    fn test_sunot_002_casual_10h_sunday_2h_overtime() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::Casual);
        let base_rate = dec("28.54");
        let overtime_hours = dec("2.0");

        let result = calculate_weekend_overtime(
            overtime_hours,
            base_rate,
            &employee,
            &config,
            DayType::Sunday,
            sunday_date(),
            "shift_001",
            1,
        );

        assert!(result.pay_line.is_some());
        let pay_line = result.pay_line.unwrap();

        assert_eq!(pay_line.category, PayCategory::Overtime200);
        assert_eq!(pay_line.hours, dec("2.0"));
        // 2h × ($28.54 × 2.5) = 2h × $71.35 = $142.70
        assert_eq!(pay_line.rate, dec("71.35"));
        assert_eq!(pay_line.amount, dec("142.70"));
    }

    // ==========================================================================
    // Test: No overtime when hours are zero
    // ==========================================================================
    #[test]
    fn test_no_overtime_when_zero_hours() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");

        let result = calculate_weekend_overtime(
            dec("0.0"),
            base_rate,
            &employee,
            &config,
            DayType::Saturday,
            saturday_date(),
            "shift_001",
            1,
        );

        assert!(result.pay_line.is_none());
        assert!(result.audit_step.is_none());
    }

    // ==========================================================================
    // Test: Audit step has correct information
    // ==========================================================================
    #[test]
    fn test_audit_step_content() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");
        let overtime_hours = dec("2.0");

        let result = calculate_weekend_overtime(
            overtime_hours,
            base_rate,
            &employee,
            &config,
            DayType::Saturday,
            saturday_date(),
            "shift_001",
            5,
        );

        assert!(result.audit_step.is_some());
        let step = result.audit_step.unwrap();

        assert_eq!(step.step_number, 5);
        assert_eq!(step.rule_id, "weekend_overtime");
        assert_eq!(step.rule_name, "Saturday Overtime");
        assert_eq!(step.clause_ref, "25.1(a)(i)(B)");

        // Check input contains expected fields
        assert_eq!(step.input["hours"].as_str().unwrap(), "2");
        assert_eq!(step.input["base_rate"].as_str().unwrap(), "28.54");
        assert_eq!(step.input["employment_type"].as_str().unwrap(), "full_time");
        assert_eq!(step.input["day_type"].as_str().unwrap(), "Saturday");

        // Check output contains expected fields
        assert_eq!(step.output["multiplier"].as_str().unwrap(), "2");
        assert_eq!(step.output["rate"].as_str().unwrap(), "57.08");
        assert_eq!(step.output["amount"].as_str().unwrap(), "114.16");
    }

    // ==========================================================================
    // Test: Audit reasoning for casual mentions loading
    // ==========================================================================
    #[test]
    fn test_audit_reasoning_for_casual_mentions_loading() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::Casual);
        let base_rate = dec("28.54");

        let result = calculate_weekend_overtime(
            dec("2.0"),
            base_rate,
            &employee,
            &config,
            DayType::Saturday,
            saturday_date(),
            "shift_001",
            1,
        );

        assert!(result.audit_step.is_some());
        let step = result.audit_step.unwrap();
        assert!(step.reasoning.contains("casual loading"));
    }

    // ==========================================================================
    // Test: Part-time rates same as full-time
    // ==========================================================================
    #[test]
    fn test_part_time_rates_same_as_full_time() {
        let config = load_config();
        let ft_employee = create_test_employee(EmploymentType::FullTime);
        let pt_employee = create_test_employee(EmploymentType::PartTime);
        let base_rate = dec("28.54");
        let overtime_hours = dec("2.0");

        let ft_result = calculate_weekend_overtime(
            overtime_hours,
            base_rate,
            &ft_employee,
            &config,
            DayType::Saturday,
            saturday_date(),
            "shift_001",
            1,
        );

        let pt_result = calculate_weekend_overtime(
            overtime_hours,
            base_rate,
            &pt_employee,
            &config,
            DayType::Saturday,
            saturday_date(),
            "shift_001",
            1,
        );

        assert_eq!(
            ft_result.pay_line.unwrap().rate,
            pt_result.pay_line.unwrap().rate
        );
    }

    // ==========================================================================
    // Test: Pay line has correct date and shift_id
    // ==========================================================================
    #[test]
    fn test_pay_line_preserves_date_and_shift_id() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");
        let custom_date = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();

        let result = calculate_weekend_overtime(
            dec("2.0"),
            base_rate,
            &employee,
            &config,
            DayType::Saturday,
            custom_date,
            "my_shift_123",
            1,
        );

        assert!(result.pay_line.is_some());
        let pay_line = result.pay_line.unwrap();
        assert_eq!(pay_line.date, custom_date);
        assert_eq!(pay_line.shift_id, "my_shift_123");
    }

    // ==========================================================================
    // Test: Fractional overtime hours
    // ==========================================================================
    #[test]
    fn test_fractional_overtime_hours() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");
        let overtime_hours = dec("1.5");

        let result = calculate_weekend_overtime(
            overtime_hours,
            base_rate,
            &employee,
            &config,
            DayType::Sunday,
            sunday_date(),
            "shift_001",
            1,
        );

        assert!(result.pay_line.is_some());
        let pay_line = result.pay_line.unwrap();

        assert_eq!(pay_line.hours, dec("1.5"));
        // 1.5h × $57.08 = $85.62
        assert_eq!(pay_line.amount, dec("85.62"));
    }

    // ==========================================================================
    // Test: Weekend overtime is NOT tiered (all at same rate)
    // ==========================================================================
    #[test]
    fn test_weekend_overtime_is_not_tiered() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");

        // Even with 4 hours of overtime (which would be tiered on weekdays),
        // weekend overtime should be a single pay line at 200%
        let result = calculate_weekend_overtime(
            dec("4.0"),
            base_rate,
            &employee,
            &config,
            DayType::Saturday,
            saturday_date(),
            "shift_001",
            1,
        );

        assert!(result.pay_line.is_some());
        let pay_line = result.pay_line.unwrap();

        // All 4 hours at 200%
        assert_eq!(pay_line.hours, dec("4.0"));
        assert_eq!(pay_line.rate, dec("57.08")); // 28.54 × 2.0
        assert_eq!(pay_line.amount, dec("228.32")); // 4 × 57.08
        assert_eq!(pay_line.category, PayCategory::Overtime200);
    }

    // ==========================================================================
    // Test: Weekday day type returns empty result
    // ==========================================================================
    #[test]
    fn test_weekday_returns_empty_result() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");

        let result = calculate_weekend_overtime(
            dec("2.0"),
            base_rate,
            &employee,
            &config,
            DayType::Weekday,
            NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            "shift_001",
            1,
        );

        assert!(result.pay_line.is_none());
        assert!(result.audit_step.is_none());
    }
}
