//! Configuration types for award interpretation.
//!
//! This module contains the strongly-typed configuration structures that
//! are deserialized from YAML configuration files.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::collections::HashMap;

/// Metadata about the award.
///
/// Contains identifying information about the award, including its
/// Fair Work code, name, version, and source URL.
#[derive(Debug, Clone, Deserialize)]
pub struct AwardMetadata {
    /// The Fair Work award code (e.g., "MA000018").
    pub code: String,
    /// The human-readable name of the award.
    pub name: String,
    /// The version or effective date of the award.
    pub version: String,
    /// URL to the official award documentation.
    pub source_url: String,
}

/// A classification within the award.
///
/// Classifications define the various employee categories and their
/// associated pay grades.
#[derive(Debug, Clone, Deserialize)]
pub struct Classification {
    /// The human-readable name of the classification.
    pub name: String,
    /// A description of the classification.
    pub description: String,
    /// Reference to the award clause defining this classification.
    pub clause: String,
}

/// Classifications configuration file structure.
#[derive(Debug, Clone, Deserialize)]
pub struct ClassificationsConfig {
    /// Map of classification code to classification details.
    pub classifications: HashMap<String, Classification>,
}

/// Rate information for a specific classification.
#[derive(Debug, Clone, Deserialize)]
pub struct ClassificationRate {
    /// The weekly rate for this classification.
    pub weekly: Decimal,
    /// The hourly rate for this classification.
    pub hourly: Decimal,
}

/// Allowance rates.
#[derive(Debug, Clone, Deserialize)]
pub struct AllowanceRates {
    /// The laundry allowance per shift.
    pub laundry_per_shift: Decimal,
    /// The maximum laundry allowance per week.
    pub laundry_per_week: Decimal,
}

/// Rate configuration for a specific effective date.
#[derive(Debug, Clone, Deserialize)]
pub struct RateConfig {
    /// The effective date for these rates.
    pub effective_date: NaiveDate,
    /// Map of classification code to rates.
    pub rates: HashMap<String, ClassificationRate>,
    /// Allowance rates.
    pub allowances: AllowanceRates,
}

/// Penalty rates by employment type.
#[derive(Debug, Clone, Deserialize)]
pub struct PenaltyRates {
    /// Reference to the award clause for these penalties.
    pub clause: String,
    /// Penalty multiplier for full-time employees.
    pub full_time: Decimal,
    /// Penalty multiplier for part-time employees.
    pub part_time: Decimal,
    /// Penalty multiplier for casual employees.
    pub casual: Decimal,
}

/// Overtime rates by employment type.
#[derive(Debug, Clone, Deserialize)]
pub struct OvertimeRates {
    /// Overtime multiplier for full-time employees.
    pub full_time: Decimal,
    /// Overtime multiplier for part-time employees.
    pub part_time: Decimal,
    /// Overtime multiplier for casual employees.
    pub casual: Decimal,
}

/// Overtime configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct OvertimeConfig {
    /// Reference to the award clause for overtime.
    pub clause: String,
    /// Rates for the first two hours of overtime.
    pub first_two_hours: OvertimeRates,
    /// Rates for overtime after two hours.
    pub after_two_hours: OvertimeRates,
}

/// Penalty configuration from penalties.yaml.
#[derive(Debug, Clone, Deserialize)]
pub struct PenaltyConfig {
    /// Penalty rates configuration.
    pub penalties: Penalties,
    /// Overtime configuration.
    pub overtime: OvertimeSection,
}

/// Penalties section.
#[derive(Debug, Clone, Deserialize)]
pub struct Penalties {
    /// Saturday penalty rates.
    pub saturday: PenaltyRates,
    /// Sunday penalty rates.
    pub sunday: PenaltyRates,
}

/// Overtime section in penalties config.
#[derive(Debug, Clone, Deserialize)]
pub struct OvertimeSection {
    /// Number of hours before overtime kicks in on a weekday.
    pub daily_threshold_hours: u32,
    /// Weekday overtime rates.
    pub weekday: OvertimeConfig,
}

/// The complete award configuration loaded from YAML files.
///
/// This struct aggregates all configuration loaded from the various
/// YAML files in an award configuration directory.
#[derive(Debug, Clone)]
pub struct AwardConfig {
    /// Award metadata.
    metadata: AwardMetadata,
    /// Classifications available under this award.
    classifications: HashMap<String, Classification>,
    /// Rate configurations by effective date (sorted oldest first).
    rates: Vec<RateConfig>,
    /// Penalty configuration.
    penalties: PenaltyConfig,
}

impl AwardConfig {
    /// Creates a new AwardConfig from its component parts.
    pub fn new(
        metadata: AwardMetadata,
        classifications: HashMap<String, Classification>,
        rates: Vec<RateConfig>,
        penalties: PenaltyConfig,
    ) -> Self {
        let mut sorted_rates = rates;
        sorted_rates.sort_by(|a, b| a.effective_date.cmp(&b.effective_date));
        Self {
            metadata,
            classifications,
            rates: sorted_rates,
            penalties,
        }
    }

    /// Returns the award metadata.
    pub fn award(&self) -> &AwardMetadata {
        &self.metadata
    }

    /// Returns all classifications.
    pub fn classifications(&self) -> &HashMap<String, Classification> {
        &self.classifications
    }

    /// Returns the penalty configuration.
    pub fn penalties(&self) -> &PenaltyConfig {
        &self.penalties
    }

    /// Returns all rate configurations.
    pub fn rates(&self) -> &[RateConfig] {
        &self.rates
    }
}
