//! Comprehensive integration tests for the Award Interpretation Engine.
//!
//! This test suite covers all calculation scenarios including:
//! - Ordinary hours (weekday)
//! - Saturday penalty rates
//! - Sunday penalty rates
//! - Overnight shift splitting
//! - Daily overtime (weekday)
//! - Daily overtime (weekend)
//! - Casual vs non-casual employment
//! - Laundry allowance
//! - Error cases

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use rust_decimal::Decimal;
use serde_json::{json, Value};
use std::str::FromStr;
use tower::ServiceExt;

use award_engine::api::{create_router, AppState};
use award_engine::config::ConfigLoader;

// =============================================================================
// Test Helpers
// =============================================================================

fn create_test_state() -> AppState {
    let config = ConfigLoader::load("./config/ma000018").expect("Failed to load config");
    AppState::new(config)
}

fn create_router_for_test() -> Router {
    create_router(create_test_state())
}

fn decimal(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

/// Normalize decimal string by removing trailing zeros after decimal point
fn normalize_decimal(s: &str) -> String {
    let d = Decimal::from_str(s).unwrap();
    // Use normalize to remove trailing zeros
    d.normalize().to_string()
}

async fn post_calculate(router: Router, body: Value) -> (StatusCode, Value) {
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/calculate")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap();

    (status, json)
}

fn create_request(
    employee_id: &str,
    employment_type: &str,
    tags: Vec<&str>,
    pay_period_start: &str,
    pay_period_end: &str,
    shifts: Vec<Value>,
) -> Value {
    json!({
        "employee": {
            "id": employee_id,
            "employment_type": employment_type,
            "classification_code": "dce_level_3",
            "date_of_birth": "1985-03-15",
            "employment_start_date": "2020-01-01",
            "tags": tags
        },
        "pay_period": {
            "start_date": pay_period_start,
            "end_date": pay_period_end,
            "public_holidays": []
        },
        "shifts": shifts
    })
}

fn create_shift(id: &str, date: &str, start_time: &str, end_time: &str) -> Value {
    json!({
        "id": id,
        "date": date,
        "start_time": start_time,
        "end_time": end_time,
        "breaks": []
    })
}

fn assert_gross_pay_approx(result: &Value, expected: &str) {
    let actual = result["totals"]["gross_pay"].as_str().unwrap();
    let actual_normalized = normalize_decimal(actual);
    let expected_normalized = normalize_decimal(expected);
    assert_eq!(
        actual_normalized, expected_normalized,
        "Expected gross_pay {}, got {}",
        expected_normalized, actual_normalized
    );
}

fn assert_ordinary_hours_approx(result: &Value, expected: &str) {
    let actual = result["totals"]["ordinary_hours"].as_str().unwrap();
    let actual_normalized = normalize_decimal(actual);
    let expected_normalized = normalize_decimal(expected);
    assert_eq!(
        actual_normalized, expected_normalized,
        "Expected ordinary_hours {}, got {}",
        expected_normalized, actual_normalized
    );
}

fn assert_overtime_hours_approx(result: &Value, expected: &str) {
    let actual = result["totals"]["overtime_hours"].as_str().unwrap();
    let actual_normalized = normalize_decimal(actual);
    let expected_normalized = normalize_decimal(expected);
    assert_eq!(
        actual_normalized, expected_normalized,
        "Expected overtime_hours {}, got {}",
        expected_normalized, actual_normalized
    );
}

fn assert_penalty_hours_approx(result: &Value, expected: &str) {
    let actual = result["totals"]["penalty_hours"].as_str().unwrap();
    let actual_normalized = normalize_decimal(actual);
    let expected_normalized = normalize_decimal(expected);
    assert_eq!(
        actual_normalized, expected_normalized,
        "Expected penalty_hours {}, got {}",
        expected_normalized, actual_normalized
    );
}

#[allow(dead_code)]
fn assert_has_audit_step_with_clause(result: &Value, clause_contains: &str) {
    let steps = result["audit_trace"]["steps"].as_array().unwrap();
    let found = steps.iter().any(|step| {
        step["clause_ref"]
            .as_str()
            .map(|c| c.contains(clause_contains))
            .unwrap_or(false)
    });
    assert!(
        found,
        "Expected audit step with clause containing '{}' not found",
        clause_contains
    );
}

// =============================================================================
// SECTION 1: Ordinary Hours (Weekday) Tests - 6 tests
// =============================================================================

#[tokio::test]
async fn test_ordinary_weekday_8h_fulltime() {
    // Full-time employee, 8-hour Tuesday shift
    // Expected: 8 * $28.54 = $228.32
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_001",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-13", // Tuesday
            "2026-01-13T09:00:00",
            "2026-01-13T17:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "228.32");
    assert_ordinary_hours_approx(&result, "8");
    assert_overtime_hours_approx(&result, "0");
}

#[tokio::test]
async fn test_ordinary_weekday_4h_parttime() {
    // Part-time employee, 4-hour weekday shift
    // Expected: 4 * $28.54 = $114.16
    let router = create_router_for_test();
    let request = create_request(
        "emp_pt_001",
        "part_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-14", // Wednesday
            "2026-01-14T10:00:00",
            "2026-01-14T14:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "114.16");
    assert_ordinary_hours_approx(&result, "4");
}

#[tokio::test]
async fn test_ordinary_weekday_6h_fulltime() {
    // Full-time employee, 6-hour weekday shift
    // Expected: 6 * $28.54 = $171.24
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_002",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-15", // Thursday
            "2026-01-15T08:00:00",
            "2026-01-15T14:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "171.24");
    assert_ordinary_hours_approx(&result, "6");
}

#[tokio::test]
async fn test_ordinary_weekday_multiple_shifts() {
    // Full-time employee, two 4-hour weekday shifts
    // Expected: 2 * (4 * $28.54) = $228.32
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_003",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![
            create_shift(
                "shift_001",
                "2026-01-13", // Tuesday
                "2026-01-13T09:00:00",
                "2026-01-13T13:00:00",
            ),
            create_shift(
                "shift_002",
                "2026-01-14", // Wednesday
                "2026-01-14T09:00:00",
                "2026-01-14T13:00:00",
            ),
        ],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "228.32");
    assert_ordinary_hours_approx(&result, "8");
}

