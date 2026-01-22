//! Overnight shift calculation functionality.
//!
//! This module provides functions for calculating pay for shifts that span
//! midnight and cross into different days with different penalty rates.
//! Per the Aged Care Award 2010, overnight shifts must be segmented at
//! midnight boundaries with each segment receiving the appropriate rate.

use rust_decimal::Decimal;

use crate::config::AwardConfig;
use crate::error::EngineResult;
use crate::models::{AuditStep, Employee, EmploymentType, PayCategory, PayLine, Shift};

use super::base_rate::get_base_rate;
use super::casual_loading::apply_casual_loading;
use super::day_detection::{segment_by_day, DayType, ShiftSegment};
use super::saturday_penalty::calculate_saturday_pay;
use super::sunday_penalty::calculate_sunday_pay;

/// The result of an overnight shift calculation, including multiple pay lines and audit steps.
///
/// For shifts spanning multiple days, this result contains a pay line for each day segment
/// with the appropriate penalty rate applied.
#[derive(Debug, Clone)]
pub struct OvernightShiftResult {
    /// The pay lines for each segment of the overnight shift.
    pub pay_lines: Vec<PayLine>,
    /// The audit steps recording this calculation, including segmentation and per-segment calculations.
    pub audit_steps: Vec<AuditStep>,
    /// The total amount across all segments.
    pub total_amount: Decimal,
}

