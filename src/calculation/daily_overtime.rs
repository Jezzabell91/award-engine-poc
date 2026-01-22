//! Daily overtime detection functionality.
//!
//! This module provides functions for detecting when a shift exceeds the daily
//! overtime threshold and splitting hours into ordinary and overtime portions
//! as per the Aged Care Award 2010.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::models::AuditStep;

/// The result of detecting daily overtime for a shift or segment.
///
/// Contains the split between ordinary hours and overtime hours,
/// along with the audit step documenting the detection.
///
/// # Example
///
/// ```
/// use award_engine::calculation::DailyOvertimeDetection;
/// use award_engine::models::AuditStep;
/// use rust_decimal::Decimal;
/// use std::str::FromStr;
///
/// let detection = DailyOvertimeDetection {
///     ordinary_hours: Decimal::from_str("8.0").unwrap(),
///     overtime_hours: Decimal::from_str("2.0").unwrap(),
///     audit_step: AuditStep {
///         step_number: 1,
///         rule_id: "daily_overtime_detection".to_string(),
///         rule_name: "Daily Overtime Detection".to_string(),
///         clause_ref: "22.1(c), 25.1".to_string(),
///         input: serde_json::json!({"worked_hours": "10.0", "threshold": "8.0"}),
///         output: serde_json::json!({"ordinary_hours": "8.0", "overtime_hours": "2.0"}),
///         reasoning: "10.0 hours worked exceeds 8.0 hour threshold".to_string(),
///     },
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyOvertimeDetection {
    /// The number of ordinary hours (up to the threshold).
    pub ordinary_hours: Decimal,
    /// The number of overtime hours (hours exceeding the threshold).
    pub overtime_hours: Decimal,
    /// The audit step recording this detection.
    pub audit_step: AuditStep,
}

/// Default daily overtime threshold in hours.
///
/// Per Aged Care Award 2010 clause 22.1(c), ordinary hours are up to 8 hours per day.
pub const DEFAULT_DAILY_OVERTIME_THRESHOLD: Decimal = Decimal::from_parts(8, 0, 0, false, 0);

