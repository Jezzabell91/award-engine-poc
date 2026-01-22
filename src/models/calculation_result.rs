//! Calculation result models for the Award Interpretation Engine.
//!
//! This module contains the [`CalculationResult`] type and its associated structures
//! that capture all outputs from a pay calculation, including pay lines, allowances,
//! totals, and audit traces.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::PayPeriod;

/// Represents the category of pay for a pay line.
///
/// Different categories have different rates and are used to distinguish
/// between ordinary time, casual loading, weekend penalties, and overtime.
///
/// # Example
///
/// ```
/// use award_engine::models::PayCategory;
///
/// let category = PayCategory::Ordinary;
/// assert_eq!(format!("{:?}", category), "Ordinary");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayCategory {
    /// Ordinary hours for permanent employees.
    Ordinary,
    /// Ordinary hours for casual employees (includes casual loading).
    OrdinaryCasual,
    /// Saturday penalty rates for permanent employees.
    Saturday,
    /// Saturday penalty rates for casual employees.
    SaturdayCasual,
    /// Sunday penalty rates for permanent employees.
    Sunday,
    /// Sunday penalty rates for casual employees.
    SundayCasual,
    /// Overtime at 150% rate.
    Overtime150,
    /// Overtime at 200% rate.
    Overtime200,
}

/// Represents a single line item in a pay calculation.
///
/// Each pay line captures the hours worked in a specific category,
/// the applicable rate, and the resulting amount.
///
/// # Example
///
/// ```
/// use award_engine::models::{PayLine, PayCategory};
/// use rust_decimal::Decimal;
/// use chrono::NaiveDate;
/// use std::str::FromStr;
///
/// let pay_line = PayLine {
///     date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
///     shift_id: "shift_001".to_string(),
///     category: PayCategory::Ordinary,
///     hours: Decimal::from_str("8.0").unwrap(),
///     rate: Decimal::from_str("28.54").unwrap(),
///     amount: Decimal::from_str("228.32").unwrap(),
///     clause_ref: "14.2".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayLine {
    /// The date this pay line applies to.
    pub date: NaiveDate,
    /// The ID of the shift this pay line originated from.
    pub shift_id: String,
    /// The category of pay (e.g., Ordinary, Overtime150).
    pub category: PayCategory,
    /// The number of hours worked in this category.
    pub hours: Decimal,
    /// The hourly rate for this category.
    pub rate: Decimal,
    /// The total amount for this pay line (hours * rate).
    pub amount: Decimal,
    /// Reference to the award clause that justifies this pay line.
    pub clause_ref: String,
}

/// Represents an allowance payment.
///
/// Allowances are additional payments for specific conditions or expenses,
/// such as laundry allowance or travel allowance.
///
/// # Example
///
/// ```
/// use award_engine::models::AllowancePayment;
/// use rust_decimal::Decimal;
/// use std::str::FromStr;
///
/// let allowance = AllowancePayment {
///     allowance_type: "laundry".to_string(),
///     description: "Laundry allowance for uniform cleaning".to_string(),
///     units: Decimal::from_str("5.0").unwrap(),
///     rate: Decimal::from_str("0.32").unwrap(),
///     amount: Decimal::from_str("1.49").unwrap(),
///     clause_ref: "20.2".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllowancePayment {
    /// The type of allowance (e.g., "laundry", "travel", "meal").
    #[serde(rename = "type")]
    pub allowance_type: String,
    /// A description of the allowance.
    pub description: String,
    /// The number of units (e.g., shifts, kilometers).
    pub units: Decimal,
    /// The rate per unit.
    pub rate: Decimal,
    /// The total amount for this allowance (may be capped).
    pub amount: Decimal,
    /// Reference to the award clause that justifies this allowance.
    pub clause_ref: String,
}