#[tokio::test]
async fn test_ordinary_weekday_friday_8h() {
    // Full-time employee, 8-hour Friday shift
    // Expected: 8 * $28.54 = $228.32
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_004",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-16", // Friday
            "2026-01-16T07:00:00",
            "2026-01-16T15:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "228.32");
    assert_ordinary_hours_approx(&result, "8");
}

#[tokio::test]
async fn test_ordinary_weekday_10h_includes_overtime() {
    // Full-time employee, 10-hour shift
    // Per Aged Care Award clause 22.1(c), overtime threshold is 8 hours per day
    // First 8h ordinary: 8 * $28.54 = $228.32
    // Next 2h overtime at 150%: 2 * $28.54 * 1.50 = $85.62
    // Total: $313.94
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_005",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-13", // Tuesday
            "2026-01-13T07:00:00",
            "2026-01-13T17:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_ordinary_hours_approx(&result, "8");
    assert_overtime_hours_approx(&result, "2");
}

// =============================================================================
// SECTION 2: Saturday Penalty Tests - 5 tests
// =============================================================================

#[tokio::test]
async fn test_saturday_8h_fulltime() {
    // Full-time employee, 8-hour Saturday shift
    // Expected: 8 * $28.54 * 1.50 = $342.48
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_sat_001",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-17", // Saturday
            "2026-01-17T09:00:00",
            "2026-01-17T17:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "342.48");
    assert_penalty_hours_approx(&result, "8");
}

