//! Response types for the Award Interpretation Engine API.
//!
//! This module defines the error response structures and error handling
//! for the HTTP API.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::error::EngineError;

/// Health check response structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Health status ("healthy" or "unhealthy").
    pub status: String,
    /// Engine version (present when healthy).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Reason for unhealthy status (present when unhealthy).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl HealthResponse {
    /// Creates a healthy response with version information.
    pub fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
            reason: None,
        }
    }

    /// Creates an unhealthy response with a reason.
    pub fn unhealthy(reason: impl Into<String>) -> Self {
        Self {
            status: "unhealthy".to_string(),
            version: None,
            reason: Some(reason.into()),
        }
    }
}

/// API error response structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error code for programmatic handling.
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// Optional details about the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ApiError {
    /// Creates a new API error.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    /// Creates a new API error with details.
    pub fn with_details(
        code: impl Into<String>,
        message: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: Some(details.into()),
        }
    }

    /// Creates a validation error response.
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::new("VALIDATION_ERROR", message)
    }

    /// Creates a classification not found error response.
    pub fn classification_not_found(code: &str) -> Self {
        Self::with_details(
            "CLASSIFICATION_NOT_FOUND",
            format!("Classification not found: {}", code),
            format!("The classification code '{}' is not supported by this engine", code),
        )
    }

    /// Creates a malformed JSON error response.
    pub fn malformed_json(message: impl Into<String>) -> Self {
        Self::new("MALFORMED_JSON", message)
    }

    /// Creates a missing field error response.
    pub fn missing_field(field: impl Into<String>) -> Self {
        let field = field.into();
        Self::with_details(
            "MISSING_FIELD",
            format!("missing field: {}", field),
            format!("Required field '{}' was not provided in the request", field),
        )
    }
}

/// API error with HTTP status code.
pub struct ApiErrorResponse {
    /// The HTTP status code.
    pub status: StatusCode,
    /// The error body.
    pub error: ApiError,
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
        (self.status, Json(self.error)).into_response()
    }
}

impl From<EngineError> for ApiErrorResponse {
    fn from(error: EngineError) -> Self {
        match error {
            EngineError::ConfigNotFound { path } => ApiErrorResponse {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                error: ApiError::with_details(
                    "CONFIG_ERROR",
                    "Configuration error",
                    format!("Configuration file not found: {}", path),
                ),
            },
            EngineError::ConfigParseError { path, message } => ApiErrorResponse {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                error: ApiError::with_details(
                    "CONFIG_ERROR",
                    "Configuration parse error",
                    format!("Failed to parse {}: {}", path, message),
                ),
            },
            EngineError::ClassificationNotFound { code } => ApiErrorResponse {
                status: StatusCode::BAD_REQUEST,
                error: ApiError::classification_not_found(&code),
            },
            EngineError::RateNotFound {
                classification,
                date,
            } => ApiErrorResponse {
                status: StatusCode::BAD_REQUEST,
                error: ApiError::with_details(
                    "RATE_NOT_FOUND",
                    format!(
                        "Rate not found for classification '{}' on date {}",
                        classification, date
                    ),
                    "The requested classification does not have a rate for the specified date",
                ),
            },
            EngineError::InvalidShift { shift_id, message } => ApiErrorResponse {
                status: StatusCode::BAD_REQUEST,
                error: ApiError::with_details(
                    "INVALID_SHIFT",
                    format!("Invalid shift '{}': {}", shift_id, message),
                    "The shift data contains invalid information",
                ),
            },
            EngineError::InvalidEmployee { field, message } => ApiErrorResponse {
                status: StatusCode::BAD_REQUEST,
                error: ApiError::with_details(
                    "INVALID_EMPLOYEE",
                    format!("Invalid employee field '{}': {}", field, message),
                    "The employee data contains invalid information",
                ),
            },
            EngineError::CalculationError { message } => ApiErrorResponse {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                error: ApiError::with_details(
                    "CALCULATION_ERROR",
                    "Calculation failed",
                    message,
                ),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_serialization() {
        let error = ApiError::new("TEST_ERROR", "Test message");
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"code\":\"TEST_ERROR\""));
        assert!(json.contains("\"message\":\"Test message\""));
        assert!(!json.contains("details")); // Should be skipped when None
    }

    #[test]
    fn test_api_error_with_details_serialization() {
        let error = ApiError::with_details("TEST_ERROR", "Test message", "Some details");
        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("\"details\":\"Some details\""));
    }

    #[test]
    fn test_classification_not_found_error() {
        let error = ApiError::classification_not_found("unknown_class");
        assert_eq!(error.code, "CLASSIFICATION_NOT_FOUND");
        assert!(error.message.contains("unknown_class"));
    }

    #[test]
    fn test_engine_error_to_api_error() {
        let engine_error = EngineError::ClassificationNotFound {
            code: "invalid".to_string(),
        };
        let api_error: ApiErrorResponse = engine_error.into();
        assert_eq!(api_error.status, StatusCode::BAD_REQUEST);
        assert_eq!(api_error.error.code, "CLASSIFICATION_NOT_FOUND");
    }

    #[test]
    fn test_health_response_healthy() {
        let response = HealthResponse::healthy();
        assert_eq!(response.status, "healthy");
        assert_eq!(response.version, Some("0.1.0".to_string()));
        assert!(response.reason.is_none());
    }

    #[test]
    fn test_health_response_unhealthy() {
        let response = HealthResponse::unhealthy("Configuration error");
        assert_eq!(response.status, "unhealthy");
        assert!(response.version.is_none());
        assert_eq!(response.reason, Some("Configuration error".to_string()));
    }

    #[test]
    fn test_health_response_healthy_serialization() {
        let response = HealthResponse::healthy();
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"healthy\""));
        assert!(json.contains("\"version\":\"0.1.0\""));
        // Reason should not appear in healthy response
        assert!(!json.contains("reason"));
    }

    #[test]
    fn test_health_response_unhealthy_serialization() {
        let response = HealthResponse::unhealthy("Database unreachable");
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"status\":\"unhealthy\""));
        assert!(json.contains("\"reason\":\"Database unreachable\""));
        // Version should not appear in unhealthy response
        assert!(!json.contains("version"));
    }
}