/// Calculates pay for a shift that may span midnight and cross day boundaries.
///
/// This function:
/// 1. Looks up the base rate for the employee
/// 2. Segments the shift at midnight boundaries using [`segment_by_day`]
/// 3. Applies the correct penalty rate to each segment based on day type:
///    - Weekday: ordinary time (with casual loading for casuals)
///    - Saturday: 150% for non-casuals, 175% for casuals (clause 23.1, 23.2(a))
///    - Sunday: 175% for non-casuals, 200% for casuals (clause 23.1, 23.2(b))
/// 4. Generates audit trail showing segmentation and per-segment calculations
///
/// # Arguments
///
/// * `shift` - The shift to calculate pay for
/// * `employee` - The employee who worked the shift
/// * `config` - The award configuration containing rates and penalties
/// * `start_step_number` - The starting step number for audit trail sequencing
///
/// # Returns
///
/// Returns an `OvernightShiftResult` containing pay lines for each segment and audit steps,
/// or an error if the base rate lookup fails.
///
/// # Award Reference
///
/// - Clause 22.1: Ordinary hours
/// - Clause 23.1: Weekend penalties for permanent employees
/// - Clause 23.2(a): Saturday penalty for casuals (175%)
/// - Clause 23.2(b): Sunday penalty for casuals (200%)
///
/// # Examples
///
/// ```no_run
/// use award_engine::calculation::calculate_overnight_shift;
/// use award_engine::config::ConfigLoader;
/// use award_engine::models::{Employee, EmploymentType, Shift};
/// use chrono::{NaiveDate, NaiveDateTime};
///
/// let loader = ConfigLoader::load("config/ma000018").unwrap();
/// let config = loader.config();
///
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
/// // Saturday 22:00 to Sunday 06:00 shift
/// let shift = Shift {
///     id: "shift_001".to_string(),
///     date: NaiveDate::from_ymd_opt(2026, 1, 17).unwrap(),
///     start_time: NaiveDateTime::parse_from_str("2026-01-17 22:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
///     end_time: NaiveDateTime::parse_from_str("2026-01-18 06:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
///     breaks: vec![],
/// };
///
/// let result = calculate_overnight_shift(&shift, &employee, config, 1).unwrap();
/// // Result contains two pay lines: one for Saturday hours, one for Sunday hours
/// assert_eq!(result.pay_lines.len(), 2);
/// ```
pub fn calculate_overnight_shift(
    shift: &Shift,
    employee: &Employee,
    config: &AwardConfig,
    start_step_number: u32,
) -> EngineResult<OvernightShiftResult> {
    let mut audit_steps = Vec::new();
    let mut current_step = start_step_number;

    // Step 1: Look up base rate
    let base_rate_result = get_base_rate(employee, shift.date, config, current_step)?;
    let base_rate = base_rate_result.rate;
    audit_steps.push(base_rate_result.audit_step);
    current_step += 1;

    // Step 2: Segment the shift by day boundaries
    let segments = segment_by_day(shift);

    // Create audit step for segmentation
    let segment_descriptions: Vec<serde_json::Value> = segments
        .iter()
        .map(|s| {
            serde_json::json!({
                "day_type": format!("{}", s.day_type),
                "hours": s.hours.normalize().to_string(),
                "start_time": s.start_time.to_string(),
                "end_time": s.end_time.to_string()
            })
        })
        .collect();

    let segmentation_step = AuditStep {
        step_number: current_step,
        rule_id: "shift_segmentation".to_string(),
        rule_name: "Shift Day Segmentation".to_string(),
        clause_ref: "23".to_string(),
        input: serde_json::json!({
            "shift_id": shift.id,
            "start_time": shift.start_time.to_string(),
            "end_time": shift.end_time.to_string(),
            "total_hours": shift.worked_hours().normalize().to_string()
        }),
        output: serde_json::json!({
            "segment_count": segments.len(),
            "segments": segment_descriptions
        }),
        reasoning: if segments.len() == 1 {
            format!(
                "Shift is entirely within {} - no midnight crossing",
                segments[0].day_type
            )
        } else {
            format!(
                "Shift crosses midnight: split into {} segments ({})",
                segments.len(),
                segments
                    .iter()
                    .map(|s| format!("{}: {}h", s.day_type, s.hours.normalize()))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        },
    };
    audit_steps.push(segmentation_step);
    current_step += 1;

    // Step 3: Calculate pay for each segment
    let mut pay_lines = Vec::new();
    let mut total_amount = Decimal::ZERO;

    for segment in &segments {
        let (mut pay_line, segment_audit) =
            calculate_segment_pay(segment, employee, base_rate, config, current_step)?;

        // Set the shift_id on the pay line
        pay_line.shift_id = shift.id.clone();

        total_amount += pay_line.amount;
        pay_lines.push(pay_line);
        audit_steps.push(segment_audit);
        current_step += 1;
    }

    // Step 4: Create summary audit step
    let summary_step = AuditStep {
        step_number: current_step,
        rule_id: "overnight_shift_total".to_string(),
        rule_name: "Overnight Shift Total Calculation".to_string(),
        clause_ref: "23".to_string(),
        input: serde_json::json!({
            "shift_id": shift.id,
            "segment_count": pay_lines.len(),
            "segment_amounts": pay_lines.iter().map(|p| p.amount.normalize().to_string()).collect::<Vec<_>>()
        }),
        output: serde_json::json!({
            "total_amount": total_amount.normalize().to_string(),
            "total_hours": shift.worked_hours().normalize().to_string()
        }),
        reasoning: format!(
            "Total overnight shift pay: {} segment(s) = ${}",
            pay_lines.len(),
            total_amount.normalize()
        ),
    };
    audit_steps.push(summary_step);

    Ok(OvernightShiftResult {
        pay_lines,
        audit_steps,
        total_amount,
    })
}

/// Calculates pay for a single shift segment based on its day type.
///
/// Returns the pay line and audit step for the segment.
fn calculate_segment_pay(
    segment: &ShiftSegment,
    employee: &Employee,
    base_rate: Decimal,
    config: &AwardConfig,
    step_number: u32,
) -> EngineResult<(PayLine, AuditStep)> {
    match segment.day_type {
        DayType::Saturday => {
            let result = calculate_saturday_pay(segment, employee, base_rate, config, step_number);
            Ok((result.pay_line, result.audit_step))
        }
        DayType::Sunday => {
            let result = calculate_sunday_pay(segment, employee, base_rate, config, step_number);
            Ok((result.pay_line, result.audit_step))
        }
        DayType::Weekday => {
            // For weekday segments, apply ordinary time with casual loading if applicable
            let casual_result = apply_casual_loading(base_rate, employee, step_number);
            let effective_rate = casual_result.loaded_rate;
            let amount = segment.hours * effective_rate;

            let (category, clause_ref) = match employee.employment_type {
                EmploymentType::Casual => (PayCategory::OrdinaryCasual, "10.4(b), 22.1"),
                EmploymentType::FullTime | EmploymentType::PartTime => {
                    (PayCategory::Ordinary, "22.1")
                }
            };

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
                clause_ref: clause_ref.to_string(),
            };

            let audit_step = AuditStep {
                step_number,
                rule_id: "weekday_ordinary".to_string(),
                rule_name: "Weekday Ordinary Time".to_string(),
                clause_ref: clause_ref.to_string(),
                input: serde_json::json!({
                    "hours": segment.hours.normalize().to_string(),
                    "base_rate": base_rate.normalize().to_string(),
                    "employment_type": employment_type_str,
                    "day_type": "Weekday"
                }),
                output: serde_json::json!({
                    "effective_rate": effective_rate.normalize().to_string(),
                    "amount": amount.normalize().to_string(),
                    "category": format!("{:?}", category)
                }),
                reasoning: format!(
                    "Weekday ordinary time: {} hours × ${} = ${}",
                    segment.hours.normalize(),
                    effective_rate.normalize(),
                    amount.normalize()
                ),
            };

            Ok((pay_line, audit_step))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn make_date(date_str: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap()
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

    fn load_config() -> AwardConfig {
        ConfigLoader::load("config/ma000018")
            .expect("Failed to load config")
            .config()
            .clone()
    }

    // ==========================================================================
    // OVN-001: fulltime Sat 22:00 to Sun 06:00
    // Expected: Saturday 2h × $28.54 × 1.50 = $85.62
    //           Sunday 6h × $28.54 × 1.75 = $299.67
    //           Total: $385.29
    // ==========================================================================
    #[test]
    fn test_ovn_001_fulltime_sat_to_sun() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);

        // 2026-01-17 is Saturday, 2026-01-18 is Sunday
        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-17"),
            start_time: make_datetime("2026-01-17", "22:00:00"),
            end_time: make_datetime("2026-01-18", "06:00:00"),
            breaks: vec![],
        };

        let result = calculate_overnight_shift(&shift, &employee, &config, 1).unwrap();

        assert_eq!(result.pay_lines.len(), 2);

        // Saturday segment: 2h × $28.54 × 1.50 = $85.62
        assert_eq!(result.pay_lines[0].category, PayCategory::Saturday);
        assert_eq!(result.pay_lines[0].hours, dec("2.0"));
        assert_eq!(result.pay_lines[0].amount, dec("85.62"));
        assert_eq!(result.pay_lines[0].clause_ref, "23.1");

        // Sunday segment: 6h × $28.54 × 1.75 = $299.67
        assert_eq!(result.pay_lines[1].category, PayCategory::Sunday);
        assert_eq!(result.pay_lines[1].hours, dec("6.0"));
        assert_eq!(result.pay_lines[1].amount, dec("299.67"));
        assert_eq!(result.pay_lines[1].clause_ref, "23.1");

        // Total: $385.29
        assert_eq!(result.total_amount, dec("385.29"));
    }

    // ==========================================================================
    // OVN-002: casual Sat 22:00 to Sun 06:00
    // Expected: Saturday 2h × $28.54 × 1.75 = $99.89
    //           Sunday 6h × $28.54 × 2.00 = $342.48
    //           Total: $442.37
    // ==========================================================================
    #[test]
    fn test_ovn_002_casual_sat_to_sun() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::Casual);

        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-17"),
            start_time: make_datetime("2026-01-17", "22:00:00"),
            end_time: make_datetime("2026-01-18", "06:00:00"),
            breaks: vec![],
        };

        let result = calculate_overnight_shift(&shift, &employee, &config, 1).unwrap();

        assert_eq!(result.pay_lines.len(), 2);

        // Saturday segment: 2h × $28.54 × 1.75 = $99.89
        assert_eq!(result.pay_lines[0].category, PayCategory::SaturdayCasual);
        assert_eq!(result.pay_lines[0].hours, dec("2.0"));
        assert_eq!(result.pay_lines[0].amount, dec("99.89"));
        assert_eq!(result.pay_lines[0].clause_ref, "23.2(a)");

        // Sunday segment: 6h × $28.54 × 2.00 = $342.48
        assert_eq!(result.pay_lines[1].category, PayCategory::SundayCasual);
        assert_eq!(result.pay_lines[1].hours, dec("6.0"));
        assert_eq!(result.pay_lines[1].amount, dec("342.48"));
        assert_eq!(result.pay_lines[1].clause_ref, "23.2(b)");

        // Total: $442.37
        assert_eq!(result.total_amount, dec("442.37"));
    }

    // ==========================================================================
    // OVN-003: fulltime Fri 22:00 to Sat 06:00
    // Expected: Friday 2h × $28.54 × 1.00 = $57.08
    //           Saturday 6h × $28.54 × 1.50 = $256.86
    //           Total: $313.94
    // ==========================================================================
    #[test]
    fn test_ovn_003_fulltime_fri_to_sat() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);

        // 2026-01-16 is Friday, 2026-01-17 is Saturday
        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-16"),
            start_time: make_datetime("2026-01-16", "22:00:00"),
            end_time: make_datetime("2026-01-17", "06:00:00"),
            breaks: vec![],
        };

        let result = calculate_overnight_shift(&shift, &employee, &config, 1).unwrap();

        assert_eq!(result.pay_lines.len(), 2);

        // Friday segment: 2h × $28.54 × 1.00 = $57.08
        assert_eq!(result.pay_lines[0].category, PayCategory::Ordinary);
        assert_eq!(result.pay_lines[0].hours, dec("2.0"));
        assert_eq!(result.pay_lines[0].amount, dec("57.08"));
        assert_eq!(result.pay_lines[0].clause_ref, "22.1");

        // Saturday segment: 6h × $28.54 × 1.50 = $256.86
        assert_eq!(result.pay_lines[1].category, PayCategory::Saturday);
        assert_eq!(result.pay_lines[1].hours, dec("6.0"));
        assert_eq!(result.pay_lines[1].amount, dec("256.86"));
        assert_eq!(result.pay_lines[1].clause_ref, "23.1");

        // Total: $313.94
        assert_eq!(result.total_amount, dec("313.94"));
    }

    // ==========================================================================
    // OVN-004: casual Fri 22:00 to Sat 06:00
    // Expected: Friday 2h × $28.54 × 1.25 = $71.35
    //           Saturday 6h × $28.54 × 1.75 = $299.67
    //           Total: $371.02
    // ==========================================================================
    #[test]
    fn test_ovn_004_casual_fri_to_sat() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::Casual);

        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-16"),
            start_time: make_datetime("2026-01-16", "22:00:00"),
            end_time: make_datetime("2026-01-17", "06:00:00"),
            breaks: vec![],
        };

        let result = calculate_overnight_shift(&shift, &employee, &config, 1).unwrap();

        assert_eq!(result.pay_lines.len(), 2);

        // Friday segment: 2h × $28.54 × 1.25 = $71.35
        assert_eq!(result.pay_lines[0].category, PayCategory::OrdinaryCasual);
        assert_eq!(result.pay_lines[0].hours, dec("2.0"));
        assert_eq!(result.pay_lines[0].amount, dec("71.35"));

        // Saturday segment: 6h × $28.54 × 1.75 = $299.67
        assert_eq!(result.pay_lines[1].category, PayCategory::SaturdayCasual);
        assert_eq!(result.pay_lines[1].hours, dec("6.0"));
        assert_eq!(result.pay_lines[1].amount, dec("299.67"));

        // Total: $371.02
        assert_eq!(result.total_amount, dec("371.02"));
    }

    // ==========================================================================
    // OVN-005: fulltime Sun 22:00 to Mon 06:00
    // Expected: Sunday 2h × $28.54 × 1.75 = $99.89
    //           Monday 6h × $28.54 × 1.00 = $171.24
    //           Total: $271.13
    // ==========================================================================
    #[test]
    fn test_ovn_005_fulltime_sun_to_mon() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);

        // 2026-01-18 is Sunday, 2026-01-19 is Monday
        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-18"),
            start_time: make_datetime("2026-01-18", "22:00:00"),
            end_time: make_datetime("2026-01-19", "06:00:00"),
            breaks: vec![],
        };

        let result = calculate_overnight_shift(&shift, &employee, &config, 1).unwrap();

        assert_eq!(result.pay_lines.len(), 2);

        // Sunday segment: 2h × $28.54 × 1.75 = $99.89
        assert_eq!(result.pay_lines[0].category, PayCategory::Sunday);
        assert_eq!(result.pay_lines[0].hours, dec("2.0"));
        assert_eq!(result.pay_lines[0].amount, dec("99.89"));
        assert_eq!(result.pay_lines[0].clause_ref, "23.1");

        // Monday segment: 6h × $28.54 × 1.00 = $171.24
        assert_eq!(result.pay_lines[1].category, PayCategory::Ordinary);
        assert_eq!(result.pay_lines[1].hours, dec("6.0"));
        assert_eq!(result.pay_lines[1].amount, dec("171.24"));
        assert_eq!(result.pay_lines[1].clause_ref, "22.1");

        // Total: $271.13
        assert_eq!(result.total_amount, dec("271.13"));
    }

    // ==========================================================================
    // Test audit trail shows shift segmentation step
    // ==========================================================================
    #[test]
    fn test_audit_trace_shows_shift_segmentation() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);

        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-17"),
            start_time: make_datetime("2026-01-17", "22:00:00"),
            end_time: make_datetime("2026-01-18", "06:00:00"),
            breaks: vec![],
        };

        let result = calculate_overnight_shift(&shift, &employee, &config, 1).unwrap();

        // Find the segmentation step
        let segmentation_step = result
            .audit_steps
            .iter()
            .find(|s| s.rule_id == "shift_segmentation")
            .expect("Should have shift segmentation step");

        assert_eq!(segmentation_step.rule_name, "Shift Day Segmentation");
        assert!(segmentation_step
            .reasoning
            .contains("crosses midnight"));
        assert!(segmentation_step.reasoning.contains("2 segments"));
    }

    // ==========================================================================
    // Test audit trace shows separate penalty calculations per segment
    // ==========================================================================
    #[test]
    fn test_audit_trace_shows_per_segment_calculations() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);

        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-17"),
            start_time: make_datetime("2026-01-17", "22:00:00"),
            end_time: make_datetime("2026-01-18", "06:00:00"),
            breaks: vec![],
        };

        let result = calculate_overnight_shift(&shift, &employee, &config, 1).unwrap();

        // Should have: base rate lookup, segmentation, saturday calc, sunday calc, total
        assert!(result.audit_steps.len() >= 4);

        // Find Saturday and Sunday penalty steps
        let saturday_step = result
            .audit_steps
            .iter()
            .find(|s| s.rule_id == "saturday_penalty");
        let sunday_step = result
            .audit_steps
            .iter()
            .find(|s| s.rule_id == "sunday_penalty");

        assert!(saturday_step.is_some(), "Should have Saturday penalty step");
        assert!(sunday_step.is_some(), "Should have Sunday penalty step");
    }

    // ==========================================================================
    // Test single-day shift (no overnight)
    // ==========================================================================
    #[test]
    fn test_single_day_saturday_shift() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);

        // Saturday 09:00 to 17:00 (no midnight crossing)
        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-17"),
            start_time: make_datetime("2026-01-17", "09:00:00"),
            end_time: make_datetime("2026-01-17", "17:00:00"),
            breaks: vec![],
        };

        let result = calculate_overnight_shift(&shift, &employee, &config, 1).unwrap();

        assert_eq!(result.pay_lines.len(), 1);
        assert_eq!(result.pay_lines[0].category, PayCategory::Saturday);
        assert_eq!(result.pay_lines[0].hours, dec("8.0"));
        // 8h × $28.54 × 1.50 = $342.48
        assert_eq!(result.pay_lines[0].amount, dec("342.48"));
        assert_eq!(result.total_amount, dec("342.48"));
    }

    // ==========================================================================
    // Test single-day weekday shift
    // ==========================================================================
    #[test]
    fn test_single_day_weekday_shift() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);

        // Wednesday 09:00 to 17:00
        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-14"),
            start_time: make_datetime("2026-01-14", "09:00:00"),
            end_time: make_datetime("2026-01-14", "17:00:00"),
            breaks: vec![],
        };

        let result = calculate_overnight_shift(&shift, &employee, &config, 1).unwrap();

        assert_eq!(result.pay_lines.len(), 1);
        assert_eq!(result.pay_lines[0].category, PayCategory::Ordinary);
        assert_eq!(result.pay_lines[0].hours, dec("8.0"));
        // 8h × $28.54 = $228.32
        assert_eq!(result.pay_lines[0].amount, dec("228.32"));
    }

    // ==========================================================================
    // Test shift_id is correctly set on all pay lines
    // ==========================================================================
    #[test]
    fn test_shift_id_set_on_all_pay_lines() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);

        let shift = Shift {
            id: "test_shift_123".to_string(),
            date: make_date("2026-01-17"),
            start_time: make_datetime("2026-01-17", "22:00:00"),
            end_time: make_datetime("2026-01-18", "06:00:00"),
            breaks: vec![],
        };

        let result = calculate_overnight_shift(&shift, &employee, &config, 1).unwrap();

        for pay_line in &result.pay_lines {
            assert_eq!(pay_line.shift_id, "test_shift_123");
        }
    }

    // ==========================================================================
    // Test total hours equals sum of segment hours
    // ==========================================================================
    #[test]
    fn test_total_hours_equals_sum_of_segments() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::FullTime);

        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-17"),
            start_time: make_datetime("2026-01-17", "22:00:00"),
            end_time: make_datetime("2026-01-18", "06:00:00"),
            breaks: vec![],
        };

        let result = calculate_overnight_shift(&shift, &employee, &config, 1).unwrap();

        let total_hours: Decimal = result.pay_lines.iter().map(|p| p.hours).sum();
        assert_eq!(total_hours, shift.worked_hours());
        assert_eq!(total_hours, dec("8.0"));
    }

    // ==========================================================================
    // Test part-time employee gets same rates as full-time
    // ==========================================================================
    #[test]
    fn test_parttime_same_rates_as_fulltime() {
        let config = load_config();
        let employee = create_test_employee(EmploymentType::PartTime);

        let shift = Shift {
            id: "shift_001".to_string(),
            date: make_date("2026-01-17"),
            start_time: make_datetime("2026-01-17", "22:00:00"),
            end_time: make_datetime("2026-01-18", "06:00:00"),
            breaks: vec![],
        };

        let result = calculate_overnight_shift(&shift, &employee, &config, 1).unwrap();

        // Same as full-time: $385.29 total
        assert_eq!(result.total_amount, dec("385.29"));
        assert_eq!(result.pay_lines[0].category, PayCategory::Saturday);
        assert_eq!(result.pay_lines[1].category, PayCategory::Sunday);
    }
}