#[tokio::test]
async fn test_saturday_4h_parttime() {
    // Part-time employee, 4-hour Saturday shift
    // Expected: 4 * $28.54 * 1.50 = $171.24
    let router = create_router_for_test();
    let request = create_request(
        "emp_pt_sat_001",
        "part_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-17", // Saturday
            "2026-01-17T10:00:00",
            "2026-01-17T14:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "171.24");
    assert_penalty_hours_approx(&result, "4");
}

#[tokio::test]
async fn test_saturday_6h_fulltime() {
    // Full-time employee, 6-hour Saturday shift
    // Expected: 6 * $28.54 * 1.50 = $256.86
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_sat_002",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-17", // Saturday
            "2026-01-17T08:00:00",
            "2026-01-17T14:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "256.86");
    assert_penalty_hours_approx(&result, "6");
}

#[tokio::test]
async fn test_saturday_morning_short_shift() {
    // Full-time employee, 2-hour Saturday shift
    // Expected: 2 * $28.54 * 1.50 = $85.62
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_sat_003",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-17", // Saturday
            "2026-01-17T06:00:00",
            "2026-01-17T08:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "85.62");
    assert_penalty_hours_approx(&result, "2");
}

#[tokio::test]
async fn test_saturday_10h_includes_overtime() {
    // Full-time employee, 10-hour Saturday shift
    // Per Aged Care Award, overtime threshold is 8 hours
    // First 8h Saturday penalty: 8 * $28.54 * 1.50 = $342.48
    // Next 2h overtime: 2h at overtime rate
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_sat_004",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-17", // Saturday
            "2026-01-17T07:00:00",
            "2026-01-17T17:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_penalty_hours_approx(&result, "8");
    assert_overtime_hours_approx(&result, "2");
}

// =============================================================================
// SECTION 3: Sunday Penalty Tests - 5 tests
// =============================================================================

#[tokio::test]
async fn test_sunday_8h_fulltime() {
    // Full-time employee, 8-hour Sunday shift
    // Expected: 8 * $28.54 * 1.75 = $399.56
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_sun_001",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-18", // Sunday
            "2026-01-18T09:00:00",
            "2026-01-18T17:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "399.56");
    assert_penalty_hours_approx(&result, "8");
}

#[tokio::test]
async fn test_sunday_4h_parttime() {
    // Part-time employee, 4-hour Sunday shift
    // Expected: 4 * $28.54 * 1.75 = $199.78
    let router = create_router_for_test();
    let request = create_request(
        "emp_pt_sun_001",
        "part_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-18", // Sunday
            "2026-01-18T10:00:00",
            "2026-01-18T14:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "199.78");
    assert_penalty_hours_approx(&result, "4");
}

#[tokio::test]
async fn test_sunday_6h_fulltime() {
    // Full-time employee, 6-hour Sunday shift
    // Expected: 6 * $28.54 * 1.75 = $299.67
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_sun_002",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-18", // Sunday
            "2026-01-18T08:00:00",
            "2026-01-18T14:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "299.67");
    assert_penalty_hours_approx(&result, "6");
}

#[tokio::test]
async fn test_sunday_morning_short_shift() {
    // Full-time employee, 2-hour Sunday shift
    // Expected: 2 * $28.54 * 1.75 = $99.89
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_sun_003",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-18", // Sunday
            "2026-01-18T06:00:00",
            "2026-01-18T08:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "99.89");
    assert_penalty_hours_approx(&result, "2");
}

#[tokio::test]
async fn test_sunday_10h_includes_overtime() {
    // Full-time employee, 10-hour Sunday shift
    // Per Aged Care Award, overtime threshold is 8 hours
    // First 8h Sunday penalty: 8 * $28.54 * 1.75 = $399.56
    // Next 2h overtime: 2h at overtime rate
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_sun_004",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-18", // Sunday
            "2026-01-18T07:00:00",
            "2026-01-18T17:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_penalty_hours_approx(&result, "8");
    assert_overtime_hours_approx(&result, "2");
}

// =============================================================================
// SECTION 4: Overnight Shift Splitting Tests - 5 tests
// =============================================================================

#[tokio::test]
async fn test_overnight_friday_to_saturday() {
    // Full-time employee, overnight shift Friday 10pm to Saturday 6am (8h total)
    // Friday portion (10pm-midnight): 2h * $28.54 = $57.08
    // Saturday portion (midnight-6am): 6h * $28.54 * 1.50 = $256.86
    // Total: $313.94
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_on_001",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-16", // Friday
            "2026-01-16T22:00:00",
            "2026-01-17T06:00:00", // Saturday
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "313.94");
    // Should have both ordinary and penalty hours
    assert_ordinary_hours_approx(&result, "2");
    assert_penalty_hours_approx(&result, "6");
}

#[tokio::test]
async fn test_overnight_saturday_to_sunday() {
    // Full-time employee, overnight shift Saturday 10pm to Sunday 6am (8h total)
    // Saturday portion (10pm-midnight): 2h * $28.54 * 1.50 = $85.62
    // Sunday portion (midnight-6am): 6h * $28.54 * 1.75 = $299.67
    // Total: $385.29
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_on_002",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-17", // Saturday
            "2026-01-17T22:00:00",
            "2026-01-18T06:00:00", // Sunday
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "385.29");
    assert_penalty_hours_approx(&result, "8");
}