/// Aggregated totals for a pay calculation.
///
/// This struct provides a summary of all pay components, making it easy
/// to see the overall result of a calculation.
///
/// # Example
///
/// ```
/// use award_engine::models::PayTotals;
/// use rust_decimal::Decimal;
/// use std::str::FromStr;
///
/// let totals = PayTotals {
///     gross_pay: Decimal::from_str("1500.00").unwrap(),
///     ordinary_hours: Decimal::from_str("38.0").unwrap(),
///     overtime_hours: Decimal::from_str("4.0").unwrap(),
///     penalty_hours: Decimal::from_str("8.0").unwrap(),
///     allowances_total: Decimal::from_str("5.60").unwrap(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PayTotals {
    /// The total gross pay (sum of all pay lines and allowances).
    pub gross_pay: Decimal,
    /// Total ordinary hours worked.
    pub ordinary_hours: Decimal,
    /// Total overtime hours worked.
    pub overtime_hours: Decimal,
    /// Total penalty hours worked (weekend/holiday).
    pub penalty_hours: Decimal,
    /// Total value of all allowances.
    pub allowances_total: Decimal,
}

/// A single step in the audit trace recording a calculation decision.
///
/// Each step captures the input, output, and reasoning for a rule application.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditStep {
    /// The sequential step number.
    pub step_number: u32,
    /// The unique identifier of the rule that was applied.
    pub rule_id: String,
    /// The human-readable name of the rule.
    pub rule_name: String,
    /// Reference to the award clause for this rule.
    pub clause_ref: String,
    /// The input data for this step.
    pub input: serde_json::Value,
    /// The output data from this step.
    pub output: serde_json::Value,
    /// Human-readable explanation of the decision.
    pub reasoning: String,
}

/// A warning generated during calculation.
///
/// Warnings indicate potential issues that don't prevent calculation
/// but may require attention.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditWarning {
    /// A code identifying the type of warning.
    pub code: String,
    /// A human-readable description of the warning.
    pub message: String,
    /// The severity level (e.g., "low", "medium", "high").
    pub severity: String,
}

/// The complete audit trace for a calculation.
///
/// Records every decision made during the calculation process for
/// transparency and compliance.
///
/// # Example
///
/// ```
/// use award_engine::models::AuditTrace;
///
/// let trace = AuditTrace {
///     steps: vec![],
///     warnings: vec![],
///     duration_us: 1234,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditTrace {
    /// The sequence of calculation steps.
    pub steps: Vec<AuditStep>,
    /// Any warnings generated during calculation.
    pub warnings: Vec<AuditWarning>,
    /// The total calculation duration in microseconds.
    pub duration_us: u64,
}

