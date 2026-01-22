//! Overtime audit trail integration tests.
//!
//! This module provides integration tests verifying the complete audit trail
//! for overtime calculations as per US-4.4 acceptance criteria.

#[cfg(test)]
mod tests {
    use crate::calculation::{
        calculate_weekday_overtime, detect_daily_overtime, get_base_rate,
        DEFAULT_DAILY_OVERTIME_THRESHOLD,
    };
    use crate::config::ConfigLoader;
    use crate::models::{Employee, EmploymentType};
    use chrono::NaiveDate;
    use rust_decimal::Decimal;
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
        NaiveDate::from_ymd_opt(2026, 1, 15).unwrap() // Wednesday (weekday)
    }

    // ==========================================================================
    // AT-OT-001: 12h weekday shift audit trace
    // Verifies: Audit trace includes steps in order with correct rule_ids
    // ==========================================================================
    #[test]
    fn test_at_ot_001_12h_weekday_shift_audit_trace_order() {
        let config = ConfigLoader::load("config/ma000018")
            .unwrap()
            .config()
            .clone();
        let employee = create_test_employee(EmploymentType::FullTime);
        let date = test_date();
        let worked_hours = dec("12.0");
        let mut all_audit_steps = Vec::new();
        let mut step_number = 1;

        // Step 1: Base rate lookup
        let base_rate_result = get_base_rate(&employee, date, &config, step_number).unwrap();
        all_audit_steps.push(base_rate_result.audit_step.clone());
        let base_rate = base_rate_result.rate;
        step_number += 1;

        // Step 2: Daily overtime detection
        let overtime_detection =
            detect_daily_overtime(worked_hours, DEFAULT_DAILY_OVERTIME_THRESHOLD, step_number);
        all_audit_steps.push(overtime_detection.audit_step.clone());
        step_number += 1;

        // Note: For this test, we're focusing on overtime portion (after 8 hours)
        // In full calculation, there would also be ordinary_hours step

        // Steps 3-4: Weekday overtime tiers
        let overtime_result = calculate_weekday_overtime(
            overtime_detection.overtime_hours,
            base_rate,
            &employee,
            &config,
            date,
            "shift_001",
            step_number,
        );
        all_audit_steps.extend(overtime_result.audit_steps);

        // Verify we have the expected steps
        assert!(all_audit_steps.len() >= 4, "Expected at least 4 audit steps");

        // Verify step rule_ids and clause_refs as per PRD
        assert_eq!(all_audit_steps[0].rule_id, "base_rate_lookup");
        assert_eq!(all_audit_steps[0].clause_ref, "14.2");

        assert_eq!(all_audit_steps[1].rule_id, "daily_overtime_detection");
        assert_eq!(all_audit_steps[1].clause_ref, "22.1(c), 25.1");

        assert_eq!(all_audit_steps[2].rule_id, "overtime_tier_1");
        assert_eq!(all_audit_steps[2].clause_ref, "25.1(a)(i)(A)");

        assert_eq!(all_audit_steps[3].rule_id, "overtime_tier_2");
        assert_eq!(all_audit_steps[3].clause_ref, "25.1(a)(i)(A)");
    }

    // ==========================================================================
    // AT-OT-002: OT tier 1 step content
    // Verifies: Input contains hours and base_rate, output contains multiplier and amount
    // ==========================================================================
    #[test]
    fn test_at_ot_002_ot_tier_1_step_content() {
        let config = ConfigLoader::load("config/ma000018")
            .unwrap()
            .config()
            .clone();
        let employee = create_test_employee(EmploymentType::FullTime);
        let date = test_date();
        let base_rate = dec("28.54");
        let overtime_hours = dec("4.0"); // 2h at tier 1, 2h at tier 2

        let result = calculate_weekday_overtime(
            overtime_hours,
            base_rate,
            &employee,
            &config,
            date,
            "shift_001",
            1,
        );

        assert!(result.audit_steps.len() >= 1, "Expected at least 1 audit step");

        let tier1_step = &result.audit_steps[0];

        // Verify rule_id
        assert_eq!(tier1_step.rule_id, "overtime_tier_1");

        // Verify input contains hours and base_rate
        assert!(tier1_step.input.get("hours").is_some(), "Input should contain hours");
        assert!(
            tier1_step.input.get("base_rate").is_some(),
            "Input should contain base_rate"
        );
        assert_eq!(tier1_step.input["hours"].as_str().unwrap(), "2");
        assert_eq!(tier1_step.input["base_rate"].as_str().unwrap(), "28.54");

        // Verify output contains multiplier and amount
        assert!(
            tier1_step.output.get("multiplier").is_some(),
            "Output should contain multiplier"
        );
        assert!(
            tier1_step.output.get("amount").is_some(),
            "Output should contain amount"
        );
        assert_eq!(tier1_step.output["multiplier"].as_str().unwrap(), "1.5");
        assert_eq!(tier1_step.output["amount"].as_str().unwrap(), "85.62");

        // Verify reasoning explains calculation in plain English
        assert!(
            tier1_step.reasoning.contains("150%") || tier1_step.reasoning.contains("first"),
            "Reasoning should explain the tier 1 calculation"
        );
    }

    // ==========================================================================
    // AT-OT-003: OT tier 2 step content
    // Verifies: Input contains hours and base_rate, output contains multiplier and amount
    // ==========================================================================
    #[test]
    fn test_at_ot_003_ot_tier_2_step_content() {
        let config = ConfigLoader::load("config/ma000018")
            .unwrap()
            .config()
            .clone();
        let employee = create_test_employee(EmploymentType::FullTime);
        let date = test_date();
        let base_rate = dec("28.54");
        let overtime_hours = dec("4.0"); // 2h at tier 1, 2h at tier 2

        let result = calculate_weekday_overtime(
            overtime_hours,
            base_rate,
            &employee,
            &config,
            date,
            "shift_001",
            1,
        );

        assert!(result.audit_steps.len() >= 2, "Expected at least 2 audit steps");

        let tier2_step = &result.audit_steps[1];

        // Verify rule_id
        assert_eq!(tier2_step.rule_id, "overtime_tier_2");

        // Verify input contains hours and base_rate
        assert!(tier2_step.input.get("hours").is_some(), "Input should contain hours");
        assert!(
            tier2_step.input.get("base_rate").is_some(),
            "Input should contain base_rate"
        );
        assert_eq!(tier2_step.input["hours"].as_str().unwrap(), "2");
        assert_eq!(tier2_step.input["base_rate"].as_str().unwrap(), "28.54");

        // Verify output contains multiplier and amount
        assert!(
            tier2_step.output.get("multiplier").is_some(),
            "Output should contain multiplier"
        );
        assert!(
            tier2_step.output.get("amount").is_some(),
            "Output should contain amount"
        );
        assert_eq!(tier2_step.output["multiplier"].as_str().unwrap(), "2");
        assert_eq!(tier2_step.output["amount"].as_str().unwrap(), "114.16");

        // Verify reasoning explains calculation in plain English
        assert!(
            tier2_step.reasoning.contains("200%") || tier2_step.reasoning.contains("after"),
            "Reasoning should explain the tier 2 calculation"
        );
    }

    // ==========================================================================
    // Additional audit trail tests for completeness
    // ==========================================================================

    #[test]
    fn test_daily_overtime_detection_audit_step_format() {
        let worked_hours = dec("12.0");
        let threshold = dec("8.0");

        let result = detect_daily_overtime(worked_hours, threshold, 1);

        // Verify rule_id and clause_ref
        assert_eq!(result.audit_step.rule_id, "daily_overtime_detection");
        assert_eq!(result.audit_step.clause_ref, "22.1(c), 25.1");

        // Verify input contains worked_hours and threshold
        assert_eq!(
            result.audit_step.input["worked_hours"].as_str().unwrap(),
            "12"
        );
        assert_eq!(result.audit_step.input["threshold"].as_str().unwrap(), "8");

        // Verify output contains ordinary_hours and overtime_hours
        assert_eq!(
            result.audit_step.output["ordinary_hours"].as_str().unwrap(),
            "8"
        );
        assert_eq!(
            result.audit_step.output["overtime_hours"].as_str().unwrap(),
            "4"
        );

        // Verify reasoning explains the detection
        assert!(
            result.audit_step.reasoning.contains("exceeds")
                || result.audit_step.reasoning.contains("overtime"),
            "Reasoning should explain overtime detection"
        );
    }

    #[test]
    fn test_weekend_overtime_audit_step_format() {
        use crate::calculation::{calculate_weekend_overtime, DayType};

        let config = ConfigLoader::load("config/ma000018")
            .unwrap()
            .config()
            .clone();
        let employee = create_test_employee(EmploymentType::FullTime);
        let date = NaiveDate::from_ymd_opt(2026, 1, 17).unwrap(); // Saturday
        let base_rate = dec("28.54");
        let overtime_hours = dec("2.0");

        let result = calculate_weekend_overtime(
            overtime_hours,
            base_rate,
            &employee,
            &config,
            DayType::Saturday,
            date,
            "shift_001",
            1,
        );

        assert!(result.audit_step.is_some());
        let step = result.audit_step.unwrap();

        // Verify rule_id
        assert_eq!(step.rule_id, "weekend_overtime");

        // Verify clause_ref
        assert_eq!(step.clause_ref, "25.1(a)(i)(B)");

        // Verify input contains hours, base_rate, employment_type, day_type
        assert_eq!(step.input["hours"].as_str().unwrap(), "2");
        assert_eq!(step.input["base_rate"].as_str().unwrap(), "28.54");
        assert_eq!(step.input["employment_type"].as_str().unwrap(), "full_time");
        assert_eq!(step.input["day_type"].as_str().unwrap(), "Saturday");

        // Verify output contains multiplier, rate, amount
        assert_eq!(step.output["multiplier"].as_str().unwrap(), "2");
        assert_eq!(step.output["rate"].as_str().unwrap(), "57.08");
        assert_eq!(step.output["amount"].as_str().unwrap(), "114.16");

        // Verify reasoning explains calculation
        assert!(
            step.reasoning.contains("Saturday") && step.reasoning.contains("200%"),
            "Reasoning should explain weekend overtime calculation"
        );
    }

    #[test]
    fn test_casual_overtime_audit_mentions_loading() {
        let config = ConfigLoader::load("config/ma000018")
            .unwrap()
            .config()
            .clone();
        let employee = create_test_employee(EmploymentType::Casual);
        let date = test_date();
        let base_rate = dec("28.54");
        let overtime_hours = dec("3.0"); // 2h at tier 1, 1h at tier 2

        let result = calculate_weekday_overtime(
            overtime_hours,
            base_rate,
            &employee,
            &config,
            date,
            "shift_001",
            1,
        );

        // Both tier 1 and tier 2 should mention casual loading
        for step in &result.audit_steps {
            assert!(
                step.reasoning.contains("casual loading"),
                "Casual overtime reasoning should mention casual loading"
            );
        }
    }

    #[test]
    fn test_audit_step_numbers_are_sequential() {
        let config = ConfigLoader::load("config/ma000018")
            .unwrap()
            .config()
            .clone();
        let employee = create_test_employee(EmploymentType::FullTime);
        let date = test_date();
        let worked_hours = dec("12.0");
        let mut all_audit_steps = Vec::new();
        let mut step_number = 1;

        // Step 1: Base rate lookup
        let base_rate_result = get_base_rate(&employee, date, &config, step_number).unwrap();
        all_audit_steps.push(base_rate_result.audit_step.clone());
        let base_rate = base_rate_result.rate;
        step_number += 1;

        // Step 2: Daily overtime detection
        let overtime_detection =
            detect_daily_overtime(worked_hours, DEFAULT_DAILY_OVERTIME_THRESHOLD, step_number);
        all_audit_steps.push(overtime_detection.audit_step.clone());
        step_number += 1;

        // Steps 3-4: Weekday overtime tiers
        let overtime_result = calculate_weekday_overtime(
            overtime_detection.overtime_hours,
            base_rate,
            &employee,
            &config,
            date,
            "shift_001",
            step_number,
        );
        all_audit_steps.extend(overtime_result.audit_steps);

        // Verify step numbers are sequential
        for (i, step) in all_audit_steps.iter().enumerate() {
            assert_eq!(
                step.step_number,
                (i + 1) as u32,
                "Step number should be sequential"
            );
        }
    }

    #[test]
    fn test_base_rate_lookup_audit_clause_ref() {
        let config = ConfigLoader::load("config/ma000018")
            .unwrap()
            .config()
            .clone();
        let employee = create_test_employee(EmploymentType::FullTime);
        let date = test_date();

        let result = get_base_rate(&employee, date, &config, 1).unwrap();

        // Verify clause reference matches PRD requirement
        assert_eq!(result.audit_step.rule_id, "base_rate_lookup");
        assert_eq!(result.audit_step.clause_ref, "14.2");
    }

    #[test]
    fn test_complete_12h_weekday_audit_trace_rule_ids() {
        let config = ConfigLoader::load("config/ma000018")
            .unwrap()
            .config()
            .clone();
        let employee = create_test_employee(EmploymentType::FullTime);
        let date = test_date();
        let worked_hours = dec("12.0");
        let mut rule_ids = Vec::new();
        let mut step_number = 1;

        // Step 1: Base rate lookup
        let base_rate_result = get_base_rate(&employee, date, &config, step_number).unwrap();
        rule_ids.push(base_rate_result.audit_step.rule_id.clone());
        let base_rate = base_rate_result.rate;
        step_number += 1;

        // Step 2: Daily overtime detection
        let overtime_detection =
            detect_daily_overtime(worked_hours, DEFAULT_DAILY_OVERTIME_THRESHOLD, step_number);
        rule_ids.push(overtime_detection.audit_step.rule_id.clone());
        step_number += 1;

        // Steps 3-4: Weekday overtime tiers (for the 4 hours of overtime)
        let overtime_result = calculate_weekday_overtime(
            overtime_detection.overtime_hours,
            base_rate,
            &employee,
            &config,
            date,
            "shift_001",
            step_number,
        );
        for step in &overtime_result.audit_steps {
            rule_ids.push(step.rule_id.clone());
        }

        // The expected rule_ids from PRD AT-OT-001
        let expected_rule_ids = vec![
            "base_rate_lookup",
            "daily_overtime_detection",
            "overtime_tier_1",
            "overtime_tier_2",
        ];

        assert_eq!(
            rule_ids, expected_rule_ids,
            "Audit trace should have the expected rule_ids in order"
        );
    }
}
