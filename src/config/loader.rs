//! Configuration loading functionality.
//!
//! This module provides the [`ConfigLoader`] type for loading award
//! configurations from YAML files.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::fs;
use std::path::Path;

use crate::error::{EngineError, EngineResult};
use crate::models::EmploymentType;

use super::types::{
    AwardConfig, AwardMetadata, Classification, ClassificationsConfig, PenaltyConfig, RateConfig,
};

/// Loads and provides access to award configuration.
///
/// The `ConfigLoader` reads YAML configuration files from a directory
/// and provides methods to query classifications, rates, and penalties.
///
/// # Directory Structure
///
/// The configuration directory should have the following structure:
/// ```text
/// config/ma000018/
/// ├── award.yaml          # Award metadata
/// ├── classifications.yaml # Employee classifications
/// ├── penalties.yaml       # Penalty and overtime rates
/// └── rates/
///     └── 2025-07-01.yaml  # Rates effective from this date
/// ```
///
/// # Example
///
/// ```no_run
/// use award_engine::config::ConfigLoader;
/// use chrono::NaiveDate;
///
/// let loader = ConfigLoader::load("./config/ma000018").unwrap();
///
/// // Get a classification
/// let classification = loader.get_classification("dce_level_3").unwrap();
/// println!("Classification: {}", classification.name);
///
/// // Get the hourly rate for a classification on a specific date
/// let date = NaiveDate::from_ymd_opt(2025, 8, 1).unwrap();
/// let rate = loader.get_hourly_rate("dce_level_3", date).unwrap();
/// println!("Hourly rate: ${}", rate);
/// ```
#[derive(Debug, Clone)]
pub struct ConfigLoader {
    config: AwardConfig,
}

impl ConfigLoader {
    /// Loads configuration from the specified directory.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration directory (e.g., "./config/ma000018")
    ///
    /// # Returns
    ///
    /// Returns a `ConfigLoader` instance on success, or an error if:
    /// - Any required file is missing
    /// - Any file contains invalid YAML
    /// - Any required field is missing from the configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// use award_engine::config::ConfigLoader;
    ///
    /// let loader = ConfigLoader::load("./config/ma000018")?;
    /// # Ok::<(), award_engine::error::EngineError>(())
    /// ```
    pub fn load<P: AsRef<Path>>(path: P) -> EngineResult<Self> {
        let path = path.as_ref();

        // Load award.yaml
        let award_path = path.join("award.yaml");
        let metadata = Self::load_yaml::<AwardMetadata>(&award_path)?;

        // Load classifications.yaml
        let classifications_path = path.join("classifications.yaml");
        let classifications_config =
            Self::load_yaml::<ClassificationsConfig>(&classifications_path)?;

        // Load penalties.yaml
        let penalties_path = path.join("penalties.yaml");
        let penalties = Self::load_yaml::<PenaltyConfig>(&penalties_path)?;

        // Load all rate files from the rates directory
        let rates_dir = path.join("rates");
        let rates = Self::load_rates(&rates_dir)?;

        let config = AwardConfig::new(
            metadata,
            classifications_config.classifications,
            rates,
            penalties,
        );

        Ok(Self { config })
    }

    /// Loads and parses a YAML file.
    fn load_yaml<T: serde::de::DeserializeOwned>(path: &Path) -> EngineResult<T> {
        let path_str = path.display().to_string();

        let content = fs::read_to_string(path).map_err(|_| EngineError::ConfigNotFound {
            path: path_str.clone(),
        })?;

        serde_yaml::from_str(&content).map_err(|e| EngineError::ConfigParseError {
            path: path_str,
            message: e.to_string(),
        })
    }

