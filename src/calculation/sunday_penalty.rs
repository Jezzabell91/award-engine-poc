//! Sunday penalty rate calculation functionality.
//!
//! This module provides functions for calculating Sunday penalty rates
//! as per clause 23.1 and 23.2(b) of the Aged Care Award 2010.

use rust_decimal::Decimal;

use crate::config::AwardConfig;
use crate::models::{AuditStep, Employee, EmploymentType, PayCategory, PayLine};

use super::ShiftSegment;

/// The result of a Sunday penalty calculation, including the pay line and audit step.
#[derive(Debug, Clone)]
pub struct SundayPayResult {
    /// The pay line for the Sunday penalty.
    pub pay_line: PayLine,
    /// The audit step recording this calculation.
    pub audit_step: AuditStep,
}

/// Calculates Sunday penalty pay for a shift segment.
///
/// Applies the appropriate Sunday penalty rate based on employment type:
/// - Full-time: 175% of base rate (clause 23.1)
/// - Part-time: 175% of base rate (clause 23.1)
/// - Casual: 200% of base rate (clause 23.2(b)) - NOT ordinary rate + casual loading + penalty
///
/// # Arguments
///
/// * `segment` - The shift segment to calculate pay for (must be on a Sunday)
/// * `employee` - The employee working the shift
/// * `base_rate` - The base hourly rate from the award
/// * `config` - The award configuration containing penalty rates
/// * `step_number` - The step number for audit trail sequencing
///
/// # Returns
///
/// Returns a `SundayPayResult` containing the pay line and audit step.
///
/// # Award Reference
///
/// - Clause 23.1: Weekend penalty rates for full-time and part-time employees
/// - Clause 23.2(b): Sunday penalty rate for casual employees (200%)
///
/// # Examples
///
/// ```no_run
/// use award_engine::calculation::{calculate_sunday_pay, ShiftSegment, DayType};
/// use award_engine::config::ConfigLoader;
/// use award_engine::models::{Employee, EmploymentType};
/// use chrono::{NaiveDate, NaiveDateTime};
/// use rust_decimal::Decimal;
/// use std::str::FromStr;
///
/// let loader = ConfigLoader::load("config/ma000018").unwrap();
/// let config = loader.config();
/// let employee = Employee {
///     id: "emp_001".to_string(),
///     employment_type: EmploymentType::FullTime,
///     classification_code: "dce_level_3".to_string(),
///     date_of_birth: NaiveDate::from_ymd_opt(1990, 1, 15).unwrap(),
///     employment_start_date: NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
///     base_hourly_rate: None,
///     tags: vec![],
/// };
///
/// let segment = ShiftSegment {
///     start_time: NaiveDateTime::parse_from_str("2026-01-18 09:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
///     end_time: NaiveDateTime::parse_from_str("2026-01-18 17:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
///     day_type: DayType::Sunday,
///     hours: Decimal::from_str("8.0").unwrap(),
/// };
///
/// let result = calculate_sunday_pay(&segment, &employee, Decimal::from_str("28.54").unwrap(), config, 1);
/// // 8.0 hours * $28.54 * 1.75 = $399.56
/// assert_eq!(result.pay_line.amount, Decimal::from_str("399.56").unwrap());
/// assert_eq!(result.pay_line.category, award_engine::models::PayCategory::Sunday);
/// ```
pub fn calculate_sunday_pay(
    segment: &ShiftSegment,
    employee: &Employee,
    base_rate: Decimal,
    config: &AwardConfig,
    step_number: u32,
) -> SundayPayResult {
    let penalties = config.penalties();
    let sunday_penalties = &penalties.penalties.sunday;

    let (multiplier, category, clause_ref) = match employee.employment_type {
        EmploymentType::FullTime => (
            sunday_penalties.full_time,
            PayCategory::Sunday,
            "23.1".to_string(),
        ),
        EmploymentType::PartTime => (
            sunday_penalties.part_time,
            PayCategory::Sunday,
            "23.1".to_string(),
        ),
        EmploymentType::Casual => (
            sunday_penalties.casual,
            PayCategory::SundayCasual,
            "23.2(b)".to_string(),
        ),
    };

    let effective_rate = base_rate * multiplier;
    let amount = segment.hours * effective_rate;

    let employment_type_str = match employee.employment_type {
        EmploymentType::FullTime => "full_time",
        EmploymentType::PartTime => "part_time",
        EmploymentType::Casual => "casual",
    };

    let pay_line = PayLine {
        date: segment.start_time.date(),
        shift_id: String::new(), // Will be set by caller
        category,
        hours: segment.hours,
        rate: effective_rate,
        amount,
        clause_ref: clause_ref.clone(),
    };

    let audit_step = AuditStep {
        step_number,
        rule_id: "sunday_penalty".to_string(),
        rule_name: "Sunday Penalty Rate".to_string(),
        clause_ref,
        input: serde_json::json!({
            "hours": segment.hours.normalize().to_string(),
            "base_rate": base_rate.normalize().to_string(),
            "employment_type": employment_type_str,
            "day_type": "Sunday"
        }),
        output: serde_json::json!({
            "multiplier": multiplier.normalize().to_string(),
            "effective_rate": effective_rate.normalize().to_string(),
            "amount": amount.normalize().to_string(),
            "category": format!("{:?}", category)
        }),
        reasoning: format!(
            "Sunday penalty: {} hours × ${} × {} = ${}",
            segment.hours.normalize(),
            base_rate.normalize(),
            multiplier.normalize(),
            amount.normalize()
        ),
    };

    SundayPayResult {
        pay_line,
        audit_step,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calculation::DayType;
    use crate::config::ConfigLoader;
    use chrono::{NaiveDate, NaiveDateTime};
    use std::str::FromStr;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn make_datetime(date_str: &str, time_str: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(&format!("{} {}", date_str, time_str), "%Y-%m-%d %H:%M:%S")
            .unwrap()
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

    fn create_sunday_segment(hours: Decimal) -> ShiftSegment {
        // 2026-01-18 is a Sunday
        ShiftSegment {
            start_time: make_datetime("2026-01-18", "09:00:00"),
            end_time: make_datetime("2026-01-18", "17:00:00"),
            day_type: DayType::Sunday,
            hours,
        }
    }

    fn load_config() -> AwardConfig {
        ConfigLoader::load("config/ma000018")
            .expect("Failed to load config")
            .config()
            .clone()
    }

    // ==========================================================================
    // SUN-001: fulltime 8h Sunday
    // ==========================================================================
    #[test]
    fn test_sun_001_fulltime_8h_sunday() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let segment = create_sunday_segment(dec("8.0"));

        let result = calculate_sunday_pay(&segment, &employee, dec("28.54"), &config, 1);

        // 8.0 * 28.54 * 1.75 = 399.56
        assert_eq!(result.pay_line.amount, dec("399.56"));
        assert_eq!(result.pay_line.category, PayCategory::Sunday);
        assert_eq!(result.pay_line.clause_ref, "23.1");
        assert_eq!(result.pay_line.hours, dec("8.0"));
        assert_eq!(result.pay_line.rate, dec("49.945")); // 28.54 * 1.75
    }

    // ==========================================================================
    // SUN-002: parttime 8h Sunday
    // ==========================================================================
    #[test]
    fn test_sun_002_parttime_8h_sunday() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::PartTime);
        let segment = create_sunday_segment(dec("8.0"));

        let result = calculate_sunday_pay(&segment, &employee, dec("28.54"), &config, 1);

        // 8.0 * 28.54 * 1.75 = 399.56
        assert_eq!(result.pay_line.amount, dec("399.56"));
        assert_eq!(result.pay_line.category, PayCategory::Sunday);
        assert_eq!(result.pay_line.clause_ref, "23.1");
    }

    // ==========================================================================
    // SUN-003: casual 8h Sunday
    // ==========================================================================
    #[test]
    fn test_sun_003_casual_8h_sunday() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::Casual);
        let segment = create_sunday_segment(dec("8.0"));

        let result = calculate_sunday_pay(&segment, &employee, dec("28.54"), &config, 1);

        // 8.0 * 28.54 * 2.00 = 456.64
        // Note: Casual rate is 200% of base rate, NOT base + casual loading + penalty
        assert_eq!(result.pay_line.amount, dec("456.64"));
        assert_eq!(result.pay_line.category, PayCategory::SundayCasual);
        assert_eq!(result.pay_line.clause_ref, "23.2(b)");
        assert_eq!(result.pay_line.rate, dec("57.08")); // 28.54 * 2.00
    }

    // ==========================================================================
    // SUN-004: fulltime 4h Sunday
    // ==========================================================================
    #[test]
    fn test_sun_004_fulltime_4h_sunday() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let segment = ShiftSegment {
            start_time: make_datetime("2026-01-18", "09:00:00"),
            end_time: make_datetime("2026-01-18", "13:00:00"),
            day_type: DayType::Sunday,
            hours: dec("4.0"),
        };

        let result = calculate_sunday_pay(&segment, &employee, dec("28.54"), &config, 1);

        // 4.0 * 28.54 * 1.75 = 199.78
        assert_eq!(result.pay_line.amount, dec("199.78"));
        assert_eq!(result.pay_line.category, PayCategory::Sunday);
        assert_eq!(result.pay_line.hours, dec("4.0"));
    }

    // ==========================================================================
    // SUN-005: casual 6.5h Sunday
    // ==========================================================================
    #[test]
    fn test_sun_005_casual_6_5h_sunday() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::Casual);
        let segment = ShiftSegment {
            start_time: make_datetime("2026-01-18", "09:00:00"),
            end_time: make_datetime("2026-01-18", "15:30:00"),
            day_type: DayType::Sunday,
            hours: dec("6.5"),
        };

        let result = calculate_sunday_pay(&segment, &employee, dec("28.54"), &config, 1);

        // 6.5 * 28.54 * 2.00 = 371.02
        assert_eq!(result.pay_line.amount, dec("371.02"));
        assert_eq!(result.pay_line.category, PayCategory::SundayCasual);
        assert_eq!(result.pay_line.clause_ref, "23.2(b)");
    }

    // ==========================================================================
    // Additional tests
    // ==========================================================================
    #[test]
    fn test_audit_step_has_correct_information() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let segment = create_sunday_segment(dec("8.0"));

        let result = calculate_sunday_pay(&segment, &employee, dec("28.54"), &config, 5);

        assert_eq!(result.audit_step.step_number, 5);
        assert_eq!(result.audit_step.rule_id, "sunday_penalty");
        assert_eq!(result.audit_step.rule_name, "Sunday Penalty Rate");
        assert_eq!(result.audit_step.clause_ref, "23.1");

        // Check input contains expected fields
        assert_eq!(result.audit_step.input["hours"].as_str().unwrap(), "8");
        assert_eq!(
            result.audit_step.input["base_rate"].as_str().unwrap(),
            "28.54"
        );
        assert_eq!(
            result.audit_step.input["employment_type"].as_str().unwrap(),
            "full_time"
        );
        assert_eq!(
            result.audit_step.input["day_type"].as_str().unwrap(),
            "Sunday"
        );

        // Check output contains expected fields
        assert_eq!(
            result.audit_step.output["multiplier"].as_str().unwrap(),
            "1.75"
        );
        assert_eq!(
            result.audit_step.output["effective_rate"].as_str().unwrap(),
            "49.945"
        );
        assert_eq!(
            result.audit_step.output["amount"].as_str().unwrap(),
            "399.56"
        );
    }

    #[test]
    fn test_audit_reasoning_explains_calculation() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let segment = create_sunday_segment(dec("8.0"));

        let result = calculate_sunday_pay(&segment, &employee, dec("28.54"), &config, 1);

        assert!(result.audit_step.reasoning.contains("Sunday penalty"));
        assert!(result.audit_step.reasoning.contains("8"));
        assert!(result.audit_step.reasoning.contains("28.54"));
        assert!(result.audit_step.reasoning.contains("1.75"));
        assert!(result.audit_step.reasoning.contains("399.56"));
    }

    #[test]
    fn test_pay_line_has_correct_date() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let segment = create_sunday_segment(dec("8.0"));

        let result = calculate_sunday_pay(&segment, &employee, dec("28.54"), &config, 1);

        // 2026-01-18 is a Sunday
        assert_eq!(
            result.pay_line.date,
            NaiveDate::from_ymd_opt(2026, 1, 18).unwrap()
        );
    }

    #[test]
    fn test_casual_rate_is_not_cumulative_with_loading() {
        // Verify that casual rate is 200% of base, not (base * 1.25) * 1.75
        let config = load_config();
        let employee = create_test_employee(EmploymentType::Casual);
        let segment = create_sunday_segment(dec("8.0"));

        let result = calculate_sunday_pay(&segment, &employee, dec("28.54"), &config, 1);

        // If it were cumulative: 28.54 * 1.25 * 1.75 = 62.43125 rate, 62.43125 * 8 = 499.45
        // But it should be: 28.54 * 2.00 = 57.08 rate, 57.08 * 8 = 456.64
        assert_eq!(result.pay_line.amount, dec("456.64"));
        assert_ne!(result.pay_line.amount, dec("499.45"));
    }
}
