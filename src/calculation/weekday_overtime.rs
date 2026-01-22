//! Weekday overtime rate calculation functionality.
//!
//! This module provides functions for calculating overtime pay at tiered rates
//! for weekday shifts as per the Aged Care Award 2010 clause 25.1.
//!
//! ## Rate Structure
//!
//! **Weekday overtime is calculated in two tiers:**
//! - First 2 hours: 150% for non-casuals, 187.5% for casuals (1.5 × 1.25)
//! - After 2 hours: 200% for non-casuals, 250% for casuals (2.0 × 1.25)

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::config::AwardConfig;
use crate::models::{AuditStep, Employee, EmploymentType, PayCategory, PayLine};

/// The threshold in hours for tier 1 weekday overtime.
/// First 2 hours are paid at a lower rate (150%/187.5%).
pub const WEEKDAY_OT_TIER_1_THRESHOLD: Decimal = Decimal::from_parts(2, 0, 0, false, 0);

/// The result of weekday overtime calculation.
///
/// Contains the pay lines for each tier of overtime and the audit steps
/// documenting the calculations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeekdayOvertimeResult {
    /// Pay lines for overtime (may be 0, 1, or 2 lines depending on hours).
    pub pay_lines: Vec<PayLine>,
    /// Audit steps recording each tier calculation.
    pub audit_steps: Vec<AuditStep>,
}

