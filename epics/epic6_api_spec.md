# Epic 6: API & Integration - PRD

## Claude Instructions

You are implementing the HTTP API and integration test suite for the Award Interpretation Engine.
This epic delivers the production-ready PoC by exposing the calculation engine via REST API.

### How To Work On This PRD

1. Review the user stories below to understand all tasks and their status.
2. Review `epic6_api_progress.txt` to see what has already been done.
3. Choose **ONE** user story to work on - prioritize by:
   - Dependencies (earlier stories may unblock later ones)
   - Priority number (lower = higher priority)
   - Stories marked `"passes": false`
4. Implement **ONLY** that one feature.
5. Run feedback loops:
   - Run `cargo build` to verify compilation
   - Run `cargo test` to verify tests pass
   - Run `cargo clippy` to check for warnings
   - For API stories, test with curl or similar
6. Update this PRD: change `"passes": false` to `"passes": true` for completed stories.
7. **APPEND** your progress to `epic6_api_progress.txt` (do not modify previous entries).
8. Make a git commit with a descriptive message.

### Quality Requirements

- Use axum for HTTP framework
- Use tokio for async runtime
- Return proper HTTP status codes
- Validate all input data
- Include correlation IDs for tracing
- All public functions must have rustdoc comments

### Stop Conditions

- If **ALL** user stories have `"passes": true`, output `<promise>COMPLETE</promise>`
- **ONLY** work on ONE user story per iteration
- Do **NOT** skip ahead or combine multiple stories

### API Design Principles

- RESTful endpoints
- JSON request/response bodies
- Descriptive error messages
- Include request validation
- Support CORS headers

### Performance Targets

| Metric | Target |
|--------|--------|
| Single shift calculation | < 1ms p99 |
| 1000 shift batch | < 100ms |
| Memory per calculation | < 1KB |

### Dependencies

- Epics 1-5 must be complete (all calculation logic ready)

---

## Overview

**Epic Owner**: Tech Lead
**Duration**: 2-3 days
**Dependencies**: Epics 1-5

Expose the calculation engine via an HTTP API and create the integration test suite. This epic delivers the production-ready PoC.

---

## User Stories

