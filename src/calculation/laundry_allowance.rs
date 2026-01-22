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