/// Calculates weekday overtime pay at tiered rates.
///
/// Weekday overtime is calculated in two tiers as per clause 25.1(a)(i)(A):
/// - **Tier 1 (first 2 hours):** 150% for non-casuals, 187.5% for casuals
/// - **Tier 2 (after 2 hours):** 200% for non-casuals, 250% for casuals
///
/// # Arguments
///
/// * `overtime_hours` - The total overtime hours to be paid
/// * `base_rate` - The base hourly rate (before any loading)
/// * `employee` - The employee receiving overtime pay
/// * `config` - The award configuration containing overtime multipliers
/// * `date` - The date of the shift for pay line records
/// * `shift_id` - The shift ID for pay line records
/// * `step_number_start` - The starting step number for audit trail sequencing
///
/// # Returns
///
/// A [`WeekdayOvertimeResult`] containing:
/// - `pay_lines`: 0-2 pay lines depending on overtime hours
/// - `audit_steps`: Documentation of each tier calculation
///
/// # Award Reference
///
/// - Clause 25.1(a)(i)(A): Weekday overtime rates
///
/// # Examples
///
/// ## 1 hour overtime (non-casual)
///
/// ```
/// use award_engine::calculation::calculate_weekday_overtime;
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
/// let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
///
/// let result = calculate_weekday_overtime(
///     Decimal::from_str("1.0").unwrap(),
///     Decimal::from_str("28.54").unwrap(),
///     &employee,
///     &config,
///     date,
///     "shift_001",
///     1,
/// );
///
/// assert_eq!(result.pay_lines.len(), 1);
/// assert_eq!(result.pay_lines[0].category, PayCategory::Overtime150);
/// ```
///
/// ## 3 hours overtime (non-casual, triggers both tiers)
///
/// ```
/// use award_engine::calculation::calculate_weekday_overtime;
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
/// let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
///
/// let result = calculate_weekday_overtime(
///     Decimal::from_str("3.0").unwrap(),
///     Decimal::from_str("28.54").unwrap(),
///     &employee,
///     &config,
///     date,
///     "shift_001",
///     1,
/// );
///
/// assert_eq!(result.pay_lines.len(), 2);
/// assert_eq!(result.pay_lines[0].category, PayCategory::Overtime150);
/// assert_eq!(result.pay_lines[1].category, PayCategory::Overtime200);
/// ```
pub fn calculate_weekday_overtime(
    overtime_hours: Decimal,
    base_rate: Decimal,
    employee: &Employee,
    config: &AwardConfig,
    date: NaiveDate,
    shift_id: &str,
    step_number_start: u32,
) -> WeekdayOvertimeResult {
    let mut pay_lines = Vec::new();
    let mut audit_steps = Vec::new();
    let mut step_number = step_number_start;

    // If no overtime, return empty result
    if overtime_hours <= Decimal::ZERO {
        return WeekdayOvertimeResult {
            pay_lines,
            audit_steps,
        };
    }

    // Get overtime rates from config
    let overtime_config = &config.penalties().overtime.weekday;

    // Get the multipliers based on employment type
    let (tier1_multiplier, tier2_multiplier) = match employee.employment_type {
        EmploymentType::FullTime => (
            overtime_config.first_two_hours.full_time,
            overtime_config.after_two_hours.full_time,
        ),
        EmploymentType::PartTime => (
            overtime_config.first_two_hours.part_time,
            overtime_config.after_two_hours.part_time,
        ),
        EmploymentType::Casual => (
            overtime_config.first_two_hours.casual,
            overtime_config.after_two_hours.casual,
        ),
    };

    let employment_type_str = match employee.employment_type {
        EmploymentType::FullTime => "full_time",
        EmploymentType::PartTime => "part_time",
        EmploymentType::Casual => "casual",
    };

    // Calculate tier 1 overtime (first 2 hours)
    let tier1_hours = if overtime_hours <= WEEKDAY_OT_TIER_1_THRESHOLD {
        overtime_hours
    } else {
        WEEKDAY_OT_TIER_1_THRESHOLD
    };

    if tier1_hours > Decimal::ZERO {
        let tier1_rate = base_rate * tier1_multiplier;
        let tier1_amount = tier1_hours * tier1_rate;

        let tier1_reasoning = if employee.is_casual() {
            format!(
                "First {} hours of weekday overtime at {}% ({}% × 1.25 casual loading): {} hours × ${} = ${}",
                tier1_hours.normalize(),
                (tier1_multiplier * Decimal::from(100)).normalize(),
                Decimal::from(150),
                tier1_hours.normalize(),
                tier1_rate.normalize(),
                tier1_amount.normalize()
            )
        } else {
            format!(
                "First {} hours of weekday overtime at {}%: {} hours × ${} = ${}",
                tier1_hours.normalize(),
                (tier1_multiplier * Decimal::from(100)).normalize(),
                tier1_hours.normalize(),
                tier1_rate.normalize(),
                tier1_amount.normalize()
            )
        };

        let tier1_audit = AuditStep {
            step_number,
            rule_id: "overtime_tier_1".to_string(),
            rule_name: "Weekday Overtime Tier 1".to_string(),
            clause_ref: "25.1(a)(i)(A)".to_string(),
            input: serde_json::json!({
                "hours": tier1_hours.normalize().to_string(),
                "base_rate": base_rate.normalize().to_string(),
                "employment_type": employment_type_str
            }),
            output: serde_json::json!({
                "multiplier": tier1_multiplier.normalize().to_string(),
                "rate": tier1_rate.normalize().to_string(),
                "amount": tier1_amount.normalize().to_string()
            }),
            reasoning: tier1_reasoning,
        };

        let tier1_pay_line = PayLine {
            date,
            shift_id: shift_id.to_string(),
            category: PayCategory::Overtime150,
            hours: tier1_hours,
            rate: tier1_rate,
            amount: tier1_amount,
            clause_ref: "25.1(a)(i)(A)".to_string(),
        };

        pay_lines.push(tier1_pay_line);
        audit_steps.push(tier1_audit);
        step_number += 1;
    }

    // Calculate tier 2 overtime (after 2 hours)
    let tier2_hours = if overtime_hours > WEEKDAY_OT_TIER_1_THRESHOLD {
        overtime_hours - WEEKDAY_OT_TIER_1_THRESHOLD
    } else {
        Decimal::ZERO
    };

    if tier2_hours > Decimal::ZERO {
        let tier2_rate = base_rate * tier2_multiplier;
        let tier2_amount = tier2_hours * tier2_rate;

        let tier2_reasoning = if employee.is_casual() {
            format!(
                "Overtime after first 2 hours at {}% ({}% × 1.25 casual loading): {} hours × ${} = ${}",
                (tier2_multiplier * Decimal::from(100)).normalize(),
                Decimal::from(200),
                tier2_hours.normalize(),
                tier2_rate.normalize(),
                tier2_amount.normalize()
            )
        } else {
            format!(
                "Overtime after first 2 hours at {}%: {} hours × ${} = ${}",
                (tier2_multiplier * Decimal::from(100)).normalize(),
                tier2_hours.normalize(),
                tier2_rate.normalize(),
                tier2_amount.normalize()
            )
        };

        let tier2_audit = AuditStep {
            step_number,
            rule_id: "overtime_tier_2".to_string(),
            rule_name: "Weekday Overtime Tier 2".to_string(),
            clause_ref: "25.1(a)(i)(A)".to_string(),
            input: serde_json::json!({
                "hours": tier2_hours.normalize().to_string(),
                "base_rate": base_rate.normalize().to_string(),
                "employment_type": employment_type_str
            }),
            output: serde_json::json!({
                "multiplier": tier2_multiplier.normalize().to_string(),
                "rate": tier2_rate.normalize().to_string(),
                "amount": tier2_amount.normalize().to_string()
            }),
            reasoning: tier2_reasoning,
        };

        let tier2_pay_line = PayLine {
            date,
            shift_id: shift_id.to_string(),
            category: PayCategory::Overtime200,
            hours: tier2_hours,
            rate: tier2_rate,
            amount: tier2_amount,
            clause_ref: "25.1(a)(i)(A)".to_string(),
        };

        pay_lines.push(tier2_pay_line);
        audit_steps.push(tier2_audit);
    }

    WeekdayOvertimeResult {
        pay_lines,
        audit_steps,
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

    fn test_date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 1, 15).unwrap() // Wednesday
    }

    fn load_config() -> AwardConfig {
        ConfigLoader::load("config/ma000018")
            .unwrap()
            .config()
            .clone()
    }

    // ==========================================================================
    // WOT-001: fulltime 8h weekday - no overtime
    // ==========================================================================
    #[test]
    fn test_wot_001_fulltime_8h_weekday_no_overtime() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");

        // With 8 hours worked, there's no overtime
        let result = calculate_weekday_overtime(
            dec("0.0"),
            base_rate,
            &employee,
            &config,
            test_date(),
            "shift_001",
            1,
        );

        assert!(result.pay_lines.is_empty());
        assert!(result.audit_steps.is_empty());
    }

    // ==========================================================================
    // WOT-002: fulltime 9h weekday - 1h overtime @ 150%
    // ==========================================================================
    #[test]
    fn test_wot_002_fulltime_9h_weekday_1h_overtime() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");
        let overtime_hours = dec("1.0");

        let result = calculate_weekday_overtime(
            overtime_hours,
            base_rate,
            &employee,
            &config,
            test_date(),
            "shift_001",
            1,
        );

        assert_eq!(result.pay_lines.len(), 1);

        let ot150 = &result.pay_lines[0];
        assert_eq!(ot150.category, PayCategory::Overtime150);
        assert_eq!(ot150.hours, dec("1.0"));
        // 1h × ($28.54 × 1.5) = 1h × $42.81 = $42.81
        assert_eq!(ot150.rate, dec("42.81"));
        assert_eq!(ot150.amount, dec("42.81"));

        // Check audit step
        assert_eq!(result.audit_steps.len(), 1);
        assert_eq!(result.audit_steps[0].rule_id, "overtime_tier_1");
        assert_eq!(result.audit_steps[0].clause_ref, "25.1(a)(i)(A)");
    }

    // ==========================================================================
    // WOT-003: fulltime 10h weekday - 2h overtime @ 150%
    // ==========================================================================
    #[test]
    fn test_wot_003_fulltime_10h_weekday_2h_overtime() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");
        let overtime_hours = dec("2.0");

        let result = calculate_weekday_overtime(
            overtime_hours,
            base_rate,
            &employee,
            &config,
            test_date(),
            "shift_001",
            1,
        );

        assert_eq!(result.pay_lines.len(), 1);

        let ot150 = &result.pay_lines[0];
        assert_eq!(ot150.category, PayCategory::Overtime150);
        assert_eq!(ot150.hours, dec("2.0"));
        // 2h × ($28.54 × 1.5) = 2h × $42.81 = $85.62
        assert_eq!(ot150.rate, dec("42.81"));
        assert_eq!(ot150.amount, dec("85.62"));
    }

    // ==========================================================================
    // WOT-004: fulltime 11h weekday - 3h overtime (2h@150%, 1h@200%)
    // ==========================================================================
    #[test]
    fn test_wot_004_fulltime_11h_weekday_3h_overtime() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");
        let overtime_hours = dec("3.0");

        let result = calculate_weekday_overtime(
            overtime_hours,
            base_rate,
            &employee,
            &config,
            test_date(),
            "shift_001",
            1,
        );

        assert_eq!(result.pay_lines.len(), 2);

        // Tier 1: 2h @ 150%
        let ot150 = &result.pay_lines[0];
        assert_eq!(ot150.category, PayCategory::Overtime150);
        assert_eq!(ot150.hours, dec("2.0"));
        assert_eq!(ot150.rate, dec("42.81"));
        assert_eq!(ot150.amount, dec("85.62"));

        // Tier 2: 1h @ 200%
        let ot200 = &result.pay_lines[1];
        assert_eq!(ot200.category, PayCategory::Overtime200);
        assert_eq!(ot200.hours, dec("1.0"));
        // 1h × ($28.54 × 2.0) = 1h × $57.08 = $57.08
        assert_eq!(ot200.rate, dec("57.08"));
        assert_eq!(ot200.amount, dec("57.08"));

        // Check audit steps
        assert_eq!(result.audit_steps.len(), 2);
        assert_eq!(result.audit_steps[0].rule_id, "overtime_tier_1");
        assert_eq!(result.audit_steps[1].rule_id, "overtime_tier_2");
    }

    // ==========================================================================
    // WOT-005: fulltime 12h weekday - 4h overtime (2h@150%, 2h@200%)
    // ==========================================================================
    #[test]
    fn test_wot_005_fulltime_12h_weekday_4h_overtime() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");
        let overtime_hours = dec("4.0");

        let result = calculate_weekday_overtime(
            overtime_hours,
            base_rate,
            &employee,
            &config,
            test_date(),
            "shift_001",
            1,
        );

        assert_eq!(result.pay_lines.len(), 2);

        // Tier 1: 2h @ 150%
        let ot150 = &result.pay_lines[0];
        assert_eq!(ot150.category, PayCategory::Overtime150);
        assert_eq!(ot150.hours, dec("2.0"));
        assert_eq!(ot150.rate, dec("42.81"));
        assert_eq!(ot150.amount, dec("85.62"));

        // Tier 2: 2h @ 200%
        let ot200 = &result.pay_lines[1];
        assert_eq!(ot200.category, PayCategory::Overtime200);
        assert_eq!(ot200.hours, dec("2.0"));
        assert_eq!(ot200.rate, dec("57.08"));
        assert_eq!(ot200.amount, dec("114.16"));

        // Total OT: $85.62 + $114.16 = $199.78
    }

    // ==========================================================================
    // WCOT-001: casual 8h weekday - no overtime
    // ==========================================================================
    #[test]
    fn test_wcot_001_casual_8h_weekday_no_overtime() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::Casual);
        let base_rate = dec("28.54");

        let result = calculate_weekday_overtime(
            dec("0.0"),
            base_rate,
            &employee,
            &config,
            test_date(),
            "shift_001",
            1,
        );

        assert!(result.pay_lines.is_empty());
        assert!(result.audit_steps.is_empty());
    }

    // ==========================================================================
    // WCOT-002: casual 10h weekday - 2h overtime @ 187.5%
    // ==========================================================================
    #[test]
    fn test_wcot_002_casual_10h_weekday_2h_overtime() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::Casual);
        let base_rate = dec("28.54");
        let overtime_hours = dec("2.0");

        let result = calculate_weekday_overtime(
            overtime_hours,
            base_rate,
            &employee,
            &config,
            test_date(),
            "shift_001",
            1,
        );

        assert_eq!(result.pay_lines.len(), 1);

        let ot_tier1 = &result.pay_lines[0];
        assert_eq!(ot_tier1.category, PayCategory::Overtime150);
        assert_eq!(ot_tier1.hours, dec("2.0"));
        // 2h × ($28.54 × 1.875) = 2h × $53.5125 = $107.025
        // However, with Decimal precision: 28.54 × 1.875 = 53.5125
        // 2 × 53.5125 = 107.025 (rounds to 107.03 in display)
        assert_eq!(ot_tier1.rate, dec("53.5125"));
        assert_eq!(ot_tier1.amount, dec("107.025"));
    }

    // ==========================================================================
    // WCOT-003: casual 12h weekday - 4h overtime (2h@187.5%, 2h@250%)
    // ==========================================================================
    #[test]
    fn test_wcot_003_casual_12h_weekday_4h_overtime() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::Casual);
        let base_rate = dec("28.54");
        let overtime_hours = dec("4.0");

        let result = calculate_weekday_overtime(
            overtime_hours,
            base_rate,
            &employee,
            &config,
            test_date(),
            "shift_001",
            1,
        );

        assert_eq!(result.pay_lines.len(), 2);

        // Tier 1: 2h @ 187.5%
        let ot_tier1 = &result.pay_lines[0];
        assert_eq!(ot_tier1.category, PayCategory::Overtime150);
        assert_eq!(ot_tier1.hours, dec("2.0"));
        // 28.54 × 1.875 = 53.5125
        assert_eq!(ot_tier1.rate, dec("53.5125"));
        // 2 × 53.5125 = 107.025
        assert_eq!(ot_tier1.amount, dec("107.025"));

        // Tier 2: 2h @ 250%
        let ot_tier2 = &result.pay_lines[1];
        assert_eq!(ot_tier2.category, PayCategory::Overtime200);
        assert_eq!(ot_tier2.hours, dec("2.0"));
        // 28.54 × 2.5 = 71.35
        assert_eq!(ot_tier2.rate, dec("71.35"));
        // 2 × 71.35 = 142.70
        assert_eq!(ot_tier2.amount, dec("142.70"));
    }

    // ==========================================================================
    // Additional tests for audit trail completeness
    // ==========================================================================

    #[test]
    fn test_audit_step_numbers_sequential() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");

        let result = calculate_weekday_overtime(
            dec("4.0"),
            base_rate,
            &employee,
            &config,
            test_date(),
            "shift_001",
            5,
        );

        assert_eq!(result.audit_steps[0].step_number, 5);
        assert_eq!(result.audit_steps[1].step_number, 6);
    }

    #[test]
    fn test_audit_reasoning_for_fulltime() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");

        let result = calculate_weekday_overtime(
            dec("3.0"),
            base_rate,
            &employee,
            &config,
            test_date(),
            "shift_001",
            1,
        );

        // Should contain rate information
        assert!(result.audit_steps[0].reasoning.contains("150%"));
        assert!(result.audit_steps[1].reasoning.contains("200%"));
    }

    #[test]
    fn test_audit_reasoning_for_casual_mentions_loading() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::Casual);
        let base_rate = dec("28.54");

        let result = calculate_weekday_overtime(
            dec("3.0"),
            base_rate,
            &employee,
            &config,
            test_date(),
            "shift_001",
            1,
        );

        // Should mention casual loading
        assert!(result.audit_steps[0].reasoning.contains("casual loading"));
        assert!(result.audit_steps[1].reasoning.contains("casual loading"));
    }

    #[test]
    fn test_audit_input_contains_required_fields() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");

        let result = calculate_weekday_overtime(
            dec("1.0"),
            base_rate,
            &employee,
            &config,
            test_date(),
            "shift_001",
            1,
        );

        let step = &result.audit_steps[0];
        assert!(step.input.get("hours").is_some());
        assert!(step.input.get("base_rate").is_some());
        assert!(step.input.get("employment_type").is_some());
    }

    #[test]
    fn test_audit_output_contains_required_fields() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");

        let result = calculate_weekday_overtime(
            dec("1.0"),
            base_rate,
            &employee,
            &config,
            test_date(),
            "shift_001",
            1,
        );

        let step = &result.audit_steps[0];
        assert!(step.output.get("multiplier").is_some());
        assert!(step.output.get("rate").is_some());
        assert!(step.output.get("amount").is_some());
    }

    #[test]
    fn test_pay_line_shift_id_preserved() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");

        let result = calculate_weekday_overtime(
            dec("1.0"),
            base_rate,
            &employee,
            &config,
            test_date(),
            "my_custom_shift_123",
            1,
        );

        assert_eq!(result.pay_lines[0].shift_id, "my_custom_shift_123");
    }

    #[test]
    fn test_pay_line_date_preserved() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();

        let result = calculate_weekday_overtime(
            dec("1.0"),
            base_rate,
            &employee,
            &config,
            date,
            "shift_001",
            1,
        );

        assert_eq!(result.pay_lines[0].date, date);
    }

    #[test]
    fn test_pay_line_clause_ref_correct() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");

        let result = calculate_weekday_overtime(
            dec("3.0"),
            base_rate,
            &employee,
            &config,
            test_date(),
            "shift_001",
            1,
        );

        assert_eq!(result.pay_lines[0].clause_ref, "25.1(a)(i)(A)");
        assert_eq!(result.pay_lines[1].clause_ref, "25.1(a)(i)(A)");
    }

    #[test]
    fn test_fractional_overtime_hours() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);
        let base_rate = dec("28.54");
        let overtime_hours = dec("2.5");

        let result = calculate_weekday_overtime(
            overtime_hours,
            base_rate,
            &employee,
            &config,
            test_date(),
            "shift_001",
            1,
        );

        assert_eq!(result.pay_lines.len(), 2);

        // Tier 1: 2h @ 150%
        assert_eq!(result.pay_lines[0].hours, dec("2.0"));

        // Tier 2: 0.5h @ 200%
        assert_eq!(result.pay_lines[1].hours, dec("0.5"));
        // 0.5h × $57.08 = $28.54
        assert_eq!(result.pay_lines[1].amount, dec("28.54"));
    }

    #[test]
    fn test_part_time_rates_same_as_full_time() {
        let config = load_config();
        let ft_employee = create_test_employee(EmploymentType::FullTime);
        let pt_employee = create_test_employee(EmploymentType::PartTime);
        let base_rate = dec("28.54");
        let overtime_hours = dec("3.0");

        let ft_result = calculate_weekday_overtime(
            overtime_hours,
            base_rate,
            &ft_employee,
            &config,
            test_date(),
            "shift_001",
            1,
        );

        let pt_result = calculate_weekday_overtime(
            overtime_hours,
            base_rate,
            &pt_employee,
            &config,
            test_date(),
            "shift_001",
            1,
        );

        // Part-time and full-time should have the same overtime rates
        assert_eq!(ft_result.pay_lines[0].rate, pt_result.pay_lines[0].rate);
        assert_eq!(ft_result.pay_lines[1].rate, pt_result.pay_lines[1].rate);
    }
}