```json
[
  {
    "id": "US-6.1",
    "category": "api",
    "priority": 1,
    "title": "Calculate Endpoint",
    "description": "POST /calculate endpoint to submit timesheets and receive calculated pay",
    "acceptance_criteria": [
      "POST /calculate accepts JSON body with employee, pay_period, and shifts",
      "Returns HTTP 200 with CalculationResult JSON on success",
      "Returns HTTP 400 with error details for malformed JSON",
      "Returns HTTP 400 with validation errors for missing required fields",
      "Returns HTTP 400 with CLASSIFICATION_NOT_FOUND for unknown classification",
      "Response includes Content-Type: application/json header",
      "Request is logged with correlation ID"
    ],
    "request_schema": {
      "employee": {
        "id": "string (required)",
        "employment_type": "full_time | part_time | casual (required)",
        "classification_code": "string (required)",
        "date_of_birth": "YYYY-MM-DD (required)",
        "employment_start_date": "YYYY-MM-DD (required)",
        "base_hourly_rate": "decimal (optional)",
        "tags": ["string"]
      },
      "pay_period": {
        "start_date": "YYYY-MM-DD (required)",
        "end_date": "YYYY-MM-DD (required)",
        "public_holidays": []
      },
      "shifts": [
        {
          "id": "string (required)",
          "date": "YYYY-MM-DD (required)",
          "start_time": "ISO8601 datetime (required)",
          "end_time": "ISO8601 datetime (required)",
          "breaks": []
        }
      ]
    },
    "response_schema": {
      "calculation_id": "uuid",
      "timestamp": "ISO8601 datetime",
      "engine_version": "string",
      "employee_id": "string",
      "pay_period": {},
      "pay_lines": [],
      "allowances": [],
      "totals": {
        "gross_pay": "decimal",
        "ordinary_hours": "decimal",
        "overtime_hours": "decimal",
        "penalty_hours": "decimal",
        "allowances_total": "decimal"
      },
      "audit_trace": {}
    },
    "test_cases": [
      {
        "id": "API-001",
        "name": "valid request returns 200",
        "method": "POST",
        "path": "/calculate",
        "body": "valid CalculationRequest",
        "expected_status": 200,
        "expected_content_type": "application/json"
      },
      {
        "id": "API-002",
        "name": "malformed JSON returns 400",
        "method": "POST",
        "path": "/calculate",
        "body": "{invalid json",
        "expected_status": 400
      },
      {
        "id": "API-003",
        "name": "missing employee.id returns 400",
        "method": "POST",
        "path": "/calculate",
        "body": "request without employee.id",
        "expected_status": 400,
        "expected_error": "missing field: employee.id"
      },
      {
        "id": "API-004",
        "name": "unknown classification returns 400",
        "method": "POST",
        "path": "/calculate",
        "body": "request with classification_code: 'unknown'",
        "expected_status": 400,
        "expected_error_code": "CLASSIFICATION_NOT_FOUND"
      }
    ],
    "passes": false
  },
  {
    "id": "US-6.2",
    "category": "api",
    "priority": 2,
    "title": "Health Check Endpoint",
    "description": "GET /health endpoint to verify service is running",
    "acceptance_criteria": [
      "GET /health returns HTTP 200 when service is healthy",
      "Response body contains { \"status\": \"healthy\", \"version\": \"0.1.0\" }",
      "Returns HTTP 503 if configuration cannot be loaded",
      "Response body for unhealthy contains { \"status\": \"unhealthy\", \"reason\": \"...\" }"
    ],
    "test_cases": [
      {
        "id": "HEALTH-001",
        "name": "healthy service returns 200",
        "method": "GET",
        "path": "/health",
        "expected_status": 200,
        "expected_body": {"status": "healthy", "version": "0.1.0"}
      }
    ],
    "passes": false
  },
  {
    "id": "US-6.3",
    "category": "api",
    "priority": 3,
    "title": "Info Endpoint",
    "description": "GET /info endpoint to show supported awards and classifications",
    "acceptance_criteria": [
      "GET /info returns HTTP 200",
      "Response includes engine_version",
      "Response includes supported_awards array",
      "Each award includes: code, name, classifications, effective_date"
    ],
    "test_cases": [
      {
        "id": "INFO-001",
        "name": "info returns supported awards",
        "method": "GET",
        "path": "/info",
        "expected_status": 200,
        "expected_body": {
          "engine_version": "0.1.0",
          "supported_awards": [
            {
              "code": "MA000018",
              "name": "Aged Care Award 2010",
              "classifications": ["dce_level_3"],
              "effective_date": "2025-07-01"
            }
          ]
        }
      }
    ],
    "passes": false
  },
  {
    "id": "US-6.4",
    "category": "testing",
    "priority": 4,
    "title": "Integration Test Suite",
    "description": "Comprehensive integration tests covering all calculation scenarios",
    "acceptance_criteria": [
      "`cargo test --test integration` runs all integration tests",
      "At least 5 tests for ordinary hours (weekday)",
      "At least 5 tests for Saturday penalty",
      "At least 5 tests for Sunday penalty",
      "At least 5 tests for overnight shift splitting",
      "At least 5 tests for daily overtime (weekday)",
      "At least 5 tests for daily overtime (weekend)",
      "At least 5 tests for casual vs non-casual",
      "At least 5 tests for laundry allowance",
      "At least 5 tests for error cases",
      "Total: 45+ integration tests",
      "Each test uses realistic test fixtures",
      "Each test validates all response fields",
      "Each test checks audit trace contains expected steps"
    ],
    "test_fixtures": [
      {
        "name": "simple_weekday.json",
        "description": "Full-time employee, 8-hour Monday shift",
        "expected_gross_pay": "228.32"
      },
      {
        "name": "casual_saturday.json",
        "description": "Casual employee, 8-hour Saturday shift with laundry",
        "expected_gross_pay": "399.88"
      },
      {
        "name": "weekday_overtime.json",
        "description": "Full-time employee, 12-hour weekday shift",
        "expected_gross_pay": "428.10"
      },
      {
        "name": "overnight_sat_sun.json",
        "description": "Part-time employee, overnight Sat-Sun shift",
        "expected_gross_pay": "385.29"
      }
    ],
    "passes": false
  },
  {
    "id": "US-6.5",
    "category": "performance",
    "priority": 5,
    "title": "Performance Benchmarks",
    "description": "Create benchmark suite proving performance targets are met",
    "acceptance_criteria": [
      "`cargo bench` runs benchmark suite",
      "Benchmark for single shift calculation (target: < 100μs)",
      "Benchmark for single timesheet with 1 shift (target: < 1ms)",
      "Benchmark for timesheet with 14 shifts (target: < 5ms)",
      "Benchmark for batch of 100 timesheets (target: < 100ms)",
      "Benchmark for batch of 1000 timesheets (target: < 500ms)",
      "Uses criterion for statistical analysis",
      "Includes warmup iterations",
      "Reports mean, median, and p99 latencies",
      "Generates HTML reports in target/criterion/"
    ],
    "benchmark_implementation": {
      "file": "benches/calculation_benchmarks.rs",
      "benchmarks": [
        {
          "name": "single_shift",
          "description": "Calculate pay for a single 8-hour shift",
          "target_mean": "100μs"
        },
        {
          "name": "timesheet_14_shifts",
          "description": "Calculate pay for 2-week timesheet (14 shifts)",
          "target_mean": "5ms"
        },
        {
          "name": "batch_100",
          "description": "Calculate 100 timesheets sequentially",
          "target_mean": "100ms"
        },
        {
          "name": "batch_1000",
          "description": "Calculate 1000 timesheets sequentially",
          "target_mean": "500ms"
        }
      ]
    },
    "passes": false
  }
]
```