/// Detects whether hours worked exceed the daily overtime threshold.
///
/// Splits the worked hours into ordinary hours (up to the threshold) and
/// overtime hours (any hours exceeding the threshold).
///
/// # Arguments
///
/// * `worked_hours` - The total hours worked in the shift or segment
/// * `threshold` - The overtime threshold (typically 8 hours per day)
/// * `step_number` - The step number for audit trail sequencing
///
/// # Returns
///
/// A [`DailyOvertimeDetection`] containing:
/// - `ordinary_hours`: Hours up to the threshold (capped at threshold)
/// - `overtime_hours`: Hours exceeding the threshold (can be zero)
/// - `audit_step`: Documentation of the detection with clause references
///
/// # Award Reference
///
/// - Clause 22.1(c): Defines ordinary hours as up to 8 hours per day
/// - Clause 25.1: Defines overtime as hours in excess of ordinary hours
///
/// # Examples
///
/// ## Shift at threshold (no overtime)
///
/// ```
/// use award_engine::calculation::{detect_daily_overtime, DEFAULT_DAILY_OVERTIME_THRESHOLD};
/// use rust_decimal::Decimal;
/// use std::str::FromStr;
///
/// let worked = Decimal::from_str("8.0").unwrap();
/// let result = detect_daily_overtime(worked, DEFAULT_DAILY_OVERTIME_THRESHOLD, 1);
///
/// assert_eq!(result.ordinary_hours, Decimal::from_str("8.0").unwrap());
/// assert_eq!(result.overtime_hours, Decimal::ZERO);
/// ```
///
/// ## Shift exceeding threshold
///
/// ```
/// use award_engine::calculation::{detect_daily_overtime, DEFAULT_DAILY_OVERTIME_THRESHOLD};
/// use rust_decimal::Decimal;
/// use std::str::FromStr;
///
/// let worked = Decimal::from_str("10.0").unwrap();
/// let result = detect_daily_overtime(worked, DEFAULT_DAILY_OVERTIME_THRESHOLD, 1);
///
/// assert_eq!(result.ordinary_hours, Decimal::from_str("8.0").unwrap());
/// assert_eq!(result.overtime_hours, Decimal::from_str("2.0").unwrap());
/// ```
///
/// ## Short shift (under threshold)
///
/// ```
/// use award_engine::calculation::{detect_daily_overtime, DEFAULT_DAILY_OVERTIME_THRESHOLD};
/// use rust_decimal::Decimal;
/// use std::str::FromStr;
///
/// let worked = Decimal::from_str("6.0").unwrap();
/// let result = detect_daily_overtime(worked, DEFAULT_DAILY_OVERTIME_THRESHOLD, 1);
///
/// assert_eq!(result.ordinary_hours, Decimal::from_str("6.0").unwrap());
/// assert_eq!(result.overtime_hours, Decimal::ZERO);
/// ```
pub fn detect_daily_overtime(
    worked_hours: Decimal,
    threshold: Decimal,
    step_number: u32,
) -> DailyOvertimeDetection {
    // Calculate ordinary hours (capped at threshold)
    let ordinary_hours = if worked_hours <= threshold {
        worked_hours
    } else {
        threshold
    };

    // Calculate overtime hours (excess over threshold)
    let overtime_hours = if worked_hours > threshold {
        worked_hours - threshold
    } else {
        Decimal::ZERO
    };

    // Determine reasoning based on outcome
    let reasoning = if overtime_hours > Decimal::ZERO {
        format!(
            "{} hours worked exceeds {} hour threshold by {} hours, triggering overtime",
            worked_hours.normalize(),
            threshold.normalize(),
            overtime_hours.normalize()
        )
    } else if worked_hours == threshold {
        format!(
            "{} hours worked equals {} hour threshold, no overtime triggered",
            worked_hours.normalize(),
            threshold.normalize()
        )
    } else {
        format!(
            "{} hours worked is under {} hour threshold, no overtime triggered",
            worked_hours.normalize(),
            threshold.normalize()
        )
    };

    let audit_step = AuditStep {
        step_number,
        rule_id: "daily_overtime_detection".to_string(),
        rule_name: "Daily Overtime Detection".to_string(),
        clause_ref: "22.1(c), 25.1".to_string(),
        input: serde_json::json!({
            "worked_hours": worked_hours.normalize().to_string(),
            "threshold": threshold.normalize().to_string()
        }),
        output: serde_json::json!({
            "ordinary_hours": ordinary_hours.normalize().to_string(),
            "overtime_hours": overtime_hours.normalize().to_string()
        }),
        reasoning,
    };

    DailyOvertimeDetection {
        ordinary_hours,
        overtime_hours,
        audit_step,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    // ==========================================================================
    // DOD-001: exactly 8 hours - no overtime
    // ==========================================================================
    #[test]
    fn test_dod_001_exactly_8_hours_no_overtime() {
        let worked_hours = dec("8.0");
        let threshold = dec("8.0");

        let result = detect_daily_overtime(worked_hours, threshold, 1);

        assert_eq!(result.ordinary_hours, dec("8.0"));
        assert_eq!(result.overtime_hours, dec("0.0"));

        // Verify audit step
        assert_eq!(result.audit_step.step_number, 1);
        assert_eq!(result.audit_step.rule_id, "daily_overtime_detection");
        assert_eq!(result.audit_step.clause_ref, "22.1(c), 25.1");
        assert_eq!(
            result.audit_step.input["worked_hours"].as_str().unwrap(),
            "8"
        );
        assert_eq!(result.audit_step.input["threshold"].as_str().unwrap(), "8");
        assert_eq!(
            result.audit_step.output["ordinary_hours"].as_str().unwrap(),
            "8"
        );
        assert_eq!(
            result.audit_step.output["overtime_hours"].as_str().unwrap(),
            "0"
        );
    }

    // ==========================================================================
    // DOD-002: 10 hours - 2 hours overtime
    // ==========================================================================
    #[test]
    fn test_dod_002_10_hours_2_hours_overtime() {
        let worked_hours = dec("10.0");
        let threshold = dec("8.0");

        let result = detect_daily_overtime(worked_hours, threshold, 1);

        assert_eq!(result.ordinary_hours, dec("8.0"));
        assert_eq!(result.overtime_hours, dec("2.0"));

        // Verify audit step output
        assert_eq!(
            result.audit_step.output["ordinary_hours"].as_str().unwrap(),
            "8"
        );
        assert_eq!(
            result.audit_step.output["overtime_hours"].as_str().unwrap(),
            "2"
        );
    }

    // ==========================================================================
    // DOD-003: 12 hours - 4 hours overtime
    // ==========================================================================
    #[test]
    fn test_dod_003_12_hours_4_hours_overtime() {
        let worked_hours = dec("12.0");
        let threshold = dec("8.0");

        let result = detect_daily_overtime(worked_hours, threshold, 1);

        assert_eq!(result.ordinary_hours, dec("8.0"));
        assert_eq!(result.overtime_hours, dec("4.0"));
    }

    // ==========================================================================
    // DOD-004: 6 hours - no overtime
    // ==========================================================================
    #[test]
    fn test_dod_004_6_hours_no_overtime() {
        let worked_hours = dec("6.0");
        let threshold = dec("8.0");

        let result = detect_daily_overtime(worked_hours, threshold, 1);

        assert_eq!(result.ordinary_hours, dec("6.0"));
        assert_eq!(result.overtime_hours, dec("0.0"));
    }

    // ==========================================================================
    // DOD-005: 8.5 hours - 0.5 hours overtime
    // ==========================================================================
    #[test]
    fn test_dod_005_8_5_hours_0_5_hours_overtime() {
        let worked_hours = dec("8.5");
        let threshold = dec("8.0");

        let result = detect_daily_overtime(worked_hours, threshold, 1);

        assert_eq!(result.ordinary_hours, dec("8.0"));
        assert_eq!(result.overtime_hours, dec("0.5"));
    }

    // ==========================================================================
    // DOD-006: 11.25 hours - 3.25 hours overtime
    // ==========================================================================
    #[test]
    fn test_dod_006_11_25_hours_3_25_hours_overtime() {
        let worked_hours = dec("11.25");
        let threshold = dec("8.0");

        let result = detect_daily_overtime(worked_hours, threshold, 1);

        assert_eq!(result.ordinary_hours, dec("8.0"));
        assert_eq!(result.overtime_hours, dec("3.25"));
    }

    // ==========================================================================
    // Additional tests for audit trail completeness
    // ==========================================================================

    #[test]
    fn test_audit_step_rule_name() {
        let result = detect_daily_overtime(dec("10.0"), dec("8.0"), 1);
        assert_eq!(result.audit_step.rule_name, "Daily Overtime Detection");
    }

    #[test]
    fn test_audit_step_reasoning_for_overtime() {
        let result = detect_daily_overtime(dec("10.0"), dec("8.0"), 1);
        assert!(result.audit_step.reasoning.contains("exceeds"));
        assert!(result.audit_step.reasoning.contains("overtime"));
    }

    #[test]
    fn test_audit_step_reasoning_for_no_overtime() {
        let result = detect_daily_overtime(dec("6.0"), dec("8.0"), 1);
        assert!(result.audit_step.reasoning.contains("under"));
        assert!(result.audit_step.reasoning.contains("no overtime"));
    }

    #[test]
    fn test_audit_step_reasoning_for_exact_threshold() {
        let result = detect_daily_overtime(dec("8.0"), dec("8.0"), 1);
        assert!(result.audit_step.reasoning.contains("equals"));
        assert!(result.audit_step.reasoning.contains("no overtime"));
    }

    #[test]
    fn test_step_number_passed_through() {
        let result = detect_daily_overtime(dec("10.0"), dec("8.0"), 5);
        assert_eq!(result.audit_step.step_number, 5);
    }

    #[test]
    fn test_custom_threshold() {
        // Test with a custom threshold (e.g., 10 hours)
        let worked_hours = dec("12.0");
        let threshold = dec("10.0");

        let result = detect_daily_overtime(worked_hours, threshold, 1);

        assert_eq!(result.ordinary_hours, dec("10.0"));
        assert_eq!(result.overtime_hours, dec("2.0"));
        assert_eq!(result.audit_step.input["threshold"].as_str().unwrap(), "10");
    }

    #[test]
    fn test_zero_hours_worked() {
        let result = detect_daily_overtime(dec("0.0"), dec("8.0"), 1);

        assert_eq!(result.ordinary_hours, dec("0.0"));
        assert_eq!(result.overtime_hours, dec("0.0"));
    }

    #[test]
    fn test_fractional_threshold() {
        // Test with fractional threshold
        let worked_hours = dec("8.5");
        let threshold = dec("7.5");

        let result = detect_daily_overtime(worked_hours, threshold, 1);

        assert_eq!(result.ordinary_hours, dec("7.5"));
        assert_eq!(result.overtime_hours, dec("1.0"));
    }

    #[test]
    fn test_default_threshold_constant() {
        assert_eq!(DEFAULT_DAILY_OVERTIME_THRESHOLD, dec("8"));
    }

    #[test]
    fn test_detection_with_default_threshold() {
        let worked_hours = dec("10.0");
        let result = detect_daily_overtime(worked_hours, DEFAULT_DAILY_OVERTIME_THRESHOLD, 1);

        assert_eq!(result.ordinary_hours, dec("8.0"));
        assert_eq!(result.overtime_hours, dec("2.0"));
    }

    #[test]
    fn test_serialization() {
        let result = detect_daily_overtime(dec("10.0"), dec("8.0"), 1);

        // Verify the result can be serialized
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"ordinary_hours\":\"8\""));
        assert!(json.contains("\"overtime_hours\":\"2\""));

        // Verify deserialization
        let deserialized: DailyOvertimeDetection = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.ordinary_hours, dec("8.0"));
        assert_eq!(deserialized.overtime_hours, dec("2.0"));
    }
}
