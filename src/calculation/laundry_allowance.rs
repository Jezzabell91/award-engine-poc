//! Laundry allowance calculation functionality.
//!
//! This module provides functions for calculating laundry allowance
//! for employees as per clause 15.2(b) of the Aged Care Award 2010.

use rust_decimal::Decimal;

use crate::models::{AllowancePayment, AuditStep, Employee};

/// The tag that enables laundry allowance for an employee.
pub const LAUNDRY_ALLOWANCE_TAG: &str = "laundry_allowance";

/// The clause reference for laundry allowance.
pub const LAUNDRY_ALLOWANCE_CLAUSE: &str = "15.2(b)";

/// The result of calculating laundry allowance, including the payment and audit step.
#[derive(Debug, Clone)]
pub struct LaundryAllowanceResult {
    /// The allowance payment, if the employee is eligible.
    pub allowance: Option<AllowancePayment>,
    /// The audit step recording this calculation.
    pub audit_step: AuditStep,
}

/// Calculates laundry allowance for an employee based on the number of shifts worked.
///
/// The laundry allowance is paid per shift to employees who have the `laundry_allowance`
/// tag, up to a weekly maximum cap.
///
/// # Arguments
///
/// * `employee` - The employee to calculate allowance for
/// * `num_shifts` - The number of shifts worked in the pay period
/// * `per_shift_rate` - The allowance amount per shift (e.g., $0.32)
/// * `weekly_cap` - The maximum allowance per week (e.g., $1.49)
/// * `step_number` - The step number for audit trail sequencing
///
/// # Returns
///
/// Returns a `LaundryAllowanceResult` containing:
/// - `Some(AllowancePayment)` if the employee has the laundry_allowance tag
/// - `None` if the employee does not have the tag
///
/// # Award Reference
///
/// Clause 15.2(b) of the Aged Care Award 2010 specifies the laundry allowance.
///
/// # Examples
///
/// ```
/// use award_engine::calculation::calculate_laundry_allowance;
/// use award_engine::models::{Employee, EmploymentType};
/// use chrono::NaiveDate;
/// use rust_decimal::Decimal;
/// use std::str::FromStr;
///
/// let employee = Employee {
///     id: "emp_001".to_string(),
///     employment_type: EmploymentType::FullTime,
///     classification_code: "dce_level_3".to_string(),
///     date_of_birth: NaiveDate::from_ymd_opt(1990, 1, 15).unwrap(),
///     employment_start_date: NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
///     base_hourly_rate: None,
///     tags: vec!["laundry_allowance".to_string()],
/// };
///
/// let result = calculate_laundry_allowance(
///     &employee,
///     3,
///     Decimal::from_str("0.32").unwrap(),
///     Decimal::from_str("1.49").unwrap(),
///     1,
/// );
///
/// assert!(result.allowance.is_some());
/// let allowance = result.allowance.unwrap();
/// assert_eq!(allowance.amount, Decimal::from_str("0.96").unwrap());
/// ```
pub fn calculate_laundry_allowance(
    employee: &Employee,
    num_shifts: u32,
    per_shift_rate: Decimal,
    weekly_cap: Decimal,
    step_number: u32,
) -> LaundryAllowanceResult {
    let has_tag = employee.tags.contains(&LAUNDRY_ALLOWANCE_TAG.to_string());

    if !has_tag {
        let audit_step = AuditStep {
            step_number,
            rule_id: "laundry_allowance".to_string(),
            rule_name: "Laundry Allowance".to_string(),
            clause_ref: LAUNDRY_ALLOWANCE_CLAUSE.to_string(),
            input: serde_json::json!({
                "employee_id": employee.id,
                "has_laundry_tag": false,
                "num_shifts": num_shifts
            }),
            output: serde_json::json!({
                "eligible": false,
                "amount": "0.00"
            }),
            reasoning: "Employee does not have 'laundry_allowance' tag - not eligible for laundry allowance".to_string(),
        };

        return LaundryAllowanceResult {
            allowance: None,
            audit_step,
        };
    }

    // Calculate the uncapped amount
    let units = Decimal::from(num_shifts);
    let uncapped_amount = units * per_shift_rate;

    // Apply weekly cap
    let (amount, cap_applied) = if uncapped_amount > weekly_cap {
        (weekly_cap, true)
    } else {
        (uncapped_amount, false)
    };

    let reasoning = if cap_applied {
        format!(
            "{} shifts × ${} = ${} (capped at weekly maximum ${})",
            num_shifts,
            per_shift_rate.normalize(),
            amount.normalize(),
            weekly_cap.normalize()
        )
    } else {
        format!(
            "{} shifts × ${} = ${}",
            num_shifts,
            per_shift_rate.normalize(),
            amount.normalize()
        )
    };

    let audit_step = AuditStep {
        step_number,
        rule_id: "laundry_allowance".to_string(),
        rule_name: "Laundry Allowance".to_string(),
        clause_ref: LAUNDRY_ALLOWANCE_CLAUSE.to_string(),
        input: serde_json::json!({
            "employee_id": employee.id,
            "has_laundry_tag": true,
            "num_shifts": num_shifts,
            "per_shift_rate": per_shift_rate.normalize().to_string(),
            "weekly_cap": weekly_cap.normalize().to_string()
        }),
        output: serde_json::json!({
            "eligible": true,
            "units": units.normalize().to_string(),
            "uncapped_amount": uncapped_amount.normalize().to_string(),
            "amount": amount.normalize().to_string(),
            "cap_applied": cap_applied
        }),
        reasoning,
    };

    let allowance = AllowancePayment {
        allowance_type: "laundry".to_string(),
        description: "Laundry Allowance".to_string(),
        units,
        rate: per_shift_rate,
        amount,
        clause_ref: LAUNDRY_ALLOWANCE_CLAUSE.to_string(),
    };

    LaundryAllowanceResult {
        allowance: Some(allowance),
        audit_step,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::EmploymentType;
    use chrono::NaiveDate;
    use std::str::FromStr;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn create_test_employee(tags: Vec<String>) -> Employee {
        Employee {
            id: "emp_001".to_string(),
            employment_type: EmploymentType::FullTime,
            classification_code: "dce_level_3".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1990, 1, 15).unwrap(),
            employment_start_date: NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
            base_hourly_rate: None,
            tags,
        }
    }

    /// LA-001: 1 shift with laundry tag
    #[test]
    fn test_la_001_one_shift_with_laundry_tag() {
        let employee = create_test_employee(vec!["laundry_allowance".to_string()]);
        let result = calculate_laundry_allowance(&employee, 1, dec("0.32"), dec("1.49"), 1);

        assert!(result.allowance.is_some());
        let allowance = result.allowance.unwrap();

        assert_eq!(allowance.allowance_type, "laundry");
        assert_eq!(allowance.description, "Laundry Allowance");
        assert_eq!(allowance.units, dec("1"));
        assert_eq!(allowance.rate, dec("0.32"));
        assert_eq!(allowance.amount, dec("0.32"));
        assert_eq!(allowance.clause_ref, "15.2(b)");

        // Verify audit step
        assert_eq!(result.audit_step.rule_id, "laundry_allowance");
        assert_eq!(result.audit_step.clause_ref, "15.2(b)");
        assert!(result.audit_step.output["eligible"].as_bool().unwrap());
        assert!(!result.audit_step.output["cap_applied"].as_bool().unwrap());
    }

    /// LA-002: 3 shifts with laundry tag
    #[test]
    fn test_la_002_three_shifts_with_laundry_tag() {
        let employee = create_test_employee(vec!["laundry_allowance".to_string()]);
        let result = calculate_laundry_allowance(&employee, 3, dec("0.32"), dec("1.49"), 1);

        assert!(result.allowance.is_some());
        let allowance = result.allowance.unwrap();

        assert_eq!(allowance.units, dec("3"));
        assert_eq!(allowance.rate, dec("0.32"));
        assert_eq!(allowance.amount, dec("0.96")); // 3 * 0.32 = 0.96
        assert!(!result.audit_step.output["cap_applied"].as_bool().unwrap());
    }

    /// LA-003: 5 shifts hits cap
    #[test]
    fn test_la_003_five_shifts_hits_cap() {
        let employee = create_test_employee(vec!["laundry_allowance".to_string()]);
        let result = calculate_laundry_allowance(&employee, 5, dec("0.32"), dec("1.49"), 1);

        assert!(result.allowance.is_some());
        let allowance = result.allowance.unwrap();

        assert_eq!(allowance.units, dec("5"));
        assert_eq!(allowance.rate, dec("0.32"));
        // 5 * 0.32 = 1.60, capped at 1.49
        assert_eq!(allowance.amount, dec("1.49"));
        assert!(result.audit_step.output["cap_applied"].as_bool().unwrap());
        assert_eq!(
            result.audit_step.output["uncapped_amount"]
                .as_str()
                .unwrap(),
            "1.6"
        );
        assert!(result.audit_step.reasoning.contains("capped"));
    }

    /// LA-004: 6 shifts exceeds cap
    #[test]
    fn test_la_004_six_shifts_exceeds_cap() {
        let employee = create_test_employee(vec!["laundry_allowance".to_string()]);
        let result = calculate_laundry_allowance(&employee, 6, dec("0.32"), dec("1.49"), 1);

        assert!(result.allowance.is_some());
        let allowance = result.allowance.unwrap();

        assert_eq!(allowance.units, dec("6"));
        assert_eq!(allowance.rate, dec("0.32"));
        // 6 * 0.32 = 1.92, capped at 1.49
        assert_eq!(allowance.amount, dec("1.49"));
        assert!(result.audit_step.output["cap_applied"].as_bool().unwrap());
        assert_eq!(
            result.audit_step.output["uncapped_amount"]
                .as_str()
                .unwrap(),
            "1.92"
        );
    }

    /// LA-005: no laundry tag
    #[test]
    fn test_la_005_no_laundry_tag() {
        let employee = create_test_employee(vec![]); // No tags
        let result = calculate_laundry_allowance(&employee, 3, dec("0.32"), dec("1.49"), 1);

        assert!(result.allowance.is_none());
        assert!(!result.audit_step.output["eligible"].as_bool().unwrap());
        assert!(result
            .audit_step
            .reasoning
            .contains("does not have 'laundry_allowance' tag"));
    }

    #[test]
    fn test_employee_with_other_tags_but_not_laundry() {
        let employee = create_test_employee(vec!["qualified".to_string(), "night_shift".to_string()]);
        let result = calculate_laundry_allowance(&employee, 3, dec("0.32"), dec("1.49"), 1);

        assert!(result.allowance.is_none());
        assert!(!result.audit_step.output["eligible"].as_bool().unwrap());
    }

    #[test]
    fn test_employee_with_laundry_and_other_tags() {
        let employee = create_test_employee(vec![
            "qualified".to_string(),
            "laundry_allowance".to_string(),
            "night_shift".to_string(),
        ]);
        let result = calculate_laundry_allowance(&employee, 2, dec("0.32"), dec("1.49"), 1);

        assert!(result.allowance.is_some());
        let allowance = result.allowance.unwrap();
        assert_eq!(allowance.amount, dec("0.64")); // 2 * 0.32 = 0.64
    }

    #[test]
    fn test_audit_step_has_correct_step_number() {
        let employee = create_test_employee(vec!["laundry_allowance".to_string()]);
        let result = calculate_laundry_allowance(&employee, 1, dec("0.32"), dec("1.49"), 5);

        assert_eq!(result.audit_step.step_number, 5);
    }

    #[test]
    fn test_zero_shifts_returns_zero_amount() {
        let employee = create_test_employee(vec!["laundry_allowance".to_string()]);
        let result = calculate_laundry_allowance(&employee, 0, dec("0.32"), dec("1.49"), 1);

        assert!(result.allowance.is_some());
        let allowance = result.allowance.unwrap();
        assert_eq!(allowance.units, dec("0"));
        assert_eq!(allowance.amount, dec("0"));
    }

    #[test]
    fn test_exactly_at_cap_does_not_apply_cap() {
        // 4.65625 shifts at $0.32 = $1.49 exactly, but shifts must be whole numbers
        // Let's test with values that hit cap exactly
        let employee = create_test_employee(vec!["laundry_allowance".to_string()]);
        // Using a rate where 3 shifts exactly equals the cap
        let result = calculate_laundry_allowance(&employee, 3, dec("0.50"), dec("1.50"), 1);

        assert!(result.allowance.is_some());
        let allowance = result.allowance.unwrap();
        // 3 * 0.50 = 1.50, exactly at cap - should NOT apply cap
        assert_eq!(allowance.amount, dec("1.50"));
        assert!(!result.audit_step.output["cap_applied"].as_bool().unwrap());
    }

    #[test]
    fn test_casual_employee_gets_laundry_allowance() {
        let mut employee = create_test_employee(vec!["laundry_allowance".to_string()]);
        employee.employment_type = EmploymentType::Casual;

        let result = calculate_laundry_allowance(&employee, 3, dec("0.32"), dec("1.49"), 1);

        assert!(result.allowance.is_some());
        let allowance = result.allowance.unwrap();
        assert_eq!(allowance.amount, dec("0.96"));
    }
}