---

## Test Fixtures (Reference)

### Fixture 1: Simple Weekday Shift

```json
{
  "description": "Full-time employee, 8-hour Monday shift",
  "employee": {
    "id": "emp_ft_001",
    "employment_type": "full_time",
    "classification_code": "dce_level_3",
    "date_of_birth": "1985-03-15",
    "employment_start_date": "2020-01-01",
    "tags": []
  },
  "pay_period": {
    "start_date": "2026-01-13",
    "end_date": "2026-01-19",
    "public_holidays": []
  },
  "shifts": [
    {
      "id": "shift_001",
      "date": "2026-01-13",
      "start_time": "2026-01-13T09:00:00",
      "end_time": "2026-01-13T17:00:00",
      "breaks": []
    }
  ],
  "expected": {
    "gross_pay": "228.32",
    "ordinary_hours": "8.0",
    "overtime_hours": "0.0"
  }
}
```

### Fixture 2: Casual Saturday Shift

```json
{
  "description": "Casual employee, 8-hour Saturday shift",
  "employee": {
    "id": "emp_cas_001",
    "employment_type": "casual",
    "classification_code": "dce_level_3",
    "date_of_birth": "1990-07-22",
    "employment_start_date": "2024-06-01",
    "tags": ["laundry_allowance"]
  },
  "pay_period": {
    "start_date": "2026-01-13",
    "end_date": "2026-01-19",
    "public_holidays": []
  },
  "shifts": [
    {
      "id": "shift_001",
      "date": "2026-01-17",
      "start_time": "2026-01-17T09:00:00",
      "end_time": "2026-01-17T17:00:00",
      "breaks": []
    }
  ],
  "expected": {
    "gross_pay": "399.88",
    "ordinary_hours": "8.0",
    "overtime_hours": "0.0"
  }
}
```

---

## Exit Condition

Output `<promise>COMPLETE</promise>` when:

- All user stories have `"passes": true`
- `cargo build` succeeds
- `cargo test` passes all tests (45+ integration tests)
- `cargo bench` runs successfully
- `cargo clippy` produces no warnings
- API endpoints respond correctly
- Performance targets are documented