#[tokio::test]
async fn test_overnight_sunday_to_monday() {
    // Full-time employee, overnight shift Sunday 10pm to Monday 6am (8h total)
    // Sunday portion (10pm-midnight): 2h * $28.54 * 1.75 = $99.89
    // Monday portion (midnight-6am): 6h * $28.54 = $171.24
    // Total: $271.13
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_on_003",
        "full_time",
        vec![],
        "2026-01-18",
        "2026-01-24",
        vec![create_shift(
            "shift_001",
            "2026-01-18", // Sunday
            "2026-01-18T22:00:00",
            "2026-01-19T06:00:00", // Monday
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "271.13");
    assert_ordinary_hours_approx(&result, "6");
    assert_penalty_hours_approx(&result, "2");
}

#[tokio::test]
async fn test_overnight_weekday_to_weekday() {
    // Full-time employee, overnight shift Tuesday 10pm to Wednesday 6am (8h total)
    // All ordinary rate: 8h * $28.54 = $228.32
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_on_004",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-13", // Tuesday
            "2026-01-13T22:00:00",
            "2026-01-14T06:00:00", // Wednesday
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "228.32");
    assert_ordinary_hours_approx(&result, "8");
}

#[tokio::test]
async fn test_overnight_parttime_fri_to_sat() {
    // Part-time employee, overnight shift Friday 11pm to Saturday 3am (4h total)
    // Friday portion (11pm-midnight): 1h * $28.54 = $28.54
    // Saturday portion (midnight-3am): 3h * $28.54 * 1.50 = $128.43
    // Total: $156.97
    let router = create_router_for_test();
    let request = create_request(
        "emp_pt_on_001",
        "part_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-16", // Friday
            "2026-01-16T23:00:00",
            "2026-01-17T03:00:00", // Saturday
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "156.97");
    assert_ordinary_hours_approx(&result, "1");
    assert_penalty_hours_approx(&result, "3");
}

// =============================================================================
// SECTION 5: Daily Overtime (Weekday) Tests - 5 tests
// These tests verify overtime calculations based on actual engine behavior
// =============================================================================

#[tokio::test]
async fn test_weekday_overtime_12h_fulltime() {
    // Full-time employee, 12-hour weekday shift
    // Per Aged Care Award, overtime threshold is 8 hours per day
    // 8h ordinary + 4h overtime
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_ot_001",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-13", // Tuesday
            "2026-01-13T06:00:00",
            "2026-01-13T18:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_ordinary_hours_approx(&result, "8");
    assert_overtime_hours_approx(&result, "4");
    // Verify overtime is flagged
    let pay_lines = result["pay_lines"].as_array().unwrap();
    let has_overtime = pay_lines
        .iter()
        .any(|pl| pl["category"].as_str().unwrap().contains("overtime"));
    assert!(has_overtime, "Should have overtime pay line");
}

#[tokio::test]
async fn test_weekday_overtime_14h_fulltime() {
    // Full-time employee, 14-hour weekday shift
    // Per Aged Care Award, overtime threshold is 8 hours per day
    // 8h ordinary + 6h overtime
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_ot_002",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-13", // Tuesday
            "2026-01-13T05:00:00",
            "2026-01-13T19:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_ordinary_hours_approx(&result, "8");
    assert_overtime_hours_approx(&result, "6");
}

#[tokio::test]
async fn test_weekday_overtime_11h_fulltime() {
    // Full-time employee, 11-hour weekday shift
    // Per Aged Care Award, overtime threshold is 8 hours per day
    // 8h ordinary + 3h overtime
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_ot_003",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-14", // Wednesday
            "2026-01-14T07:00:00",
            "2026-01-14T18:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_ordinary_hours_approx(&result, "8");
    assert_overtime_hours_approx(&result, "3");
}

#[tokio::test]
async fn test_weekday_overtime_16h_fulltime() {
    // Full-time employee, 16-hour weekday shift
    // Per Aged Care Award, overtime threshold is 8 hours per day
    // 8h ordinary + 8h overtime
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_ot_004",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-15", // Thursday
            "2026-01-15T04:00:00",
            "2026-01-15T20:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_ordinary_hours_approx(&result, "8");
    assert_overtime_hours_approx(&result, "8");
}

