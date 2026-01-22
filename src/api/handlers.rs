//! HTTP request handlers for the Award Interpretation Engine API.
//!
//! This module contains the handler functions for all API endpoints.

use std::time::Instant;

use axum::{
    extract::{rejection::JsonRejection, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use chrono::Utc;
use rust_decimal::Decimal;
use tracing::{info, warn};
use uuid::Uuid;

use crate::calculation::{
    calculate_laundry_allowance, calculate_ordinary_hours, calculate_saturday_pay,
    calculate_sunday_pay, calculate_weekday_overtime, calculate_weekend_overtime,
    detect_daily_overtime, get_base_rate, get_day_type, segment_by_day, DayType,
    DEFAULT_DAILY_OVERTIME_THRESHOLD,
};
use crate::models::{
    AllowancePayment, AuditStep, AuditTrace, AuditWarning, CalculationResult, Employee,
    PayCategory, PayLine, PayPeriod, PayTotals, Shift,
};

use super::request::CalculationRequest;
use super::response::{ApiError, ApiErrorResponse};
use super::state::AppState;

/// Creates the API router with all endpoints.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/calculate", post(calculate_handler))
        .with_state(state)
}

/// Handler for POST /calculate endpoint.
///
/// Accepts a calculation request and returns the calculated pay result.
async fn calculate_handler(
    State(state): State<AppState>,
    payload: Result<Json<CalculationRequest>, JsonRejection>,
) -> impl IntoResponse {
    // Generate correlation ID for request tracking
    let correlation_id = Uuid::new_v4();
    info!(correlation_id = %correlation_id, "Processing calculation request");

    // Handle JSON parsing errors
    let request = match payload {
        Ok(Json(req)) => req,
        Err(rejection) => {
            let error = match rejection {
                JsonRejection::JsonDataError(err) => {
                    // Get the body text which contains the detailed error from serde
                    let body_text = err.body_text();
                    warn!(
                        correlation_id = %correlation_id,
                        error = %body_text,
                        "JSON data error"
                    );
                    // Check if it's a missing field error
                    if body_text.contains("missing field") {
                        ApiError::new("VALIDATION_ERROR", body_text)
                    } else {
                        ApiError::malformed_json(body_text)
                    }
                }
                JsonRejection::JsonSyntaxError(err) => {
                    warn!(
                        correlation_id = %correlation_id,
                        error = %err,
                        "JSON syntax error"
                    );
                    ApiError::malformed_json(format!("Invalid JSON syntax: {}", err))
                }
                JsonRejection::MissingJsonContentType(_) => {
                    ApiError::new("MISSING_CONTENT_TYPE", "Content-Type must be application/json")
                }
                _ => ApiError::malformed_json("Failed to parse request body"),
            };
            return (
                StatusCode::BAD_REQUEST,
                [(header::CONTENT_TYPE, "application/json")],
                Json(error),
            )
                .into_response();
        }
    };

    // Convert request types to domain types
    let employee: Employee = request.employee.into();
    let pay_period: PayPeriod = request.pay_period.into();
    let shifts: Vec<Shift> = request.shifts.into_iter().map(Into::into).collect();

    // Validate the classification exists
    let config = state.config();
    if let Err(err) = config.get_classification(&employee.classification_code) {
        warn!(
            correlation_id = %correlation_id,
            classification = %employee.classification_code,
            "Classification not found"
        );
        let api_error: ApiErrorResponse = err.into();
        return (
            api_error.status,
            [(header::CONTENT_TYPE, "application/json")],
            Json(api_error.error),
        )
            .into_response();
    }

    // Perform the calculation
    let start_time = Instant::now();
    match perform_calculation(&employee, &pay_period, &shifts, config) {
        Ok(result) => {
            let duration = start_time.elapsed();
            info!(
                correlation_id = %correlation_id,
                employee_id = %employee.id,
                shifts_count = shifts.len(),
                gross_pay = %result.totals.gross_pay,
                duration_us = duration.as_micros(),
                "Calculation completed successfully"
            );
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "application/json")],
                Json(result),
            )
                .into_response()
        }
        Err(err) => {
            warn!(
                correlation_id = %correlation_id,
                error = %err,
                "Calculation failed"
            );
            let api_error: ApiErrorResponse = err.into();
            (
                api_error.status,
                [(header::CONTENT_TYPE, "application/json")],
                Json(api_error.error),
            )
                .into_response()
        }
    }
}