/// The complete result of a pay calculation.
///
/// This struct captures all outputs from the award interpretation engine,
/// including pay lines, allowances, totals, and a complete audit trace
/// for transparency and compliance.
///
/// # Example
///
/// ```
/// use award_engine::models::{CalculationResult, PayPeriod, PayTotals, AuditTrace};
/// use chrono::{Utc, NaiveDate};
/// use uuid::Uuid;
/// use rust_decimal::Decimal;
///
/// let result = CalculationResult {
///     calculation_id: Uuid::new_v4(),
///     timestamp: Utc::now(),
///     engine_version: "1.0.0".to_string(),
///     employee_id: "emp_001".to_string(),
///     pay_period: PayPeriod {
///         start_date: NaiveDate::from_ymd_opt(2026, 1, 13).unwrap(),
///         end_date: NaiveDate::from_ymd_opt(2026, 1, 26).unwrap(),
///         public_holidays: vec![],
///     },
///     pay_lines: vec![],
///     allowances: vec![],
///     totals: PayTotals {
///         gross_pay: Decimal::ZERO,
///         ordinary_hours: Decimal::ZERO,
///         overtime_hours: Decimal::ZERO,
///         penalty_hours: Decimal::ZERO,
///         allowances_total: Decimal::ZERO,
///     },
///     audit_trace: AuditTrace {
///         steps: vec![],
///         warnings: vec![],
///         duration_us: 0,
///     },
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalculationResult {
    /// Unique identifier for this calculation.
    pub calculation_id: Uuid,
    /// When the calculation was performed.
    pub timestamp: DateTime<Utc>,
    /// The version of the engine that performed the calculation.
    pub engine_version: String,
    /// The ID of the employee the calculation is for.
    pub employee_id: String,
    /// The pay period for this calculation.
    pub pay_period: PayPeriod,
    /// Individual pay lines making up the calculation.
    pub pay_lines: Vec<PayLine>,
    /// Allowance payments included in the calculation.
    pub allowances: Vec<AllowancePayment>,
    /// Aggregated totals for the calculation.
    pub totals: PayTotals,
    /// Complete audit trace of calculation decisions.
    pub audit_trace: AuditTrace,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    /// Helper function to create Decimal values from strings
    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn create_sample_pay_period() -> PayPeriod {
        PayPeriod {
            start_date: NaiveDate::from_ymd_opt(2026, 1, 13).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 1, 26).unwrap(),
            public_holidays: vec![],
        }
    }

    fn create_sample_pay_line(amount: Decimal) -> PayLine {
        PayLine {
            date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            shift_id: "shift_001".to_string(),
            category: PayCategory::Ordinary,
            hours: dec("8.0"),
            rate: dec("28.54"),
            amount,
            clause_ref: "14.2".to_string(),
        }
    }

    fn create_sample_allowance(amount: Decimal) -> AllowancePayment {
        AllowancePayment {
            allowance_type: "laundry".to_string(),
            description: "Laundry allowance".to_string(),
            units: dec("5.0"),
            rate: dec("0.32"),
            amount,
            clause_ref: "20.2".to_string(),
        }
    }

    fn create_sample_audit_trace() -> AuditTrace {
        AuditTrace {
            steps: vec![],
            warnings: vec![],
            duration_us: 1000,
        }
    }

    /// CR-001: gross_pay equals sum of pay_lines
    #[test]
    fn test_gross_pay_equals_sum_of_pay_lines() {
        let pay_lines = vec![
            create_sample_pay_line(dec("100.00")),
            create_sample_pay_line(dec("50.00")),
            create_sample_pay_line(dec("75.50")),
        ];

        let sum: Decimal = pay_lines.iter().map(|pl| pl.amount).sum();
        assert_eq!(sum, dec("225.50"));

        let result = CalculationResult {
            calculation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            engine_version: "1.0.0".to_string(),
            employee_id: "emp_001".to_string(),
            pay_period: create_sample_pay_period(),
            pay_lines,
            allowances: vec![],
            totals: PayTotals {
                gross_pay: dec("225.50"),
                ordinary_hours: dec("24.0"),
                overtime_hours: dec("0"),
                penalty_hours: dec("0"),
                allowances_total: dec("0"),
            },
            audit_trace: create_sample_audit_trace(),
        };

        let calculated_sum: Decimal = result.pay_lines.iter().map(|pl| pl.amount).sum();
        assert_eq!(result.totals.gross_pay, calculated_sum);
    }

    #[test]
    fn test_pay_category_serialization() {
        let category = PayCategory::Ordinary;
        let json = serde_json::to_string(&category).unwrap();
        assert_eq!(json, "\"ordinary\"");

        let category = PayCategory::OrdinaryCasual;
        let json = serde_json::to_string(&category).unwrap();
        assert_eq!(json, "\"ordinary_casual\"");

        let category = PayCategory::Overtime150;
        let json = serde_json::to_string(&category).unwrap();
        assert_eq!(json, "\"overtime150\"");
    }

    #[test]
    fn test_pay_category_deserialization() {
        let category: PayCategory = serde_json::from_str("\"saturday_casual\"").unwrap();
        assert_eq!(category, PayCategory::SaturdayCasual);

        let category: PayCategory = serde_json::from_str("\"sunday\"").unwrap();
        assert_eq!(category, PayCategory::Sunday);

        let category: PayCategory = serde_json::from_str("\"overtime200\"").unwrap();
        assert_eq!(category, PayCategory::Overtime200);
    }

    #[test]
    fn test_pay_line_serialization() {
        let pay_line = PayLine {
            date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            shift_id: "shift_001".to_string(),
            category: PayCategory::Ordinary,
            hours: dec("8.0"),
            rate: dec("28.54"),
            amount: dec("228.32"),
            clause_ref: "14.2".to_string(),
        };

        let json = serde_json::to_string(&pay_line).unwrap();
        assert!(json.contains("\"date\":\"2026-01-15\""));
        assert!(json.contains("\"shift_id\":\"shift_001\""));
        assert!(json.contains("\"category\":\"ordinary\""));
        assert!(json.contains("\"hours\":\"8.0\""));
        assert!(json.contains("\"clause_ref\":\"14.2\""));
    }

    #[test]
    fn test_pay_line_deserialization() {
        let json = r#"{
            "date": "2026-01-15",
            "shift_id": "shift_001",
            "category": "ordinary",
            "hours": "8.0",
            "rate": "28.54",
            "amount": "228.32",
            "clause_ref": "14.2"
        }"#;

        let pay_line: PayLine = serde_json::from_str(json).unwrap();
        assert_eq!(pay_line.date, NaiveDate::from_ymd_opt(2026, 1, 15).unwrap());
        assert_eq!(pay_line.shift_id, "shift_001");
        assert_eq!(pay_line.category, PayCategory::Ordinary);
        assert_eq!(pay_line.hours, dec("8.0"));
        assert_eq!(pay_line.rate, dec("28.54"));
        assert_eq!(pay_line.amount, dec("228.32"));
    }

    #[test]
    fn test_allowance_payment_serialization() {
        let allowance = AllowancePayment {
            allowance_type: "laundry".to_string(),
            description: "Laundry allowance for uniform cleaning".to_string(),
            units: dec("5.0"),
            rate: dec("0.32"),
            amount: dec("1.49"),
            clause_ref: "20.2".to_string(),
        };

        let json = serde_json::to_string(&allowance).unwrap();
        assert!(json.contains("\"type\":\"laundry\""));
        assert!(json.contains("\"description\":\"Laundry allowance for uniform cleaning\""));
        assert!(json.contains("\"clause_ref\":\"20.2\""));
    }

    #[test]
    fn test_allowance_payment_deserialization() {
        let json = r#"{
            "type": "meal",
            "description": "Meal allowance for overtime",
            "units": "1.0",
            "rate": "15.00",
            "amount": "15.00",
            "clause_ref": "20.3"
        }"#;

        let allowance: AllowancePayment = serde_json::from_str(json).unwrap();
        assert_eq!(allowance.allowance_type, "meal");
        assert_eq!(allowance.description, "Meal allowance for overtime");
        assert_eq!(allowance.units, dec("1.0"));
        assert_eq!(allowance.rate, dec("15.00"));
        assert_eq!(allowance.amount, dec("15.00"));
        assert_eq!(allowance.clause_ref, "20.3");
    }

    #[test]
    fn test_pay_totals_serialization() {
        let totals = PayTotals {
            gross_pay: dec("1500.00"),
            ordinary_hours: dec("38.0"),
            overtime_hours: dec("4.0"),
            penalty_hours: dec("8.0"),
            allowances_total: dec("5.60"),
        };

        let json = serde_json::to_string(&totals).unwrap();
        assert!(json.contains("\"gross_pay\":\"1500.00\""));
        assert!(json.contains("\"ordinary_hours\":\"38.0\""));
        assert!(json.contains("\"overtime_hours\":\"4.0\""));
        assert!(json.contains("\"penalty_hours\":\"8.0\""));
        assert!(json.contains("\"allowances_total\":\"5.60\""));
    }

    #[test]
    fn test_pay_totals_deserialization() {
        let json = r#"{
            "gross_pay": "2000.50",
            "ordinary_hours": "40.0",
            "overtime_hours": "2.0",
            "penalty_hours": "0",
            "allowances_total": "10.00"
        }"#;

        let totals: PayTotals = serde_json::from_str(json).unwrap();
        assert_eq!(totals.gross_pay, dec("2000.50"));
        assert_eq!(totals.ordinary_hours, dec("40.0"));
        assert_eq!(totals.overtime_hours, dec("2.0"));
        assert_eq!(totals.penalty_hours, dec("0"));
        assert_eq!(totals.allowances_total, dec("10.00"));
    }

    #[test]
    fn test_audit_step_serialization() {
        let step = AuditStep {
            step_number: 1,
            rule_id: "rule_001".to_string(),
            rule_name: "Calculate ordinary hours".to_string(),
            clause_ref: "14.2".to_string(),
            input: serde_json::json!({"hours": 8.0}),
            output: serde_json::json!({"amount": 228.32}),
            reasoning: "Applied standard hourly rate for DCE Level 3".to_string(),
        };

        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("\"step_number\":1"));
        assert!(json.contains("\"rule_id\":\"rule_001\""));
        assert!(json.contains("\"rule_name\":\"Calculate ordinary hours\""));
    }

    #[test]
    fn test_audit_warning_serialization() {
        let warning = AuditWarning {
            code: "WARN_001".to_string(),
            message: "Shift exceeds 10 hours".to_string(),
            severity: "medium".to_string(),
        };

        let json = serde_json::to_string(&warning).unwrap();
        assert!(json.contains("\"code\":\"WARN_001\""));
        assert!(json.contains("\"message\":\"Shift exceeds 10 hours\""));
        assert!(json.contains("\"severity\":\"medium\""));
    }

    #[test]
    fn test_audit_trace_serialization() {
        let trace = AuditTrace {
            steps: vec![AuditStep {
                step_number: 1,
                rule_id: "rule_001".to_string(),
                rule_name: "Test rule".to_string(),
                clause_ref: "14.2".to_string(),
                input: serde_json::json!({}),
                output: serde_json::json!({}),
                reasoning: "Test reasoning".to_string(),
            }],
            warnings: vec![AuditWarning {
                code: "WARN_001".to_string(),
                message: "Test warning".to_string(),
                severity: "low".to_string(),
            }],
            duration_us: 1234,
        };

        let json = serde_json::to_string(&trace).unwrap();
        assert!(json.contains("\"duration_us\":1234"));
        assert!(json.contains("\"steps\":["));
        assert!(json.contains("\"warnings\":["));
    }

    #[test]
    fn test_calculation_result_serialization() {
        let result = CalculationResult {
            calculation_id: Uuid::nil(),
            timestamp: DateTime::parse_from_rfc3339("2026-01-15T10:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            engine_version: "1.0.0".to_string(),
            employee_id: "emp_001".to_string(),
            pay_period: create_sample_pay_period(),
            pay_lines: vec![create_sample_pay_line(dec("228.32"))],
            allowances: vec![create_sample_allowance(dec("1.49"))],
            totals: PayTotals {
                gross_pay: dec("229.81"),
                ordinary_hours: dec("8.0"),
                overtime_hours: dec("0"),
                penalty_hours: dec("0"),
                allowances_total: dec("1.49"),
            },
            audit_trace: create_sample_audit_trace(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"calculation_id\":\"00000000-0000-0000-0000-000000000000\""));
        assert!(json.contains("\"engine_version\":\"1.0.0\""));
        assert!(json.contains("\"employee_id\":\"emp_001\""));
        assert!(json.contains("\"pay_period\":{"));
        assert!(json.contains("\"pay_lines\":["));
        assert!(json.contains("\"allowances\":["));
        assert!(json.contains("\"totals\":{"));
        assert!(json.contains("\"audit_trace\":{"));
    }

    #[test]
    fn test_calculation_result_deserialization() {
        let json = r#"{
            "calculation_id": "12345678-1234-1234-1234-123456789012",
            "timestamp": "2026-01-15T10:00:00Z",
            "engine_version": "1.0.0",
            "employee_id": "emp_001",
            "pay_period": {
                "start_date": "2026-01-13",
                "end_date": "2026-01-26",
                "public_holidays": []
            },
            "pay_lines": [],
            "allowances": [],
            "totals": {
                "gross_pay": "0",
                "ordinary_hours": "0",
                "overtime_hours": "0",
                "penalty_hours": "0",
                "allowances_total": "0"
            },
            "audit_trace": {
                "steps": [],
                "warnings": [],
                "duration_us": 0
            }
        }"#;

        let result: CalculationResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.engine_version, "1.0.0");
        assert_eq!(result.employee_id, "emp_001");
        assert!(result.pay_lines.is_empty());
        assert!(result.allowances.is_empty());
    }

    #[test]
    fn test_all_pay_categories() {
        let categories = vec![
            PayCategory::Ordinary,
            PayCategory::OrdinaryCasual,
            PayCategory::Saturday,
            PayCategory::SaturdayCasual,
            PayCategory::Sunday,
            PayCategory::SundayCasual,
            PayCategory::Overtime150,
            PayCategory::Overtime200,
        ];

        for category in categories {
            let json = serde_json::to_string(&category).unwrap();
            let deserialized: PayCategory = serde_json::from_str(&json).unwrap();
            assert_eq!(category, deserialized);
        }
    }

    #[test]
    fn test_decimal_precision_in_pay_line() {
        let pay_line = PayLine {
            date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
            shift_id: "shift_001".to_string(),
            category: PayCategory::Ordinary,
            hours: dec("7.5"),
            rate: dec("28.54"),
            amount: dec("214.05"),
            clause_ref: "14.2".to_string(),
        };

        assert_eq!(pay_line.hours * pay_line.rate, dec("214.05"));
    }

    #[test]
    fn test_multiple_pay_lines_sum() {
        let pay_lines = vec![
            PayLine {
                date: NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(),
                shift_id: "shift_001".to_string(),
                category: PayCategory::Ordinary,
                hours: dec("8.0"),
                rate: dec("28.54"),
                amount: dec("228.32"),
                clause_ref: "14.2".to_string(),
            },
            PayLine {
                date: NaiveDate::from_ymd_opt(2026, 1, 16).unwrap(),
                shift_id: "shift_002".to_string(),
                category: PayCategory::Saturday,
                hours: dec("8.0"),
                rate: dec("42.81"),
                amount: dec("342.48"),
                clause_ref: "23.1".to_string(),
            },
            PayLine {
                date: NaiveDate::from_ymd_opt(2026, 1, 17).unwrap(),
                shift_id: "shift_003".to_string(),
                category: PayCategory::Sunday,
                hours: dec("4.0"),
                rate: dec("57.08"),
                amount: dec("228.32"),
                clause_ref: "23.2".to_string(),
            },
        ];

        let total: Decimal = pay_lines.iter().map(|pl| pl.amount).sum();
        assert_eq!(total, dec("799.12"));
    }

    #[test]
    fn test_audit_steps_ordered() {
        let trace = AuditTrace {
            steps: vec![
                AuditStep {
                    step_number: 1,
                    rule_id: "rule_001".to_string(),
                    rule_name: "First step".to_string(),
                    clause_ref: "14.2".to_string(),
                    input: serde_json::json!({}),
                    output: serde_json::json!({}),
                    reasoning: "First".to_string(),
                },
                AuditStep {
                    step_number: 2,
                    rule_id: "rule_002".to_string(),
                    rule_name: "Second step".to_string(),
                    clause_ref: "23.1".to_string(),
                    input: serde_json::json!({}),
                    output: serde_json::json!({}),
                    reasoning: "Second".to_string(),
                },
                AuditStep {
                    step_number: 3,
                    rule_id: "rule_003".to_string(),
                    rule_name: "Third step".to_string(),
                    clause_ref: "25.1".to_string(),
                    input: serde_json::json!({}),
                    output: serde_json::json!({}),
                    reasoning: "Third".to_string(),
                },
            ],
            warnings: vec![],
            duration_us: 1000,
        };

        // Verify steps can be iterated in order
        let step_numbers: Vec<u32> = trace.steps.iter().map(|s| s.step_number).collect();
        assert_eq!(step_numbers, vec![1, 2, 3]);
    }
}
