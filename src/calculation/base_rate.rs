//! Base rate lookup functionality.
//!
//! This module provides functions for determining an employee's base hourly rate,
//! either from their employee override or from the award configuration.

use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::config::AwardConfig;
use crate::error::{EngineError, EngineResult};
use crate::models::{AuditStep, Employee};

/// The result of a base rate lookup, including the rate and audit step.
#[derive(Debug, Clone)]
pub struct BaseRateLookupResult {
    /// The determined base hourly rate.
    pub rate: Decimal,
    /// The audit step recording this lookup.
    pub audit_step: AuditStep,
}

/// Determines an employee's base hourly rate.
///
/// This function looks up the base rate for an employee based on the following priority:
/// 1. If `employee.base_hourly_rate` is `Some`, use that override value
/// 2. Otherwise, look up the rate from the config by classification code and effective date
///
/// # Arguments
///
/// * `employee` - The employee to look up the rate for
/// * `effective_date` - The date for which to find the applicable rate
/// * `config` - The award configuration containing classification rates
///
/// # Returns
///
/// Returns a `BaseRateLookupResult` containing the rate and an audit step, or an error if:
/// - The classification code is not found in the config (`ClassificationNotFound`)
/// - No rate exists for the classification on the effective date (`RateNotFound`)
///
/// # Award Reference
///
/// Clause 14.2 of the Aged Care Award 2010 defines classification rates.
///
/// # Examples
///
/// ```
/// use award_engine::calculation::get_base_rate;
/// use award_engine::models::Employee;
/// use chrono::NaiveDate;
/// ```
pub fn get_base_rate(
    employee: &Employee,
    effective_date: NaiveDate,
    config: &AwardConfig,
    step_number: u32,
) -> EngineResult<BaseRateLookupResult> {
    // Check if employee has an override rate
    if let Some(override_rate) = employee.base_hourly_rate {
        let audit_step = AuditStep {
            step_number,
            rule_id: "base_rate_lookup".to_string(),
            rule_name: "Base Rate Lookup".to_string(),
            clause_ref: "14.2".to_string(),
            input: serde_json::json!({
                "classification_code": employee.classification_code,
                "employee_override_rate": override_rate.to_string(),
                "effective_date": effective_date.to_string()
            }),
            output: serde_json::json!({
                "rate": override_rate.to_string(),
                "source": "employee_override"
            }),
            reasoning: format!(
                "Using employee override rate ${} instead of classification lookup",
                override_rate
            ),
        };

        return Ok(BaseRateLookupResult {
            rate: override_rate,
            audit_step,
        });
    }

    // Check if classification exists in config
    if !config
        .classifications()
        .contains_key(&employee.classification_code)
    {
        return Err(EngineError::ClassificationNotFound {
            code: employee.classification_code.clone(),
        });
    }

    // Find the applicable rate for the effective date
    // Rates are sorted by effective_date ascending, so we find the most recent
    // rate that is on or before the effective_date (searching from the end)
    let applicable_rate = config
        .rates()
        .iter()
        .rfind(|r| r.effective_date <= effective_date);

    match applicable_rate {
        Some(rate_config) => {
            // Check if the classification has a rate in this rate config
            match rate_config.rates.get(&employee.classification_code) {
                Some(classification_rate) => {
                    let rate = classification_rate.hourly;
                    let audit_step = AuditStep {
                        step_number,
                        rule_id: "base_rate_lookup".to_string(),
                        rule_name: "Base Rate Lookup".to_string(),
                        clause_ref: "14.2".to_string(),
                        input: serde_json::json!({
                            "classification_code": employee.classification_code,
                            "effective_date": effective_date.to_string()
                        }),
                        output: serde_json::json!({
                            "rate": rate.to_string(),
                            "source": "config",
                            "rate_effective_date": rate_config.effective_date.to_string()
                        }),
                        reasoning: format!(
                            "Looked up rate for classification '{}' effective {}: ${}",
                            employee.classification_code, rate_config.effective_date, rate
                        ),
                    };

                    Ok(BaseRateLookupResult { rate, audit_step })
                }
                None => Err(EngineError::RateNotFound {
                    classification: employee.classification_code.clone(),
                    date: effective_date,
                }),
            }
        }
        None => Err(EngineError::RateNotFound {
            classification: employee.classification_code.clone(),
            date: effective_date,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        AllowanceRates, AwardMetadata, Classification, ClassificationRate, OvertimeConfig,
        OvertimeRates, OvertimeSection, Penalties, PenaltyConfig, PenaltyRates, RateConfig,
        WeekendOvertimeConfig,
    };
    use crate::models::EmploymentType;
    use std::collections::HashMap;
    use std::str::FromStr;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn create_test_config() -> AwardConfig {
        let metadata = AwardMetadata {
            code: "MA000018".to_string(),
            name: "Aged Care Award 2010".to_string(),
            version: "2025-07-01".to_string(),
            source_url: "https://example.com".to_string(),
        };

        let mut classifications = HashMap::new();
        classifications.insert(
            "dce_level_3".to_string(),
            Classification {
                name: "Direct Care Employee Level 3 - Qualified".to_string(),
                description: "Qualified direct care worker".to_string(),
                clause: "14.2".to_string(),
            },
        );

        let mut rates_map = HashMap::new();
        rates_map.insert(
            "dce_level_3".to_string(),
            ClassificationRate {
                weekly: dec("1084.70"),
                hourly: dec("28.54"),
            },
        );

        let rates = vec![RateConfig {
            effective_date: NaiveDate::from_ymd_opt(2025, 7, 1).unwrap(),
            rates: rates_map,
            allowances: AllowanceRates {
                laundry_per_shift: dec("0.32"),
                laundry_per_week: dec("1.49"),
            },
        }];

        let penalties = PenaltyConfig {
            penalties: Penalties {
                saturday: PenaltyRates {
                    clause: "23.1".to_string(),
                    full_time: dec("1.5"),
                    part_time: dec("1.5"),
                    casual: dec("1.75"),
                },
                sunday: PenaltyRates {
                    clause: "23.2".to_string(),
                    full_time: dec("2.0"),
                    part_time: dec("2.0"),
                    casual: dec("2.25"),
                },
            },
            overtime: OvertimeSection {
                daily_threshold_hours: 8,
                weekday: OvertimeConfig {
                    clause: "25.1".to_string(),
                    first_two_hours: OvertimeRates {
                        full_time: dec("1.5"),
                        part_time: dec("1.5"),
                        casual: dec("1.75"),
                    },
                    after_two_hours: OvertimeRates {
                        full_time: dec("2.0"),
                        part_time: dec("2.0"),
                        casual: dec("2.25"),
                    },
                },
                weekend: WeekendOvertimeConfig {
                    clause: "25.1(a)(i)(B)".to_string(),
                    saturday: OvertimeRates {
                        full_time: dec("2.0"),
                        part_time: dec("2.0"),
                        casual: dec("2.5"),
                    },
                    sunday: OvertimeRates {
                        full_time: dec("2.0"),
                        part_time: dec("2.0"),
                        casual: dec("2.5"),
                    },
                },
            },
        };

        AwardConfig::new(metadata, classifications, rates, penalties)
    }

    fn create_test_employee(classification: &str, override_rate: Option<Decimal>) -> Employee {
        Employee {
            id: "emp_001".to_string(),
            employment_type: EmploymentType::FullTime,
            classification_code: classification.to_string(),
            date_of_birth: NaiveDate::from_ymd_opt(1990, 1, 15).unwrap(),
            employment_start_date: NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
            base_hourly_rate: override_rate,
            tags: vec![],
        }
    }

    /// BR-001: config rate for dce_level_3
    #[test]
    fn test_config_rate_for_dce_level_3() {
        let config = create_test_config();
        let employee = create_test_employee("dce_level_3", None);
        let effective_date = NaiveDate::from_ymd_opt(2025, 8, 1).unwrap();

        let result = get_base_rate(&employee, effective_date, &config, 1).unwrap();

        assert_eq!(result.rate, dec("28.54"));
        assert_eq!(result.audit_step.rule_id, "base_rate_lookup");
        assert_eq!(result.audit_step.clause_ref, "14.2");
        assert!(
            result.audit_step.input["classification_code"]
                .as_str()
                .unwrap()
                .contains("dce_level_3")
        );
        assert!(
            result.audit_step.output["rate"]
                .as_str()
                .unwrap()
                .contains("28.54")
        );
    }

    /// BR-002: override rate takes precedence
    #[test]
    fn test_override_rate_takes_precedence() {
        let config = create_test_config();
        let employee = create_test_employee("dce_level_3", Some(dec("32.00")));
        let effective_date = NaiveDate::from_ymd_opt(2025, 8, 1).unwrap();

        let result = get_base_rate(&employee, effective_date, &config, 1).unwrap();

        assert_eq!(result.rate, dec("32.00"));
        assert_eq!(result.audit_step.rule_id, "base_rate_lookup");
        assert_eq!(result.audit_step.clause_ref, "14.2");
        assert!(
            result.audit_step.output["source"]
                .as_str()
                .unwrap()
                .contains("employee_override")
        );
    }

    /// BR-003: unknown classification returns error
    #[test]
    fn test_unknown_classification_returns_error() {
        let config = create_test_config();
        let employee = create_test_employee("unknown", None);
        let effective_date = NaiveDate::from_ymd_opt(2025, 8, 1).unwrap();

        let result = get_base_rate(&employee, effective_date, &config, 1);

        assert!(result.is_err());
        match result.unwrap_err() {
            EngineError::ClassificationNotFound { code } => {
                assert_eq!(code, "unknown");
            }
            other => panic!("Expected ClassificationNotFound, got {:?}", other),
        }
    }

    /// BR-004: no rate for early date returns error
    #[test]
    fn test_no_rate_for_early_date_returns_error() {
        let config = create_test_config();
        let employee = create_test_employee("dce_level_3", None);
        let effective_date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();

        let result = get_base_rate(&employee, effective_date, &config, 1);

        assert!(result.is_err());
        match result.unwrap_err() {
            EngineError::RateNotFound {
                classification,
                date,
            } => {
                assert_eq!(classification, "dce_level_3");
                assert_eq!(date, NaiveDate::from_ymd_opt(2020, 1, 1).unwrap());
            }
            other => panic!("Expected RateNotFound, got {:?}", other),
        }
    }

    #[test]
    fn test_audit_step_has_correct_step_number() {
        let config = create_test_config();
        let employee = create_test_employee("dce_level_3", None);
        let effective_date = NaiveDate::from_ymd_opt(2025, 8, 1).unwrap();

        let result = get_base_rate(&employee, effective_date, &config, 5).unwrap();

        assert_eq!(result.audit_step.step_number, 5);
    }

    #[test]
    fn test_audit_step_reasoning_contains_rate() {
        let config = create_test_config();
        let employee = create_test_employee("dce_level_3", None);
        let effective_date = NaiveDate::from_ymd_opt(2025, 8, 1).unwrap();

        let result = get_base_rate(&employee, effective_date, &config, 1).unwrap();

        assert!(result.audit_step.reasoning.contains("28.54"));
    }
}
