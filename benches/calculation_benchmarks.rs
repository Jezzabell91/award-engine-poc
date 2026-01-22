//! Performance benchmarks for the Award Interpretation Engine.
//!
//! This benchmark suite verifies that the calculation engine meets performance targets:
//! - Single shift calculation: < 100μs mean
//! - Single timesheet with 1 shift: < 1ms mean
//! - Timesheet with 14 shifts: < 5ms mean
//! - Batch of 100 timesheets: < 100ms mean
//! - Batch of 1000 timesheets: < 500ms mean
//!
//! Run with: `cargo bench`
//! HTML reports are generated in `target/criterion/`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use award_engine::api::{create_router, AppState, CalculationRequest};
use award_engine::config::ConfigLoader;

use axum::{body::Body, http::Request};
use tower::ServiceExt;

/// Creates a test state with loaded configuration.
fn create_test_state() -> AppState {
    let config = ConfigLoader::load("./config/ma000018").expect("Failed to load config");
    AppState::new(config)
}

/// Creates a single 8-hour shift for a given date.
fn create_single_shift(date: &str) -> serde_json::Value {
    serde_json::json!({
        "id": format!("shift_{}", date),
        "date": date,
        "start_time": format!("{}T09:00:00", date),
        "end_time": format!("{}T17:00:00", date),
        "breaks": []
    })
}

/// Creates a calculation request with a specified number of shifts.
fn create_request_with_shifts(shift_count: usize) -> CalculationRequest {
    // Generate dates for 2 weeks starting from a Monday
    let base_dates = [
        "2026-01-13", // Monday
        "2026-01-14", // Tuesday
        "2026-01-15", // Wednesday
        "2026-01-16", // Thursday
        "2026-01-17", // Friday (or Saturday - but we'll use weekday for consistency)
        "2026-01-19", // Monday
        "2026-01-20", // Tuesday
        "2026-01-21", // Wednesday
        "2026-01-22", // Thursday
        "2026-01-23", // Friday
        "2026-01-26", // Monday
        "2026-01-27", // Tuesday
        "2026-01-28", // Wednesday
        "2026-01-29", // Thursday
    ];

    let shifts: Vec<serde_json::Value> = base_dates
        .iter()
        .cycle()
        .take(shift_count)
        .enumerate()
        .map(|(i, date)| {
            serde_json::json!({
                "id": format!("shift_{:03}", i + 1),
                "date": date,
                "start_time": format!("{}T09:00:00", date),
                "end_time": format!("{}T17:00:00", date),
                "breaks": []
            })
        })
        .collect();

    let request_json = serde_json::json!({
        "employee": {
            "id": "emp_bench_001",
            "employment_type": "full_time",
            "classification_code": "dce_level_3",
            "date_of_birth": "1985-03-15",
            "employment_start_date": "2020-01-01",
            "tags": []
        },
        "pay_period": {
            "start_date": "2026-01-13",
            "end_date": "2026-01-30",
            "public_holidays": []
        },
        "shifts": shifts
    });

    serde_json::from_value(request_json).expect("Failed to create request")
}

/// Benchmark: Single shift calculation.
///
/// Target: < 100μs mean
fn bench_single_shift(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let state = create_test_state();
    let router = create_router(state);
    let request = create_request_with_shifts(1);
    let body = serde_json::to_string(&request).unwrap();

    c.bench_function("single_shift", |b| {
        b.to_async(&rt).iter(|| async {
            let router = router.clone();
            let response = router
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/calculate")
                        .header("Content-Type", "application/json")
                        .body(Body::from(body.clone()))
                        .unwrap(),
                )
                .await
                .unwrap();
            black_box(response)
        })
    });
}

/// Benchmark: Timesheet with 14 shifts (2-week period).
///
/// Target: < 5ms mean
fn bench_timesheet_14_shifts(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let state = create_test_state();
    let router = create_router(state);
    let request = create_request_with_shifts(14);
    let body = serde_json::to_string(&request).unwrap();

    c.bench_function("timesheet_14_shifts", |b| {
        b.to_async(&rt).iter(|| async {
            let router = router.clone();
            let response = router
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/calculate")
                        .header("Content-Type", "application/json")
                        .body(Body::from(body.clone()))
                        .unwrap(),
                )
                .await
                .unwrap();
            black_box(response)
        })
    });
}