    /// Loads all rate files from the rates directory.
    fn load_rates(rates_dir: &Path) -> EngineResult<Vec<RateConfig>> {
        let rates_dir_str = rates_dir.display().to_string();

        if !rates_dir.exists() {
            return Err(EngineError::ConfigNotFound {
                path: rates_dir_str,
            });
        }

        let entries = fs::read_dir(rates_dir).map_err(|_| EngineError::ConfigNotFound {
            path: rates_dir_str.clone(),
        })?;

        let mut rates = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|_| EngineError::ConfigNotFound {
                path: rates_dir_str.clone(),
            })?;

            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "yaml") {
                let rate_config = Self::load_yaml::<RateConfig>(&path)?;
                rates.push(rate_config);
            }
        }

        if rates.is_empty() {
            return Err(EngineError::ConfigNotFound {
                path: format!("{} (no rate files found)", rates_dir_str),
            });
        }

        Ok(rates)
    }

    /// Returns the underlying award configuration.
    pub fn config(&self) -> &AwardConfig {
        &self.config
    }

    /// Returns the award metadata.
    pub fn award(&self) -> &AwardMetadata {
        self.config.award()
    }

    /// Gets a classification by its code.
    ///
    /// # Arguments
    ///
    /// * `code` - The classification code (e.g., "dce_level_3")
    ///
    /// # Returns
    ///
    /// Returns the classification if found, or `ClassificationNotFound` error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use award_engine::config::ConfigLoader;
    ///
    /// let loader = ConfigLoader::load("./config/ma000018")?;
    /// let classification = loader.get_classification("dce_level_3")?;
    /// println!("Classification: {}", classification.name);
    /// # Ok::<(), award_engine::error::EngineError>(())
    /// ```
    pub fn get_classification(&self, code: &str) -> EngineResult<&Classification> {
        self.config
            .classifications()
            .get(code)
            .ok_or_else(|| EngineError::ClassificationNotFound {
                code: code.to_string(),
            })
    }

    /// Gets the hourly rate for a classification on a given date.
    ///
    /// The method finds the most recent rate configuration that is effective
    /// on or before the given date.
    ///
    /// # Arguments
    ///
    /// * `classification` - The classification code
    /// * `date` - The date for which to get the rate
    ///
    /// # Returns
    ///
    /// Returns the hourly rate if found, or an error if:
    /// - The classification is not found in any rate configuration
    /// - No rate configuration is effective for the given date
    ///
    /// # Example
    ///
    /// ```no_run
    /// use award_engine::config::ConfigLoader;
    /// use chrono::NaiveDate;
    ///
    /// let loader = ConfigLoader::load("./config/ma000018")?;
    /// let date = NaiveDate::from_ymd_opt(2025, 8, 1).unwrap();
    /// let rate = loader.get_hourly_rate("dce_level_3", date)?;
    /// println!("Hourly rate: ${}", rate);
    /// # Ok::<(), award_engine::error::EngineError>(())
    /// ```
    pub fn get_hourly_rate(&self, classification: &str, date: NaiveDate) -> EngineResult<Decimal> {
        // Find the most recent rate config that is effective on or before the date
        let rate_config = self
            .config
            .rates()
            .iter()
            .rev()
            .find(|rc| rc.effective_date <= date)
            .ok_or_else(|| EngineError::RateNotFound {
                classification: classification.to_string(),
                date,
            })?;

        rate_config
            .rates
            .get(classification)
            .map(|r| r.hourly)
            .ok_or_else(|| EngineError::RateNotFound {
                classification: classification.to_string(),
                date,
            })
    }

    /// Gets the penalty rate multiplier for a day type and employment type.
    ///
    /// # Arguments
    ///
    /// * `day_type` - The type of day ("saturday" or "sunday")
    /// * `employment_type` - The employee's employment type
    ///
    /// # Returns
    ///
    /// Returns the penalty multiplier (e.g., 1.5 for 150% of base rate).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use award_engine::config::ConfigLoader;
    /// use award_engine::models::EmploymentType;
    ///
    /// let loader = ConfigLoader::load("./config/ma000018")?;
    /// let penalty = loader.get_penalty("saturday", EmploymentType::Casual)?;
    /// println!("Saturday casual penalty: {}x", penalty);
    /// # Ok::<(), award_engine::error::EngineError>(())
    /// ```
    pub fn get_penalty(
        &self,
        day_type: &str,
        employment_type: EmploymentType,
    ) -> EngineResult<Decimal> {
        let penalties = &self.config.penalties().penalties;

        let penalty_rates = match day_type.to_lowercase().as_str() {
            "saturday" => &penalties.saturday,
            "sunday" => &penalties.sunday,
            _ => {
                return Err(EngineError::CalculationError {
                    message: format!("Unknown day type: {}", day_type),
                });
            }
        };

        Ok(match employment_type {
            EmploymentType::FullTime => penalty_rates.full_time,
            EmploymentType::PartTime => penalty_rates.part_time,
            EmploymentType::Casual => penalty_rates.casual,
        })
    }

    /// Gets the allowance rates from the most recent rate configuration.
    pub fn get_allowance_rates(&self, date: NaiveDate) -> EngineResult<(Decimal, Decimal)> {
        let rate_config = self
            .config
            .rates()
            .iter()
            .rev()
            .find(|rc| rc.effective_date <= date)
            .ok_or_else(|| EngineError::ConfigNotFound {
                path: "No rate configuration found for date".to_string(),
            })?;

        Ok((
            rate_config.allowances.laundry_per_shift,
            rate_config.allowances.laundry_per_week,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn config_path() -> &'static str {
        "./config/ma000018"
    }

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn test_load_valid_configuration() {
        let result = ConfigLoader::load(config_path());
        assert!(result.is_ok(), "Failed to load config: {:?}", result.err());

        let loader = result.unwrap();
        assert_eq!(loader.award().code, "MA000018");
        assert_eq!(loader.award().name, "Aged Care Award 2010");
    }

    #[test]
    fn test_get_classification() {
        let loader = ConfigLoader::load(config_path()).unwrap();

        let classification = loader.get_classification("dce_level_3");
        assert!(classification.is_ok());

        let classification = classification.unwrap();
        assert_eq!(
            classification.name,
            "Direct Care Employee Level 3 - Qualified"
        );
        assert_eq!(classification.clause, "14.2");
    }

    #[test]
    fn test_get_classification_unknown_returns_error() {
        let loader = ConfigLoader::load(config_path()).unwrap();

        let result = loader.get_classification("unknown");
        assert!(result.is_err());

        match result {
            Err(EngineError::ClassificationNotFound { code }) => {
                assert_eq!(code, "unknown");
            }
            _ => panic!("Expected ClassificationNotFound error"),
        }
    }

    #[test]
    fn test_get_hourly_rate_for_dce_level_3() {
        let loader = ConfigLoader::load(config_path()).unwrap();

        let date = NaiveDate::from_ymd_opt(2025, 8, 1).unwrap();
        let rate = loader.get_hourly_rate("dce_level_3", date);

        assert!(rate.is_ok(), "Failed to get rate: {:?}", rate.err());
        assert_eq!(rate.unwrap(), dec("28.54"));
    }

    #[test]
    fn test_get_penalty_saturday_casual() {
        let loader = ConfigLoader::load(config_path()).unwrap();

        let penalty = loader.get_penalty("saturday", EmploymentType::Casual);
        assert!(penalty.is_ok());
        assert_eq!(penalty.unwrap(), dec("1.75"));
    }

    #[test]
    fn test_get_penalty_saturday_fulltime() {
        let loader = ConfigLoader::load(config_path()).unwrap();

        let penalty = loader.get_penalty("saturday", EmploymentType::FullTime);
        assert!(penalty.is_ok());
        assert_eq!(penalty.unwrap(), dec("1.50"));
    }

    #[test]
    fn test_get_penalty_sunday_casual() {
        let loader = ConfigLoader::load(config_path()).unwrap();

        let penalty = loader.get_penalty("sunday", EmploymentType::Casual);
        assert!(penalty.is_ok());
        assert_eq!(penalty.unwrap(), dec("2.00"));
    }

    #[test]
    fn test_get_penalty_sunday_fulltime() {
        let loader = ConfigLoader::load(config_path()).unwrap();

        let penalty = loader.get_penalty("sunday", EmploymentType::FullTime);
        assert!(penalty.is_ok());
        assert_eq!(penalty.unwrap(), dec("1.75"));
    }

    #[test]
    fn test_load_missing_directory_returns_error() {
        let result = ConfigLoader::load("/nonexistent/path");
        assert!(result.is_err());

        match result {
            Err(EngineError::ConfigNotFound { path }) => {
                assert!(path.contains("award.yaml"));
            }
            _ => panic!("Expected ConfigNotFound error"),
        }
    }

    #[test]
    fn test_award_metadata_loaded_correctly() {
        let loader = ConfigLoader::load(config_path()).unwrap();

        assert_eq!(loader.award().code, "MA000018");
        assert_eq!(loader.award().name, "Aged Care Award 2010");
        assert_eq!(loader.award().version, "2025-07-01");
        assert_eq!(
            loader.award().source_url,
            "https://library.fairwork.gov.au/award/?krn=MA000018"
        );
    }

    #[test]
    fn test_allowance_rates_loaded_correctly() {
        let loader = ConfigLoader::load(config_path()).unwrap();

        let date = NaiveDate::from_ymd_opt(2025, 8, 1).unwrap();
        let (per_shift, per_week) = loader.get_allowance_rates(date).unwrap();

        assert_eq!(per_shift, dec("0.32"));
        assert_eq!(per_week, dec("1.49"));
    }

    #[test]
    fn test_rate_not_found_for_date_before_effective() {
        let loader = ConfigLoader::load(config_path()).unwrap();

        // Date before the effective date of any rate config
        let date = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let result = loader.get_hourly_rate("dce_level_3", date);

        assert!(result.is_err());
        match result {
            Err(EngineError::RateNotFound {
                classification,
                date: d,
            }) => {
                assert_eq!(classification, "dce_level_3");
                assert_eq!(d, date);
            }
            _ => panic!("Expected RateNotFound error"),
        }
    }
}
