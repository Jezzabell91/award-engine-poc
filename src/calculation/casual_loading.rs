//! Casual loading calculation functionality.
//!
//! This module provides functions for applying the casual loading multiplier
//! to base rates for casual employees as per the Aged Care Award 2010.

use rust_decimal::Decimal;

use crate::models::{AuditStep, Employee, EmploymentType};

/// Returns the casual loading multiplier as defined in clause 10.4(b).
///
/// The multiplier is 1.25 (25% loading).
pub fn casual_loading_multiplier() -> Decimal {
    Decimal::new(125, 2)
}

/// The result of applying casual loading, including the rate and audit step.
#[derive(Debug, Clone)]
pub struct CasualLoadingResult {
    /// The rate after applying casual loading (if applicable).
    pub loaded_rate: Decimal,
    /// The audit step recording this calculation.
    pub audit_step: AuditStep,
}

/// Applies casual loading to a base rate for casual employees.
///
/// For casual employees, a 25% loading is applied to the base rate as per
/// clause 10.4(b) of the Aged Care Award 2010. For full-time and part-time
/// employees, the base rate is returned unchanged.
///
/// # Arguments
///
/// * `base_rate` - The base hourly rate before any loading
/// * `employee` - The employee to apply loading for
/// * `step_number` - The step number for audit trail sequencing
///
/// # Returns
///
/// Returns a `CasualLoadingResult` containing the loaded rate and an audit step.
///
/// # Award Reference
///
/// Clause 10.4(b) of the Aged Care Award 2010 specifies the 25% casual loading.
///
/// # Examples
///
/// ```
/// use award_engine::calculation::apply_casual_loading;
/// use award_engine::models::{Employee, EmploymentType};
/// use chrono::NaiveDate;
/// use rust_decimal::Decimal;
/// use std::str::FromStr;
///
/// let employee = Employee {
///     id: "emp_001".to_string(),
///     employment_type: EmploymentType::Casual,
///     classification_code: "dce_level_3".to_string(),
///     date_of_birth: NaiveDate::from_ymd_opt(1990, 1, 15).unwrap(),
///     employment_start_date: NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
///     base_hourly_rate: None,
///     tags: vec![],
/// };
///
/// let result = apply_casual_loading(Decimal::from_str("28.54").unwrap(), &employee, 1);
/// assert_eq!(result.loaded_rate, Decimal::from_str("35.675").unwrap());
/// ```
pub fn apply_casual_loading(
    base_rate: Decimal,
    employee: &Employee,
    step_number: u32,
) -> CasualLoadingResult {
    let employment_type_str = match employee.employment_type {
        EmploymentType::FullTime => "full_time",
        EmploymentType::PartTime => "part_time",
        EmploymentType::Casual => "casual",
    };

    if employee.is_casual() {
        let loaded_rate = base_rate * casual_loading_multiplier();
        let multiplier = casual_loading_multiplier();

        let audit_step = AuditStep {
            step_number,
            rule_id: "casual_loading".to_string(),
            rule_name: "Casual Loading".to_string(),
            clause_ref: "10.4(b)".to_string(),
            input: serde_json::json!({
                "base_rate": base_rate.normalize().to_string(),
                "employment_type": employment_type_str
            }),
            output: serde_json::json!({
                "loaded_rate": loaded_rate.normalize().to_string(),
                "loading_applied": true,
                "multiplier": multiplier.normalize().to_string()
            }),
            reasoning: format!(
                "${} x {} = ${}",
                base_rate.normalize(),
                multiplier.normalize(),
                loaded_rate.normalize()
            ),
        };

        CasualLoadingResult {
            loaded_rate,
            audit_step,
        }
    } else {
        let audit_step = AuditStep {
            step_number,
            rule_id: "casual_loading".to_string(),
            rule_name: "Casual Loading".to_string(),
            clause_ref: "10.4(b)".to_string(),
            input: serde_json::json!({
                "base_rate": base_rate.to_string(),
                "employment_type": employment_type_str
            }),
            output: serde_json::json!({
                "loaded_rate": base_rate.to_string(),
                "loading_applied": false
            }),
            reasoning: format!(
                "No casual loading applied - employee is {} (not casual)",
                employment_type_str
            ),
        };

        CasualLoadingResult {
            loaded_rate: base_rate,
            audit_step,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
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

    /// CL-001: casual gets 25% loading
    #[test]
    fn test_casual_gets_25_percent_loading() {
        let employee = create_test_employee(EmploymentType::Casual);
        let result = apply_casual_loading(dec("28.54"), &employee, 1);

        assert_eq!(result.loaded_rate, dec("35.675"));
        assert_eq!(result.audit_step.rule_id, "casual_loading");
        assert_eq!(result.audit_step.clause_ref, "10.4(b)");
        assert_eq!(
            result.audit_step.input["base_rate"].as_str().unwrap(),
            "28.54"
        );
        assert_eq!(
            result.audit_step.input["employment_type"].as_str().unwrap(),
            "casual"
        );
        assert_eq!(
            result.audit_step.output["loaded_rate"].as_str().unwrap(),
            "35.675"
        );
        assert!(result.audit_step.reasoning.contains("28.54"));
        assert!(result.audit_step.reasoning.contains("1.25"));
        assert!(result.audit_step.reasoning.contains("35.675"));
    }

    /// CL-002: fulltime gets no loading
    #[test]
    fn test_fulltime_gets_no_loading() {
        let employee = create_test_employee(EmploymentType::FullTime);
        let result = apply_casual_loading(dec("28.54"), &employee, 1);

        assert_eq!(result.loaded_rate, dec("28.54"));
        assert_eq!(result.audit_step.rule_id, "casual_loading");
        assert_eq!(result.audit_step.clause_ref, "10.4(b)");
        assert_eq!(
            result.audit_step.input["employment_type"].as_str().unwrap(),
            "full_time"
        );
        assert_eq!(
            result.audit_step.output["loading_applied"]
                .as_bool()
                .unwrap(),
            false
        );
    }

    /// CL-003: parttime gets no loading
    #[test]
    fn test_parttime_gets_no_loading() {
        let employee = create_test_employee(EmploymentType::PartTime);
        let result = apply_casual_loading(dec("28.54"), &employee, 1);

        assert_eq!(result.loaded_rate, dec("28.54"));
        assert_eq!(result.audit_step.rule_id, "casual_loading");
        assert_eq!(result.audit_step.clause_ref, "10.4(b)");
        assert_eq!(
            result.audit_step.input["employment_type"].as_str().unwrap(),
            "part_time"
        );
        assert_eq!(
            result.audit_step.output["loading_applied"]
                .as_bool()
                .unwrap(),
            false
        );
    }

    /// CL-004: casual loading on different rate
    #[test]
    fn test_casual_loading_on_different_rate() {
        let employee = create_test_employee(EmploymentType::Casual);
        let result = apply_casual_loading(dec("25.00"), &employee, 1);

        assert_eq!(result.loaded_rate, dec("31.25"));
    }

    /// CL-005: casual loading on zero rate
    #[test]
    fn test_casual_loading_on_zero_rate() {
        let employee = create_test_employee(EmploymentType::Casual);
        let result = apply_casual_loading(dec("0.00"), &employee, 1);

        assert_eq!(result.loaded_rate, dec("0.00"));
    }

    #[test]
    fn test_audit_step_has_correct_step_number() {
        let employee = create_test_employee(EmploymentType::Casual);
        let result = apply_casual_loading(dec("28.54"), &employee, 5);

        assert_eq!(result.audit_step.step_number, 5);
    }

    #[test]
    fn test_casual_loading_multiplier_is_exactly_1_25() {
        assert_eq!(casual_loading_multiplier(), dec("1.25"));
    }

    #[test]
    fn test_audit_reasoning_explains_calculation_for_casual() {
        let employee = create_test_employee(EmploymentType::Casual);
        let result = apply_casual_loading(dec("28.54"), &employee, 1);

        // Should contain the calculation: "$28.54 x 1.25 = $35.675"
        assert!(result.audit_step.reasoning.contains("$28.54"));
        assert!(result.audit_step.reasoning.contains("x"));
        assert!(result.audit_step.reasoning.contains("1.25"));
        assert!(result.audit_step.reasoning.contains("$35.675"));
    }

    #[test]
    fn test_audit_reasoning_explains_no_loading_for_fulltime() {
        let employee = create_test_employee(EmploymentType::FullTime);
        let result = apply_casual_loading(dec("28.54"), &employee, 1);

        assert!(result.audit_step.reasoning.contains("No casual loading"));
        assert!(result.audit_step.reasoning.contains("full_time"));
    }

    #[test]
    fn test_audit_output_shows_loading_applied_true_for_casual() {
        let employee = create_test_employee(EmploymentType::Casual);
        let result = apply_casual_loading(dec("28.54"), &employee, 1);

        assert_eq!(
            result.audit_step.output["loading_applied"]
                .as_bool()
                .unwrap(),
            true
        );
        assert_eq!(
            result.audit_step.output["multiplier"].as_str().unwrap(),
            "1.25"
        );
    }
}