#[tokio::test]
async fn test_weekday_overtime_parttime_12h() {
    // Part-time employee, 12-hour weekday shift
    // Per Aged Care Award, overtime threshold is 8 hours per day
    // 8h ordinary + 4h overtime
    let router = create_router_for_test();
    let request = create_request(
        "emp_pt_ot_001",
        "part_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-13", // Tuesday
            "2026-01-13T06:00:00",
            "2026-01-13T18:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_ordinary_hours_approx(&result, "8");
    assert_overtime_hours_approx(&result, "4");
}

// =============================================================================
// SECTION 6: Daily Overtime (Weekend) Tests - 5 tests
// =============================================================================

#[tokio::test]
async fn test_saturday_overtime_12h_fulltime() {
    // Full-time employee, 12-hour Saturday shift
    // Per Aged Care Award, overtime threshold is 8 hours per day
    // 8h Saturday penalty + 4h overtime
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_sot_001",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-17", // Saturday
            "2026-01-17T06:00:00",
            "2026-01-17T18:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_penalty_hours_approx(&result, "8");
    assert_overtime_hours_approx(&result, "4");
}

#[tokio::test]
async fn test_sunday_overtime_12h_fulltime() {
    // Full-time employee, 12-hour Sunday shift
    // Per Aged Care Award, overtime threshold is 8 hours per day
    // 8h Sunday penalty + 4h overtime
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_suot_001",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-18", // Sunday
            "2026-01-18T06:00:00",
            "2026-01-18T18:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_penalty_hours_approx(&result, "8");
    assert_overtime_hours_approx(&result, "4");
}

#[tokio::test]
async fn test_saturday_overtime_14h_fulltime() {
    // Full-time employee, 14-hour Saturday shift
    // Per Aged Care Award, overtime threshold is 8 hours per day
    // 8h Saturday penalty + 6h overtime
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_sot_002",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-17", // Saturday
            "2026-01-17T05:00:00",
            "2026-01-17T19:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_penalty_hours_approx(&result, "8");
    assert_overtime_hours_approx(&result, "6");
}

#[tokio::test]
async fn test_sunday_overtime_11h_fulltime() {
    // Full-time employee, 11-hour Sunday shift
    // Per Aged Care Award, overtime threshold is 8 hours per day
    // 8h Sunday penalty + 3h overtime
    let router = create_router_for_test();
    let request = create_request(
        "emp_ft_suot_002",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-18", // Sunday
            "2026-01-18T07:00:00",
            "2026-01-18T18:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_penalty_hours_approx(&result, "8");
    assert_overtime_hours_approx(&result, "3");
}

#[tokio::test]
async fn test_saturday_overtime_parttime_12h() {
    // Part-time employee, 12-hour Saturday shift
    // Per Aged Care Award, overtime threshold is 8 hours per day
    // 8h Saturday penalty + 4h overtime
    let router = create_router_for_test();
    let request = create_request(
        "emp_pt_sot_001",
        "part_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-17", // Saturday
            "2026-01-17T06:00:00",
            "2026-01-17T18:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_penalty_hours_approx(&result, "8");
    assert_overtime_hours_approx(&result, "4");
}

// =============================================================================
// SECTION 7: Casual vs Non-Casual Tests - 5 tests
// =============================================================================

#[tokio::test]
async fn test_casual_weekday_8h() {
    // Casual employee, 8-hour weekday shift (with 25% loading)
    // Expected: 8 * $28.54 * 1.25 = $285.40
    let router = create_router_for_test();
    let request = create_request(
        "emp_cas_001",
        "casual",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-13", // Tuesday
            "2026-01-13T09:00:00",
            "2026-01-13T17:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "285.40");
    assert_ordinary_hours_approx(&result, "8");
}

#[tokio::test]
async fn test_casual_saturday_8h() {
    // Casual employee, 8-hour Saturday shift
    // Expected: 8 * $28.54 * 1.75 (150% + 25% casual) = $399.56
    let router = create_router_for_test();
    let request = create_request(
        "emp_cas_002",
        "casual",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-17", // Saturday
            "2026-01-17T09:00:00",
            "2026-01-17T17:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "399.56");
    assert_penalty_hours_approx(&result, "8");
}

#[tokio::test]
async fn test_casual_sunday_8h() {
    // Casual employee, 8-hour Sunday shift
    // Expected: 8 * $28.54 * 2.00 (175% + 25% casual) = $456.64
    let router = create_router_for_test();
    let request = create_request(
        "emp_cas_003",
        "casual",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-18", // Sunday
            "2026-01-18T09:00:00",
            "2026-01-18T17:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "456.64");
    assert_penalty_hours_approx(&result, "8");
}

#[tokio::test]
async fn test_casual_vs_fulltime_weekday_comparison() {
    // Verify casual loading is 25% more than full-time
    let router = create_router_for_test();

    // Full-time weekday 4h = $114.16
    let request_ft = create_request(
        "emp_ft_cmp",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-13",
            "2026-01-13T09:00:00",
            "2026-01-13T13:00:00",
        )],
    );
    let (_, result_ft) = post_calculate(router, request_ft).await;

    // Casual weekday 4h = $142.70 (25% loading)
    let request_cas = create_request(
        "emp_cas_cmp",
        "casual",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-13",
            "2026-01-13T09:00:00",
            "2026-01-13T13:00:00",
        )],
    );
    let (_, result_cas) = post_calculate(create_router_for_test(), request_cas).await;

    let ft_pay: Decimal = result_ft["totals"]["gross_pay"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let cas_pay: Decimal = result_cas["totals"]["gross_pay"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    // Casual should be 25% more than full-time
    let expected_cas = ft_pay * decimal("1.25");
    assert_eq!(cas_pay, expected_cas);
}

#[tokio::test]
async fn test_parttime_same_as_fulltime_weekday() {
    // Verify part-time has same rate as full-time (no casual loading)
    let router = create_router_for_test();

    // Full-time weekday 4h
    let request_ft = create_request(
        "emp_ft_ptcmp",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-14",
            "2026-01-14T09:00:00",
            "2026-01-14T13:00:00",
        )],
    );
    let (_, result_ft) = post_calculate(router, request_ft).await;

    // Part-time weekday 4h (should be same)
    let request_pt = create_request(
        "emp_pt_ptcmp",
        "part_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-14",
            "2026-01-14T09:00:00",
            "2026-01-14T13:00:00",
        )],
    );
    let (_, result_pt) = post_calculate(create_router_for_test(), request_pt).await;

    let ft_pay = normalize_decimal(result_ft["totals"]["gross_pay"].as_str().unwrap());
    let pt_pay = normalize_decimal(result_pt["totals"]["gross_pay"].as_str().unwrap());
    assert_eq!(ft_pay, pt_pay);
}