/// Performs the pay calculation for an employee's shifts.
fn perform_calculation(
    employee: &Employee,
    pay_period: &PayPeriod,
    shifts: &[Shift],
    config: &crate::config::ConfigLoader,
) -> Result<CalculationResult, crate::error::EngineError> {
    let start_time = Instant::now();
    let mut all_pay_lines: Vec<PayLine> = Vec::new();
    let mut all_audit_steps: Vec<AuditStep> = Vec::new();
    let all_warnings: Vec<AuditWarning> = Vec::new();
    let mut step_number: u32 = 1;

    let award_config = config.config();

    // Get the effective date for rate lookups (use first shift date or pay period start)
    let effective_date = shifts
        .first()
        .map(|s| s.date)
        .unwrap_or(pay_period.start_date);

    // Get base rate for the employee
    let base_rate_result = get_base_rate(employee, effective_date, award_config, step_number)?;
    let base_rate = base_rate_result.rate;
    all_audit_steps.push(base_rate_result.audit_step);
    step_number += 1;

    // Process each shift
    for shift in shifts {
        // Segment the shift by day (handles overnight shifts)
        let segments = segment_by_day(shift);
        let total_worked_hours = shift.worked_hours();

        // Detect daily overtime for the entire shift
        let overtime_detection = detect_daily_overtime(
            total_worked_hours,
            DEFAULT_DAILY_OVERTIME_THRESHOLD,
            step_number,
        );
        all_audit_steps.push(overtime_detection.audit_step.clone());
        step_number += 1;

        // Track if we've already handled ordinary hours for this shift
        let mut ordinary_hours_remaining = overtime_detection.ordinary_hours;

        for segment in &segments {
            let day_type = get_day_type(segment.start_time);

            // Calculate hours for this segment, limited by remaining ordinary hours
            let segment_ordinary_hours = if ordinary_hours_remaining >= segment.hours {
                ordinary_hours_remaining -= segment.hours;
                segment.hours
            } else {
                let hours = ordinary_hours_remaining;
                ordinary_hours_remaining = Decimal::ZERO;
                hours
            };

            match day_type {
                DayType::Weekday => {
                    if segment_ordinary_hours > Decimal::ZERO {
                        // Calculate ordinary hours using the existing function
                        let ordinary_result = calculate_ordinary_hours(
                            shift,
                            employee,
                            award_config,
                            step_number,
                        )?;

                        // Adjust the pay line for the actual segment hours
                        let mut pay_line = ordinary_result.pay_line;
                        pay_line.shift_id = shift.id.clone();
                        pay_line.date = segment.start_time.date();
                        pay_line.hours = segment_ordinary_hours;
                        pay_line.amount = segment_ordinary_hours * pay_line.rate;

                        all_pay_lines.push(pay_line);
                        let steps_count = ordinary_result.audit_steps.len();
                        all_audit_steps.extend(ordinary_result.audit_steps);
                        step_number += steps_count as u32;
                    }
                }
                DayType::Saturday => {
                    if segment_ordinary_hours > Decimal::ZERO {
                        // Create a segment for the ordinary hours
                        let mut seg = segment.clone();
                        seg.hours = segment_ordinary_hours;

                        let saturday_result = calculate_saturday_pay(
                            &seg,
                            employee,
                            base_rate,
                            award_config,
                            step_number,
                        );

                        let mut pay_line = saturday_result.pay_line;
                        pay_line.shift_id = shift.id.clone();
                        all_pay_lines.push(pay_line);
                        all_audit_steps.push(saturday_result.audit_step);
                        step_number += 1;
                    }
                }
                DayType::Sunday => {
                    if segment_ordinary_hours > Decimal::ZERO {
                        // Create a segment for the ordinary hours
                        let mut seg = segment.clone();
                        seg.hours = segment_ordinary_hours;

                        let sunday_result = calculate_sunday_pay(
                            &seg,
                            employee,
                            base_rate,
                            award_config,
                            step_number,
                        );

                        let mut pay_line = sunday_result.pay_line;
                        pay_line.shift_id = shift.id.clone();
                        all_pay_lines.push(pay_line);
                        all_audit_steps.push(sunday_result.audit_step);
                        step_number += 1;
                    }
                }
            }
        }

        // Calculate overtime if applicable
        if overtime_detection.overtime_hours > Decimal::ZERO {
            // Determine the day type of the shift (use the primary shift date)
            let primary_day_type = get_day_type(shift.start_time);

            match primary_day_type {
                DayType::Weekday => {
                    let overtime_result = calculate_weekday_overtime(
                        overtime_detection.overtime_hours,
                        base_rate,
                        employee,
                        award_config,
                        shift.date,
                        &shift.id,
                        step_number,
                    );

                    all_pay_lines.extend(overtime_result.pay_lines);
                    let steps_count = overtime_result.audit_steps.len();
                    all_audit_steps.extend(overtime_result.audit_steps);
                    step_number += steps_count as u32;
                }
                DayType::Saturday => {
                    let overtime_result = calculate_weekend_overtime(
                        overtime_detection.overtime_hours,
                        base_rate,
                        employee,
                        award_config,
                        DayType::Saturday,
                        shift.date,
                        &shift.id,
                        step_number,
                    );

                    if let Some(pay_line) = overtime_result.pay_line {
                        all_pay_lines.push(pay_line);
                    }
                    if let Some(audit_step) = overtime_result.audit_step {
                        all_audit_steps.push(audit_step);
                        step_number += 1;
                    }
                }
                DayType::Sunday => {
                    let overtime_result = calculate_weekend_overtime(
                        overtime_detection.overtime_hours,
                        base_rate,
                        employee,
                        award_config,
                        DayType::Sunday,
                        shift.date,
                        &shift.id,
                        step_number,
                    );

                    if let Some(pay_line) = overtime_result.pay_line {
                        all_pay_lines.push(pay_line);
                    }
                    if let Some(audit_step) = overtime_result.audit_step {
                        all_audit_steps.push(audit_step);
                        step_number += 1;
                    }
                }
            }
        }
    }

    // Calculate laundry allowance
    let (laundry_per_shift, laundry_per_week) = config.get_allowance_rates(effective_date)?;
    let laundry_result = calculate_laundry_allowance(
        employee,
        shifts.len() as u32,
        laundry_per_shift,
        laundry_per_week,
        step_number,
    );
    all_audit_steps.push(laundry_result.audit_step);

    let allowances: Vec<AllowancePayment> = laundry_result.allowance.into_iter().collect();

    // Calculate totals
    let pay_lines_total: Decimal = all_pay_lines.iter().map(|pl| pl.amount).sum();
    let allowances_total: Decimal = allowances.iter().map(|a| a.amount).sum();
    let gross_pay = pay_lines_total + allowances_total;

    let ordinary_hours: Decimal = all_pay_lines
        .iter()
        .filter(|pl| matches!(pl.category, PayCategory::Ordinary | PayCategory::OrdinaryCasual))
        .map(|pl| pl.hours)
        .sum();

    let overtime_hours: Decimal = all_pay_lines
        .iter()
        .filter(|pl| matches!(pl.category, PayCategory::Overtime150 | PayCategory::Overtime200))
        .map(|pl| pl.hours)
        .sum();

    let penalty_hours: Decimal = all_pay_lines
        .iter()
        .filter(|pl| {
            matches!(
                pl.category,
                PayCategory::Saturday
                    | PayCategory::SaturdayCasual
                    | PayCategory::Sunday
                    | PayCategory::SundayCasual
            )
        })
        .map(|pl| pl.hours)
        .sum();

    let duration_us = start_time.elapsed().as_micros() as u64;

    Ok(CalculationResult {
        calculation_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        employee_id: employee.id.clone(),
        pay_period: pay_period.clone(),
        pay_lines: all_pay_lines,
        allowances,
        totals: PayTotals {
            gross_pay,
            ordinary_hours,
            overtime_hours,
            penalty_hours,
            allowances_total,
        },
        audit_trace: AuditTrace {
            steps: all_audit_steps,
            warnings: all_warnings,
            duration_us,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::request::{
        CalculationRequest, EmployeeRequest, PayPeriodRequest, ShiftRequest,
    };
    use crate::config::ConfigLoader;
    use crate::models::EmploymentType;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::{NaiveDate, NaiveDateTime};
    use tower::ServiceExt;

    fn create_test_state() -> AppState {
        let config = ConfigLoader::load("./config/ma000018").expect("Failed to load config");
        AppState::new(config)
    }

    fn make_datetime(date_str: &str, time_str: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(&format!("{} {}", date_str, time_str), "%Y-%m-%d %H:%M:%S")
            .unwrap()
    }

    fn make_date(date_str: &str) -> NaiveDate {
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap()
    }

    fn create_valid_request() -> CalculationRequest {
        CalculationRequest {
            employee: EmployeeRequest {
                id: "emp_001".to_string(),
                employment_type: EmploymentType::FullTime,
                classification_code: "dce_level_3".to_string(),
                date_of_birth: make_date("1985-03-15"),
                employment_start_date: make_date("2020-01-01"),
                base_hourly_rate: None,
                tags: vec![],
            },
            pay_period: PayPeriodRequest {
                start_date: make_date("2026-01-13"),
                end_date: make_date("2026-01-19"),
                public_holidays: vec![],
            },
            shifts: vec![ShiftRequest {
                id: "shift_001".to_string(),
                date: make_date("2026-01-13"),
                start_time: make_datetime("2026-01-13", "09:00:00"),
                end_time: make_datetime("2026-01-13", "17:00:00"),
                breaks: vec![],
            }],
        }
    }

    #[tokio::test]
    async fn test_api_001_valid_request_returns_200() {
        let state = create_test_state();
        let router = create_router(state);

        let request = create_valid_request();
        let body = serde_json::to_string(&request).unwrap();

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/calculate")
                    .header("Content-Type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Verify Content-Type header
        let content_type = response.headers().get("content-type").unwrap();
        assert_eq!(content_type, "application/json");

        // Verify response body is valid CalculationResult
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: CalculationResult = serde_json::from_slice(&body).unwrap();

        assert_eq!(result.employee_id, "emp_001");
        assert!(!result.pay_lines.is_empty());
        assert!(result.totals.gross_pay > Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_api_002_malformed_json_returns_400() {
        let state = create_test_state();
        let router = create_router(state);

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/calculate")
                    .header("Content-Type", "application/json")
                    .body(Body::from("{invalid json"))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: ApiError = serde_json::from_slice(&body).unwrap();

        assert_eq!(error.code, "MALFORMED_JSON");
    }

    #[tokio::test]
    async fn test_api_003_missing_employee_id_returns_400() {
        let state = create_test_state();
        let router = create_router(state);

        // JSON with missing employee.id field
        let body = r#"{
            "employee": {
                "employment_type": "full_time",
                "classification_code": "dce_level_3",
                "date_of_birth": "1985-03-15",
                "employment_start_date": "2020-01-01"
            },
            "pay_period": {
                "start_date": "2026-01-13",
                "end_date": "2026-01-19"
            },
            "shifts": []
        }"#;

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/calculate")
                    .header("Content-Type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: ApiError = serde_json::from_slice(&body).unwrap();

        // Check that error mentions the missing field
        // serde may say "missing field `id`" or similar
        assert!(
            error.message.contains("missing field") || error.message.to_lowercase().contains("id"),
            "Expected error message to mention missing field or id, got: {}",
            error.message
        );
    }

    #[tokio::test]
    async fn test_api_004_unknown_classification_returns_400() {
        let state = create_test_state();
        let router = create_router(state);

        let mut request = create_valid_request();
        request.employee.classification_code = "unknown".to_string();
        let body = serde_json::to_string(&request).unwrap();

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/calculate")
                    .header("Content-Type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let error: ApiError = serde_json::from_slice(&body).unwrap();

        assert_eq!(error.code, "CLASSIFICATION_NOT_FOUND");
    }

    #[tokio::test]
    async fn test_fulltime_weekday_8h_calculation() {
        let state = create_test_state();
        let router = create_router(state);

        let request = create_valid_request();
        let body = serde_json::to_string(&request).unwrap();

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/calculate")
                    .header("Content-Type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: CalculationResult = serde_json::from_slice(&body).unwrap();

        // 8 hours * $28.54 = $228.32
        use std::str::FromStr;
        assert_eq!(
            result.totals.gross_pay,
            Decimal::from_str("228.32").unwrap()
        );
        assert_eq!(
            result.totals.ordinary_hours,
            Decimal::from_str("8.0").unwrap()
        );
    }

    #[tokio::test]
    async fn test_casual_saturday_with_laundry() {
        let state = create_test_state();
        let router = create_router(state);

        let request = CalculationRequest {
            employee: EmployeeRequest {
                id: "emp_cas_001".to_string(),
                employment_type: EmploymentType::Casual,
                classification_code: "dce_level_3".to_string(),
                date_of_birth: make_date("1990-07-22"),
                employment_start_date: make_date("2024-06-01"),
                base_hourly_rate: None,
                tags: vec!["laundry_allowance".to_string()],
            },
            pay_period: PayPeriodRequest {
                start_date: make_date("2026-01-13"),
                end_date: make_date("2026-01-19"),
                public_holidays: vec![],
            },
            shifts: vec![ShiftRequest {
                id: "shift_001".to_string(),
                date: make_date("2026-01-17"), // Saturday
                start_time: make_datetime("2026-01-17", "09:00:00"),
                end_time: make_datetime("2026-01-17", "17:00:00"),
                breaks: vec![],
            }],
        };

        let body = serde_json::to_string(&request).unwrap();

        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/calculate")
                    .header("Content-Type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let result: CalculationResult = serde_json::from_slice(&body).unwrap();

        // Casual Saturday: 8h * $28.54 * 1.75 = $399.56
        // Plus laundry: $0.32
        // Total: $399.88
        use std::str::FromStr;
        assert_eq!(
            result.totals.gross_pay,
            Decimal::from_str("399.88").unwrap()
        );
        assert_eq!(result.allowances.len(), 1);
        assert_eq!(result.allowances[0].allowance_type, "laundry");
    }
}
