//! Error types for the Award Interpretation Engine.
//!
//! This module provides strongly-typed errors using the `thiserror` crate
//! for all error conditions that can occur during award interpretation.

use chrono::NaiveDate;
use thiserror::Error;

/// The main error type for the Award Interpretation Engine.
///
/// All operations in the engine return this error type, making it easy
/// to handle errors consistently throughout the application.
///
/// # Example
///
/// ```
/// use award_engine::error::EngineError;
///
/// let error = EngineError::ConfigNotFound {
///     path: "/missing/file.yaml".to_string(),
/// };
/// assert_eq!(error.to_string(), "Configuration file not found: /missing/file.yaml");
/// ```
#[derive(Debug, Error)]
pub enum EngineError {
    /// Configuration file was not found at the specified path.
    #[error("Configuration file not found: {path}")]
    ConfigNotFound {
        /// The path that was not found.
        path: String,
    },

    /// Configuration file could not be parsed.
    #[error("Failed to parse configuration file '{path}': {message}")]
    ConfigParseError {
        /// The path to the file that failed to parse.
        path: String,
        /// A description of the parse error.
        message: String,
    },

    /// Classification code was not found in the configuration.
    #[error("Classification not found: {code}")]
    ClassificationNotFound {
        /// The classification code that was not found.
        code: String,
    },

    /// No rate was found for the given classification and date.
    #[error("Rate not found for classification '{classification}' on date {date}")]
    RateNotFound {
        /// The classification code.
        classification: String,
        /// The date for which the rate was requested.
        date: NaiveDate,
    },

    /// A shift was invalid or contained inconsistent data.
    #[error("Invalid shift '{shift_id}': {message}")]
    InvalidShift {
        /// The ID of the invalid shift.
        shift_id: String,
        /// A description of what made the shift invalid.
        message: String,
    },

    /// An employee record was invalid or contained inconsistent data.
    #[error("Invalid employee field '{field}': {message}")]
    InvalidEmployee {
        /// The field that was invalid.
        field: String,
        /// A description of what made the field invalid.
        message: String,
    },

    /// A general calculation error occurred.
    #[error("Calculation error: {message}")]
    CalculationError {
        /// A description of the calculation error.
        message: String,
    },
}

/// A type alias for Results that return EngineError.
pub type EngineResult<T> = Result<T, EngineError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_not_found_displays_path() {
        let error = EngineError::ConfigNotFound {
            path: "/missing/file.yaml".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Configuration file not found: /missing/file.yaml"
        );
    }

    #[test]
    fn test_classification_not_found_displays_code() {
        let error = EngineError::ClassificationNotFound {
            code: "unknown".to_string(),
        };
        assert_eq!(error.to_string(), "Classification not found: unknown");
    }

    #[test]
    fn test_config_parse_error_displays_path_and_message() {
        let error = EngineError::ConfigParseError {
            path: "/config/bad.yaml".to_string(),
            message: "invalid YAML syntax".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Failed to parse configuration file '/config/bad.yaml': invalid YAML syntax"
        );
    }

    #[test]
    fn test_rate_not_found_displays_classification_and_date() {
        let error = EngineError::RateNotFound {
            classification: "dce_level_3".to_string(),
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        };
        assert_eq!(
            error.to_string(),
            "Rate not found for classification 'dce_level_3' on date 2025-01-01"
        );
    }

    #[test]
    fn test_invalid_shift_displays_id_and_message() {
        let error = EngineError::InvalidShift {
            shift_id: "shift_001".to_string(),
            message: "end time before start time".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Invalid shift 'shift_001': end time before start time"
        );
    }

    #[test]
    fn test_invalid_employee_displays_field_and_message() {
        let error = EngineError::InvalidEmployee {
            field: "date_of_birth".to_string(),
            message: "cannot be in the future".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Invalid employee field 'date_of_birth': cannot be in the future"
        );
    }

    #[test]
    fn test_calculation_error_displays_message() {
        let error = EngineError::CalculationError {
            message: "negative hours calculated".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Calculation error: negative hours calculated"
        );
    }

    #[test]
    fn test_errors_implement_std_error() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<EngineError>();
    }

    #[test]
    fn test_error_propagation_with_question_mark() {
        fn returns_config_not_found() -> EngineResult<()> {
            Err(EngineError::ConfigNotFound {
                path: "/test".to_string(),
            })
        }

        fn propagates_error() -> EngineResult<()> {
            returns_config_not_found()?;
            Ok(())
        }

        assert!(propagates_error().is_err());
    }
}