/// Benchmark: Batch of 100 timesheets.
///
/// Target: < 100ms mean
fn bench_batch_100(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let state = create_test_state();

    // Pre-create 100 different requests (vary employee IDs for realistic scenario)
    let requests: Vec<String> = (0..100)
        .map(|i| {
            let request_json = serde_json::json!({
                "employee": {
                    "id": format!("emp_batch_{:03}", i),
                    "employment_type": if i % 3 == 0 { "casual" } else { "full_time" },
                    "classification_code": "dce_level_3",
                    "date_of_birth": "1985-03-15",
                    "employment_start_date": "2020-01-01",
                    "tags": if i % 3 == 0 { vec!["laundry_allowance"] } else { vec![] }
                },
                "pay_period": {
                    "start_date": "2026-01-13",
                    "end_date": "2026-01-19",
                    "public_holidays": []
                },
                "shifts": [create_single_shift("2026-01-13")]
            });
            serde_json::to_string(&request_json).unwrap()
        })
        .collect();

    let mut group = c.benchmark_group("batch_processing");
    group.throughput(Throughput::Elements(100));

    group.bench_function("batch_100", |b| {
        b.to_async(&rt).iter(|| async {
            let mut results = Vec::with_capacity(100);
            for body in &requests {
                let router = create_router(state.clone());
                let response = router
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/calculate")
                            .header("Content-Type", "application/json")
                            .body(Body::from(body.clone()))
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                results.push(response);
            }
            black_box(results)
        })
    });

    group.finish();
}

/// Benchmark: Batch of 1000 timesheets.
///
/// Target: < 500ms mean
fn bench_batch_1000(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let state = create_test_state();

    // Pre-create 1000 different requests
    let requests: Vec<String> = (0..1000)
        .map(|i| {
            let request_json = serde_json::json!({
                "employee": {
                    "id": format!("emp_batch_{:04}", i),
                    "employment_type": if i % 3 == 0 { "casual" } else if i % 3 == 1 { "part_time" } else { "full_time" },
                    "classification_code": "dce_level_3",
                    "date_of_birth": "1985-03-15",
                    "employment_start_date": "2020-01-01",
                    "tags": if i % 3 == 0 { vec!["laundry_allowance"] } else { vec![] }
                },
                "pay_period": {
                    "start_date": "2026-01-13",
                    "end_date": "2026-01-19",
                    "public_holidays": []
                },
                "shifts": [create_single_shift("2026-01-13")]
            });
            serde_json::to_string(&request_json).unwrap()
        })
        .collect();

    let mut group = c.benchmark_group("large_batch_processing");
    group.throughput(Throughput::Elements(1000));
    // Reduce sample size for large batches to keep benchmark time reasonable
    group.sample_size(10);

    group.bench_function("batch_1000", |b| {
        b.to_async(&rt).iter(|| async {
            let mut results = Vec::with_capacity(1000);
            for body in &requests {
                let router = create_router(state.clone());
                let response = router
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri("/calculate")
                            .header("Content-Type", "application/json")
                            .body(Body::from(body.clone()))
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                results.push(response);
            }
            black_box(results)
        })
    });

    group.finish();
}

/// Benchmark: Various shift counts to understand scaling behavior.
fn bench_scaling(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let state = create_test_state();

    let mut group = c.benchmark_group("scaling");

    for shift_count in [1, 2, 4, 7, 14].iter() {
        let router = create_router(state.clone());
        let request = create_request_with_shifts(*shift_count);
        let body = serde_json::to_string(&request).unwrap();

        group.throughput(Throughput::Elements(*shift_count as u64));
        group.bench_with_input(
            BenchmarkId::new("shifts", shift_count),
            shift_count,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    let router = router.clone();
                    let response = router
                        .oneshot(
                            Request::builder()
                                .method("POST")
                                .uri("/calculate")
                                .header("Content-Type", "application/json")
                                .body(Body::from(body.clone()))
                                .unwrap(),
                        )
                        .await
                        .unwrap();
                    black_box(response)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_single_shift,
    bench_timesheet_14_shifts,
    bench_batch_100,
    bench_batch_1000,
    bench_scaling,
);
criterion_main!(benches);