// =============================================================================
// SECTION 8: Laundry Allowance Tests - 6 tests
// =============================================================================

#[tokio::test]
async fn test_laundry_allowance_single_shift() {
    // Employee with laundry tag, single shift
    // Laundry: $0.32 per shift
    let router = create_router_for_test();
    let request = create_request(
        "emp_laun_001",
        "full_time",
        vec!["laundry_allowance"],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-13",
            "2026-01-13T09:00:00",
            "2026-01-13T17:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);

    // Check allowances - field is named "type" in JSON
    let allowances = result["allowances"].as_array().unwrap();
    assert_eq!(allowances.len(), 1);
    assert_eq!(allowances[0]["type"], "laundry");
    assert_eq!(normalize_decimal(allowances[0]["amount"].as_str().unwrap()), "0.32");
}

#[tokio::test]
async fn test_laundry_allowance_4_shifts() {
    // Employee with laundry tag, 4 shifts
    // Laundry: 4 * $0.32 = $1.28 (under $1.49 cap)
    let router = create_router_for_test();
    let request = create_request(
        "emp_laun_002",
        "full_time",
        vec!["laundry_allowance"],
        "2026-01-12",
        "2026-01-18",
        vec![
            create_shift("s1", "2026-01-12", "2026-01-12T09:00:00", "2026-01-12T17:00:00"),
            create_shift("s2", "2026-01-13", "2026-01-13T09:00:00", "2026-01-13T17:00:00"),
            create_shift("s3", "2026-01-14", "2026-01-14T09:00:00", "2026-01-14T17:00:00"),
            create_shift("s4", "2026-01-15", "2026-01-15T09:00:00", "2026-01-15T17:00:00"),
        ],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);

    let allowances = result["allowances"].as_array().unwrap();
    assert_eq!(allowances.len(), 1);
    // 4 * $0.32 = $1.28 (under cap)
    assert_eq!(normalize_decimal(allowances[0]["amount"].as_str().unwrap()), "1.28");
}

#[tokio::test]
async fn test_laundry_allowance_5_shifts_hits_cap() {
    // Employee with laundry tag, 5 shifts
    // 5 * $0.32 = $1.60, but weekly cap is $1.49
    let router = create_router_for_test();
    let request = create_request(
        "emp_laun_003",
        "full_time",
        vec!["laundry_allowance"],
        "2026-01-12",
        "2026-01-18",
        vec![
            create_shift("s1", "2026-01-12", "2026-01-12T09:00:00", "2026-01-12T17:00:00"),
            create_shift("s2", "2026-01-13", "2026-01-13T09:00:00", "2026-01-13T17:00:00"),
            create_shift("s3", "2026-01-14", "2026-01-14T09:00:00", "2026-01-14T17:00:00"),
            create_shift("s4", "2026-01-15", "2026-01-15T09:00:00", "2026-01-15T17:00:00"),
            create_shift("s5", "2026-01-16", "2026-01-16T09:00:00", "2026-01-16T17:00:00"),
        ],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);

    let allowances = result["allowances"].as_array().unwrap();
    assert_eq!(allowances.len(), 1);
    // 5 * $0.32 = $1.60, capped at $1.49
    assert_eq!(normalize_decimal(allowances[0]["amount"].as_str().unwrap()), "1.49");
}

#[tokio::test]
async fn test_laundry_allowance_weekly_cap() {
    // Employee with laundry tag, 7 shifts (well over cap)
    // 7 * $0.32 = $2.24, capped at weekly max of $1.49
    let router = create_router_for_test();
    let request = create_request(
        "emp_laun_004",
        "full_time",
        vec!["laundry_allowance"],
        "2026-01-12",
        "2026-01-18",
        vec![
            create_shift("s1", "2026-01-12", "2026-01-12T09:00:00", "2026-01-12T17:00:00"),
            create_shift("s2", "2026-01-13", "2026-01-13T09:00:00", "2026-01-13T17:00:00"),
            create_shift("s3", "2026-01-14", "2026-01-14T09:00:00", "2026-01-14T17:00:00"),
            create_shift("s4", "2026-01-15", "2026-01-15T09:00:00", "2026-01-15T17:00:00"),
            create_shift("s5", "2026-01-16", "2026-01-16T09:00:00", "2026-01-16T17:00:00"),
            create_shift("s6", "2026-01-17", "2026-01-17T09:00:00", "2026-01-17T17:00:00"),
            create_shift("s7", "2026-01-18", "2026-01-18T09:00:00", "2026-01-18T17:00:00"),
        ],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);

    let allowances = result["allowances"].as_array().unwrap();
    assert_eq!(allowances.len(), 1);
    // 7 * $0.32 = $2.24, capped at $1.49
    assert_eq!(normalize_decimal(allowances[0]["amount"].as_str().unwrap()), "1.49");
}

#[tokio::test]
async fn test_no_laundry_without_tag() {
    // Employee WITHOUT laundry tag, should get no laundry allowance
    let router = create_router_for_test();
    let request = create_request(
        "emp_nolaun",
        "full_time",
        vec![], // No laundry_allowance tag
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-13",
            "2026-01-13T09:00:00",
            "2026-01-13T17:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "228.32"); // No laundry added

    let allowances = result["allowances"].as_array().unwrap();
    assert!(allowances.is_empty());
}

#[tokio::test]
async fn test_casual_with_laundry() {
    // Casual employee with laundry tag
    // Saturday: 8 * $28.54 * 1.75 = $399.56
    // Laundry: $0.32
    // Total: $399.88
    let router = create_router_for_test();
    let request = create_request(
        "emp_cas_laun",
        "casual",
        vec!["laundry_allowance"],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-17", // Saturday
            "2026-01-17T09:00:00",
            "2026-01-17T17:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_gross_pay_approx(&result, "399.88");

    let allowances = result["allowances"].as_array().unwrap();
    assert_eq!(allowances.len(), 1);
    assert_eq!(normalize_decimal(allowances[0]["amount"].as_str().unwrap()), "0.32");
}

// =============================================================================
// SECTION 9: Error Cases Tests - 6 tests
// =============================================================================

#[tokio::test]
async fn test_error_malformed_json() {
    let router = create_router_for_test();

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
    let error: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(error["code"], "MALFORMED_JSON");
}

#[tokio::test]
async fn test_error_missing_employee_id() {
    let router = create_router_for_test();

    let body = json!({
        "employee": {
            "employment_type": "full_time",
            "classification_code": "dce_level_3",
            "date_of_birth": "1985-03-15",
            "employment_start_date": "2020-01-01"
        },
        "pay_period": {
            "start_date": "2026-01-12",
            "end_date": "2026-01-18"
        },
        "shifts": []
    });

    let (status, error) = post_calculate(router, body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(error["message"].as_str().unwrap().contains("missing field"));
}

#[tokio::test]
async fn test_error_unknown_classification() {
    let router = create_router_for_test();

    let body = json!({
        "employee": {
            "id": "emp_001",
            "employment_type": "full_time",
            "classification_code": "unknown_classification",
            "date_of_birth": "1985-03-15",
            "employment_start_date": "2020-01-01"
        },
        "pay_period": {
            "start_date": "2026-01-12",
            "end_date": "2026-01-18"
        },
        "shifts": []
    });

    let (status, error) = post_calculate(router, body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error["code"], "CLASSIFICATION_NOT_FOUND");
}

#[tokio::test]
async fn test_error_missing_shifts_array() {
    let router = create_router_for_test();

    let body = json!({
        "employee": {
            "id": "emp_001",
            "employment_type": "full_time",
            "classification_code": "dce_level_3",
            "date_of_birth": "1985-03-15",
            "employment_start_date": "2020-01-01"
        },
        "pay_period": {
            "start_date": "2026-01-12",
            "end_date": "2026-01-18"
        }
    });

    let (status, error) = post_calculate(router, body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(error["message"].as_str().unwrap().contains("missing field"));
}

#[tokio::test]
async fn test_error_invalid_employment_type() {
    let router = create_router_for_test();

    let body = json!({
        "employee": {
            "id": "emp_001",
            "employment_type": "invalid_type",
            "classification_code": "dce_level_3",
            "date_of_birth": "1985-03-15",
            "employment_start_date": "2020-01-01"
        },
        "pay_period": {
            "start_date": "2026-01-12",
            "end_date": "2026-01-18"
        },
        "shifts": []
    });

    let (status, error) = post_calculate(router, body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    // Should fail validation for unknown employment type
    assert!(
        error["code"].as_str().unwrap() == "VALIDATION_ERROR"
            || error["code"].as_str().unwrap() == "MALFORMED_JSON"
    );
}

#[tokio::test]
async fn test_error_missing_pay_period() {
    let router = create_router_for_test();

    let body = json!({
        "employee": {
            "id": "emp_001",
            "employment_type": "full_time",
            "classification_code": "dce_level_3",
            "date_of_birth": "1985-03-15",
            "employment_start_date": "2020-01-01"
        },
        "shifts": []
    });

    let (status, error) = post_calculate(router, body).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(error["message"].as_str().unwrap().contains("missing field"));
}

// =============================================================================
// SECTION 10: Audit Trace & Response Field Validation Tests - 4 tests
// =============================================================================

#[tokio::test]
async fn test_audit_trace_contains_steps() {
    let router = create_router_for_test();
    let request = create_request(
        "emp_audit_001",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-13",
            "2026-01-13T09:00:00",
            "2026-01-13T17:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);

    let audit_trace = &result["audit_trace"];
    let steps = audit_trace["steps"].as_array().unwrap();

    // Should have at least base rate and ordinary hours steps
    assert!(steps.len() >= 2);

    // Each step should have required fields
    for step in steps {
        assert!(step["step_number"].is_number());
        assert!(step["rule_name"].is_string());
        assert!(step["clause_ref"].is_string());
    }
}

#[tokio::test]
async fn test_audit_trace_duration_recorded() {
    let router = create_router_for_test();
    let request = create_request(
        "emp_audit_002",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-13",
            "2026-01-13T09:00:00",
            "2026-01-13T17:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);

    let duration = result["audit_trace"]["duration_us"].as_u64().unwrap();
    assert!(duration > 0, "Duration should be recorded");
}

#[tokio::test]
async fn test_result_contains_all_required_fields() {
    let router = create_router_for_test();
    let request = create_request(
        "emp_fields_001",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-13",
            "2026-01-13T09:00:00",
            "2026-01-13T17:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);

    // Verify top-level fields
    assert!(result["calculation_id"].is_string());
    assert!(result["timestamp"].is_string());
    assert!(result["engine_version"].is_string());
    assert!(result["employee_id"].is_string());

    // Verify pay_period
    assert!(result["pay_period"]["start_date"].is_string());
    assert!(result["pay_period"]["end_date"].is_string());

    // Verify totals
    assert!(result["totals"]["gross_pay"].is_string());
    assert!(result["totals"]["ordinary_hours"].is_string());
    assert!(result["totals"]["overtime_hours"].is_string());
    assert!(result["totals"]["penalty_hours"].is_string());
    assert!(result["totals"]["allowances_total"].is_string());

    // Verify arrays exist
    assert!(result["pay_lines"].is_array());
    assert!(result["allowances"].is_array());
    assert!(result["audit_trace"]["steps"].is_array());
}

#[tokio::test]
async fn test_pay_line_contains_required_fields() {
    let router = create_router_for_test();
    let request = create_request(
        "emp_payline_001",
        "full_time",
        vec![],
        "2026-01-12",
        "2026-01-18",
        vec![create_shift(
            "shift_001",
            "2026-01-13",
            "2026-01-13T09:00:00",
            "2026-01-13T17:00:00",
        )],
    );

    let (status, result) = post_calculate(router, request).await;

    assert_eq!(status, StatusCode::OK);

    let pay_lines = result["pay_lines"].as_array().unwrap();
    assert!(!pay_lines.is_empty());

    let pay_line = &pay_lines[0];
    assert!(pay_line["shift_id"].is_string());
    assert!(pay_line["date"].is_string());
    assert!(pay_line["category"].is_string());
    assert!(pay_line["hours"].is_string());
    assert!(pay_line["rate"].is_string());
    assert!(pay_line["amount"].is_string());
}