/// Integration tests for allowances in CalculationResult (US-5.2)
#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::models::{
        AuditTrace, CalculationResult, EmploymentType, PayCategory, PayLine, PayPeriod, PayTotals,
    };
    use chrono::{NaiveDate, Utc};
    use std::str::FromStr;
    use uuid::Uuid;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn create_test_employee_with_tags(
        employment_type: EmploymentType,
        tags: Vec<String>,
    ) -> Employee {
        Employee {
            id: "emp_001".to_string(),
            employment_type,
            classification_code: "dce_level_3".to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1990, 1, 15).unwrap(),
            employment_start_date: NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
            base_hourly_rate: None,
            tags,
        }
    }

    fn create_pay_period() -> PayPeriod {
        PayPeriod {
            start_date: NaiveDate::from_ymd_opt(2026, 1, 13).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 1, 19).unwrap(),
            public_holidays: vec![],
        }
    }

    fn create_ordinary_pay_line(shift_id: &str, date: NaiveDate, amount: Decimal) -> PayLine {
        PayLine {
            date,
            shift_id: shift_id.to_string(),
            category: PayCategory::Ordinary,
            hours: dec("8.0"),
            rate: dec("28.54"),
            amount,
            clause_ref: "22.1".to_string(),
        }
    }

    /// CRAL-001: single shift with laundry allowance
    /// Full-time, 8h Monday shift with laundry tag
    /// Expected: pay_lines_total = $228.32, allowances_total = $0.32, gross_pay = $228.64
    #[test]
    fn test_cral_001_single_shift_with_laundry_allowance() {
        let employee = create_test_employee_with_tags(
            EmploymentType::FullTime,
            vec!["laundry_allowance".to_string()],
        );
        let pay_period = create_pay_period();

        // Single 8-hour shift on Monday 2026-01-13
        let shift_date = NaiveDate::from_ymd_opt(2026, 1, 13).unwrap();
        let pay_line = create_ordinary_pay_line("shift_001", shift_date, dec("228.32"));

        // Calculate laundry allowance for 1 shift
        let laundry_result = calculate_laundry_allowance(&employee, 1, dec("0.32"), dec("1.49"), 4);

        // Build the calculation result
        let pay_lines = vec![pay_line];
        let allowances = match laundry_result.allowance {
            Some(a) => vec![a],
            None => vec![],
        };

        // Calculate totals
        let pay_lines_total: Decimal = pay_lines.iter().map(|pl| pl.amount).sum();
        let allowances_total: Decimal = allowances.iter().map(|a| a.amount).sum();
        let gross_pay = pay_lines_total + allowances_total;

        let result = CalculationResult {
            calculation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            engine_version: "1.0.0".to_string(),
            employee_id: employee.id.clone(),
            pay_period,
            pay_lines,
            allowances,
            totals: PayTotals {
                gross_pay,
                ordinary_hours: dec("8.0"),
                overtime_hours: dec("0"),
                penalty_hours: dec("0"),
                allowances_total,
            },
            audit_trace: AuditTrace {
                steps: vec![laundry_result.audit_step],
                warnings: vec![],
                duration_us: 1000,
            },
        };

        // Verify acceptance criteria
        assert_eq!(result.pay_lines.len(), 1);
        assert_eq!(result.allowances.len(), 1);

        // Verify pay_lines_total
        let calculated_pay_lines_total: Decimal =
            result.pay_lines.iter().map(|pl| pl.amount).sum();
        assert_eq!(calculated_pay_lines_total, dec("228.32"));

        // Verify allowances_total
        assert_eq!(result.totals.allowances_total, dec("0.32"));
        let calculated_allowances_total: Decimal =
            result.allowances.iter().map(|a| a.amount).sum();
        assert_eq!(calculated_allowances_total, dec("0.32"));

        // Verify gross_pay includes allowances
        assert_eq!(result.totals.gross_pay, dec("228.64"));
        assert_eq!(
            result.totals.gross_pay,
            calculated_pay_lines_total + calculated_allowances_total
        );

        // Verify allowance details
        let allowance = &result.allowances[0];
        assert_eq!(allowance.allowance_type, "laundry");
        assert_eq!(allowance.description, "Laundry Allowance");
        assert_eq!(allowance.clause_ref, "15.2(b)");

        // Verify audit trace includes allowance calculation
        assert!(result
            .audit_trace
            .steps
            .iter()
            .any(|s| s.rule_id == "laundry_allowance"));
    }

    /// CRAL-002: multiple shifts hit laundry cap
    /// Casual, 5 shifts with laundry tag
    /// Expected: allowances_total = $1.49 (capped at weekly maximum)
    #[test]
    fn test_cral_002_multiple_shifts_hit_laundry_cap() {
        let employee = create_test_employee_with_tags(
            EmploymentType::Casual,
            vec!["laundry_allowance".to_string()],
        );
        let pay_period = create_pay_period();

        // 5 shifts - create pay lines for each
        let mut pay_lines = Vec::new();
        for i in 0..5 {
            let date = NaiveDate::from_ymd_opt(2026, 1, 13 + i).unwrap();
            // Casual rate: 28.54 * 1.25 = 35.675, 8h = 285.40
            let pay_line = PayLine {
                date,
                shift_id: format!("shift_{:03}", i + 1),
                category: PayCategory::OrdinaryCasual,
                hours: dec("8.0"),
                rate: dec("35.675"),
                amount: dec("285.40"),
                clause_ref: "22.1".to_string(),
            };
            pay_lines.push(pay_line);
        }

        // Calculate laundry allowance for 5 shifts (should hit cap)
        let laundry_result = calculate_laundry_allowance(&employee, 5, dec("0.32"), dec("1.49"), 1);

        let allowances = match laundry_result.allowance {
            Some(a) => vec![a],
            None => vec![],
        };

        // Calculate totals
        let pay_lines_total: Decimal = pay_lines.iter().map(|pl| pl.amount).sum();
        let allowances_total: Decimal = allowances.iter().map(|a| a.amount).sum();
        let gross_pay = pay_lines_total + allowances_total;

        let result = CalculationResult {
            calculation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            engine_version: "1.0.0".to_string(),
            employee_id: employee.id.clone(),
            pay_period,
            pay_lines,
            allowances,
            totals: PayTotals {
                gross_pay,
                ordinary_hours: dec("40.0"),
                overtime_hours: dec("0"),
                penalty_hours: dec("0"),
                allowances_total,
            },
            audit_trace: AuditTrace {
                steps: vec![laundry_result.audit_step],
                warnings: vec![],
                duration_us: 1000,
            },
        };

        // Verify allowances_total is capped at $1.49
        // 5 shifts * $0.32 = $1.60, but cap is $1.49
        assert_eq!(result.totals.allowances_total, dec("1.49"));

        let calculated_allowances_total: Decimal =
            result.allowances.iter().map(|a| a.amount).sum();
        assert_eq!(calculated_allowances_total, dec("1.49"));

        // Verify the cap was applied
        let allowance = &result.allowances[0];
        assert_eq!(allowance.units, dec("5"));
        assert_eq!(allowance.amount, dec("1.49")); // Capped

        // Verify gross pay includes capped allowance
        // 5 * 285.40 = 1427.00
        let expected_pay_lines_total = dec("1427.00");
        assert_eq!(
            result.totals.gross_pay,
            expected_pay_lines_total + dec("1.49")
        );
    }

    /// CRAL-003: no allowances
    /// Full-time, 8h shift without laundry tag
    /// Expected: allowances_total = $0.00, allowances array length = 0
    #[test]
    fn test_cral_003_no_allowances() {
        let employee = create_test_employee_with_tags(
            EmploymentType::FullTime,
            vec![], // No laundry tag
        );
        let pay_period = create_pay_period();

        // Single 8-hour shift
        let shift_date = NaiveDate::from_ymd_opt(2026, 1, 13).unwrap();
        let pay_line = create_ordinary_pay_line("shift_001", shift_date, dec("228.32"));

        // Calculate laundry allowance - should return None
        let laundry_result = calculate_laundry_allowance(&employee, 1, dec("0.32"), dec("1.49"), 4);

        // No allowance should be returned
        assert!(laundry_result.allowance.is_none());

        let pay_lines = vec![pay_line];
        let allowances: Vec<AllowancePayment> = vec![]; // Empty

        // Calculate totals
        let pay_lines_total: Decimal = pay_lines.iter().map(|pl| pl.amount).sum();
        let allowances_total = Decimal::ZERO;
        let gross_pay = pay_lines_total + allowances_total;

        let result = CalculationResult {
            calculation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            engine_version: "1.0.0".to_string(),
            employee_id: employee.id.clone(),
            pay_period,
            pay_lines,
            allowances,
            totals: PayTotals {
                gross_pay,
                ordinary_hours: dec("8.0"),
                overtime_hours: dec("0"),
                penalty_hours: dec("0"),
                allowances_total,
            },
            audit_trace: AuditTrace {
                steps: vec![laundry_result.audit_step],
                warnings: vec![],
                duration_us: 1000,
            },
        };

        // Verify allowances array is empty
        assert_eq!(result.allowances.len(), 0);

        // Verify allowances_total is zero
        assert_eq!(result.totals.allowances_total, dec("0"));

        // Verify gross pay equals pay lines total (no allowances)
        assert_eq!(result.totals.gross_pay, dec("228.32"));
        assert_eq!(result.totals.gross_pay, pay_lines_total);

        // Verify audit trace still records the allowance check (even though not eligible)
        assert!(result
            .audit_trace
            .steps
            .iter()
            .any(|s| s.rule_id == "laundry_allowance"));
    }

    /// Test that allowances appear after pay lines in the result structure
    #[test]
    fn test_allowances_appear_after_pay_lines() {
        let employee = create_test_employee_with_tags(
            EmploymentType::FullTime,
            vec!["laundry_allowance".to_string()],
        );
        let pay_period = create_pay_period();

        let shift_date = NaiveDate::from_ymd_opt(2026, 1, 13).unwrap();
        let pay_line = create_ordinary_pay_line("shift_001", shift_date, dec("228.32"));

        let laundry_result = calculate_laundry_allowance(&employee, 1, dec("0.32"), dec("1.49"), 4);
        let allowances = match laundry_result.allowance {
            Some(a) => vec![a],
            None => vec![],
        };

        let result = CalculationResult {
            calculation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            engine_version: "1.0.0".to_string(),
            employee_id: employee.id.clone(),
            pay_period,
            pay_lines: vec![pay_line],
            allowances,
            totals: PayTotals {
                gross_pay: dec("228.64"),
                ordinary_hours: dec("8.0"),
                overtime_hours: dec("0"),
                penalty_hours: dec("0"),
                allowances_total: dec("0.32"),
            },
            audit_trace: AuditTrace {
                steps: vec![laundry_result.audit_step],
                warnings: vec![],
                duration_us: 1000,
            },
        };

        // Verify structure: pay_lines field exists and comes before allowances in serialization
        let json = serde_json::to_string(&result).unwrap();

        // In CalculationResult struct definition, pay_lines is declared before allowances
        // This verifies the JSON serialization order
        let pay_lines_pos = json.find("\"pay_lines\"").unwrap();
        let allowances_pos = json.find("\"allowances\"").unwrap();
        assert!(
            pay_lines_pos < allowances_pos,
            "pay_lines should appear before allowances in JSON"
        );
    }

    /// Test gross_pay calculation is correct with multiple pay lines and allowances
    #[test]
    fn test_gross_pay_includes_pay_lines_and_allowances() {
        let employee = create_test_employee_with_tags(
            EmploymentType::FullTime,
            vec!["laundry_allowance".to_string()],
        );
        let pay_period = create_pay_period();

        // Multiple pay lines
        let pay_lines = vec![
            create_ordinary_pay_line(
                "shift_001",
                NaiveDate::from_ymd_opt(2026, 1, 13).unwrap(),
                dec("228.32"),
            ),
            create_ordinary_pay_line(
                "shift_002",
                NaiveDate::from_ymd_opt(2026, 1, 14).unwrap(),
                dec("228.32"),
            ),
            create_ordinary_pay_line(
                "shift_003",
                NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
                dec("228.32"),
            ),
        ];

        // Calculate laundry allowance for 3 shifts
        let laundry_result = calculate_laundry_allowance(&employee, 3, dec("0.32"), dec("1.49"), 1);
        let allowances = match laundry_result.allowance {
            Some(a) => vec![a],
            None => vec![],
        };

        let pay_lines_total: Decimal = pay_lines.iter().map(|pl| pl.amount).sum();
        let allowances_total: Decimal = allowances.iter().map(|a| a.amount).sum();
        let gross_pay = pay_lines_total + allowances_total;

        let result = CalculationResult {
            calculation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            engine_version: "1.0.0".to_string(),
            employee_id: employee.id.clone(),
            pay_period,
            pay_lines,
            allowances,
            totals: PayTotals {
                gross_pay,
                ordinary_hours: dec("24.0"),
                overtime_hours: dec("0"),
                penalty_hours: dec("0"),
                allowances_total,
            },
            audit_trace: AuditTrace {
                steps: vec![laundry_result.audit_step],
                warnings: vec![],
                duration_us: 1000,
            },
        };

        // Pay lines: 3 * 228.32 = 684.96
        // Allowances: 3 * 0.32 = 0.96
        // Gross: 684.96 + 0.96 = 685.92
        assert_eq!(
            result.pay_lines.iter().map(|pl| pl.amount).sum::<Decimal>(),
            dec("684.96")
        );
        assert_eq!(result.totals.allowances_total, dec("0.96"));
        assert_eq!(result.totals.gross_pay, dec("685.92"));
    }
}
