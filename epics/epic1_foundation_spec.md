# Epic 1: Project Foundation & Core Types - PRD

## Claude Instructions

You are implementing the foundation for an Award Interpretation Engine in Rust.
This PoC validates technical approach for interpreting the Aged Care Award 2010 (MA000018).

### How To Work On This PRD

1. Review the user stories below to understand all tasks and their status.
2. Review `epic1_foundation_progress.txt` to see what has already been done.
3. Choose **ONE** user story to work on - prioritize by:
   - Dependencies (earlier stories may unblock later ones)
   - Priority number (lower = higher priority)
   - Stories marked `"passes": false`
4. Implement **ONLY** that one feature.
5. Run feedback loops:
   - Run `cargo build` to verify compilation
   - Run `cargo test` to verify tests pass
   - Run `cargo clippy` to check for warnings
6. Update this PRD: change `"passes": false` to `"passes": true` for completed stories.
7. **APPEND** your progress to `epic1_foundation_progress.txt` (do not modify previous entries).
8. Make a git commit with a descriptive message.

### Quality Requirements

- Use Rust latest stable edition
- All public items must have rustdoc comments
- Use `thiserror` for error types
- Use `rust_decimal` for all monetary calculations
- Use `chrono` for all date/time handling
- Run `cargo fmt` before committing
- Run `cargo clippy` with no warnings

### Stop Conditions

- If **ALL** user stories have `"passes": true`, output `<promise>COMPLETE</promise>`
- **ONLY** work on ONE user story per iteration
- Do **NOT** skip ahead or combine multiple stories

### Definition of Done (Per Story)

1. Code Complete: All functionality implemented as specified
2. Unit Tests: All unit tests written and passing
3. Documentation: Rustdoc comments on all public items
4. No Warnings: `cargo clippy` passes with no warnings
5. Formatted: `cargo fmt` applied

### Reference Files

| File | Purpose |
|------|---------|
| `Award_Interpretation_Engine_PoC_Specification.md` | Full technical specification |
| `config/ma000018/*.yaml` | Configuration files to create |

---

## Overview

**Epic Owner**: Tech Lead
**Duration**: 2-3 days
**Dependencies**: None

Establish the Rust project structure, define all core data types, and implement the configuration loading system. This epic creates the foundation that all other epics build upon.

### Technology Stack

| Component | Technology | Version |
|-----------|------------|---------|
| Language | Rust | latest stable (stable) |
| Decimal Math | rust_decimal | 1.33+ |
| Date/Time | chrono | 0.4+ |
| Serialization | serde, serde_json, serde_yaml | Latest |
| HTTP Server | axum | 0.7+ |
| Error Handling | thiserror | 1.0 |

---

## User Stories

```json
[
  {
    "id": "US-1.1",
    "category": "setup",
    "priority": 1,
    "title": "Project Scaffolding",
    "description": "Create a properly configured Rust project with all dependencies",
    "acceptance_criteria": [
      "Project compiles without errors with `cargo build`",
      "All tests pass with `cargo test` (even if no tests yet)",
      "`cargo clippy` produces no warnings or errors",
      "Cargo.toml contains rust_decimal with serde feature",
      "Cargo.toml contains chrono with serde feature",
      "Cargo.toml contains serde with derive feature",
      "Cargo.toml contains serde_json",
      "Cargo.toml contains serde_yaml",
      "Cargo.toml contains axum",
      "Cargo.toml contains tokio with full feature",
      "Cargo.toml contains thiserror",
      "Cargo.toml contains uuid with v4 and serde features",
      "Cargo.toml contains tracing",
      "Dev dependencies include criterion for benchmarking",
      "Dev dependencies include proptest for property testing",
      "Uses Rust latest stable edition",
      "Release profile has LTO enabled"
    ],
    "test_commands": [
      "cargo build",
      "cargo test",
      "cargo clippy -- -D warnings"
    ],
    "passes": true
  },
  {
    "id": "US-1.2",
    "category": "models",
    "priority": 2,
    "title": "Employee Model",
    "description": "Create a strongly-typed Employee model with employment type enum",
    "acceptance_criteria": [
      "Employee struct has field: id (String)",
      "Employee struct has field: employment_type (EmploymentType enum)",
      "Employee struct has field: classification_code (String)",
      "Employee struct has field: date_of_birth (NaiveDate)",
      "Employee struct has field: employment_start_date (NaiveDate)",
      "Employee struct has field: base_hourly_rate (Option<Decimal>)",
      "Employee struct has field: tags (Vec<String>)",
      "EmploymentType enum has variant: FullTime",
      "EmploymentType enum has variant: PartTime",
      "EmploymentType enum has variant: Casual",
      "Employee can be serialized to JSON using serde",
      "Employee can be deserialized from JSON using serde",
      "Employee has method is_casual() returning bool",
      "is_casual() returns true for Casual employment type",
      "is_casual() returns false for FullTime employment type",
      "is_casual() returns false for PartTime employment type"
    ],
    "test_cases": [
      {
        "name": "deserialize_fulltime_employee",
        "input": "{\"id\":\"emp_001\",\"employment_type\":\"full_time\",\"classification_code\":\"dce_level_3\",\"date_of_birth\":\"1990-01-15\",\"employment_start_date\":\"2023-06-01\",\"tags\":[]}",
        "expected": "Employee with FullTime type"
      },
      {
        "name": "is_casual_returns_true_for_casual",
        "setup": "Create Employee with Casual type",
        "expected": "is_casual() returns true"
      },
      {
        "name": "is_casual_returns_false_for_fulltime",
        "setup": "Create Employee with FullTime type",
        "expected": "is_casual() returns false"
      }
    ],
    "passes": false
  },
  {
    "id": "US-1.3",
    "category": "models",
    "priority": 3,
    "title": "Shift Model",
    "description": "Create a Shift model that captures timing information and calculates worked hours",
    "acceptance_criteria": [
      "Shift struct has field: id (String)",
      "Shift struct has field: date (NaiveDate)",
      "Shift struct has field: start_time (NaiveDateTime)",
      "Shift struct has field: end_time (NaiveDateTime)",
      "Shift struct has field: breaks (Vec<Break>)",
      "Break struct has field: start_time (NaiveDateTime)",
      "Break struct has field: end_time (NaiveDateTime)",
      "Break struct has field: is_paid (bool)",
      "Shift has method worked_hours() returning Decimal",
      "worked_hours() subtracts unpaid breaks from total duration",
      "worked_hours() does NOT subtract paid breaks",
      "Shift has method day_of_week() returning chrono::Weekday",
      "Shift can be serialized to/from JSON"
    ],
    "test_cases": [
      {
        "id": "SH-001",
        "name": "8 hour shift no breaks",
        "shift": "09:00 to 17:00, no breaks",
        "expected_hours": "8.0"
      },
      {
        "id": "SH-002",
        "name": "8.5 hour shift with 30min unpaid break",
        "shift": "09:00 to 17:30, 30min unpaid break",
        "expected_hours": "8.0"
      },
      {
        "id": "SH-003",
        "name": "8.5 hour shift with 30min paid break",
        "shift": "09:00 to 17:30, 30min paid break",
        "expected_hours": "8.5"
      },
      {
        "id": "SH-004",
        "name": "overnight shift",
        "shift": "22:00 to 06:00 next day, no breaks",
        "expected_hours": "8.0"
      },
      {
        "id": "SH-005",
        "name": "zero duration shift",
        "shift": "09:00 to 09:00",
        "expected_hours": "0.0"
      }
    ],
    "passes": false
  },
  {
    "id": "US-1.4",
    "category": "models",
    "priority": 4,
    "title": "PayPeriod Model",
    "description": "Create a PayPeriod model that defines the calculation context",
    "acceptance_criteria": [
      "PayPeriod struct has field: start_date (NaiveDate)",
      "PayPeriod struct has field: end_date (NaiveDate)",
      "PayPeriod struct has field: public_holidays (Vec<PublicHoliday>)",
      "PublicHoliday struct has field: date (NaiveDate)",
      "PublicHoliday struct has field: name (String)",
      "PublicHoliday struct has field: region (String)",
      "PayPeriod has method contains_date(date) returning bool",
      "PayPeriod has method is_public_holiday(date) returning bool",
      "PayPeriod can be serialized to/from JSON"
    ],
    "test_cases": [
      {
        "id": "PP-001",
        "name": "contains_date within period",
        "period": "2026-01-13 to 2026-01-26",
        "test_date": "2026-01-15",
        "expected": true
      },
      {
        "id": "PP-002",
        "name": "contains_date outside period",
        "period": "2026-01-13 to 2026-01-26",
        "test_date": "2026-01-27",
        "expected": false
      },
      {
        "id": "PP-003",
        "name": "is_public_holiday returns true",
        "period": "2026-01-13 to 2026-01-26 with Australia Day on 2026-01-26",
        "test_date": "2026-01-26",
        "expected": true
      },
      {
        "id": "PP-004",
        "name": "is_public_holiday returns false",
        "period": "2026-01-13 to 2026-01-26 with no holidays",
        "test_date": "2026-01-15",
        "expected": false
      }
    ],
    "passes": false
  },
  {
    "id": "US-1.5",
    "category": "models",
    "priority": 5,
    "title": "CalculationResult Model",
    "description": "Create a comprehensive result model that captures all calculation outputs",
    "acceptance_criteria": [
      "CalculationResult struct has field: calculation_id (Uuid)",
      "CalculationResult struct has field: timestamp (DateTime<Utc>)",
      "CalculationResult struct has field: engine_version (String)",
      "CalculationResult struct has field: employee_id (String)",
      "CalculationResult struct has field: pay_period (PayPeriod)",
      "CalculationResult struct has field: pay_lines (Vec<PayLine>)",
      "CalculationResult struct has field: allowances (Vec<AllowancePayment>)",
      "CalculationResult struct has field: totals (PayTotals)",
      "CalculationResult struct has field: audit_trace (AuditTrace)",
      "PayLine struct has all required fields per spec",
      "PayCategory enum has variants: Ordinary, OrdinaryCasual, Saturday, SaturdayCasual, Sunday, SundayCasual, Overtime150, Overtime200",
      "PayTotals struct has: gross_pay, ordinary_hours, overtime_hours, penalty_hours, allowances_total",
      "AllowancePayment struct has: type, description, units, rate, amount, clause_ref",
      "All monetary fields use Decimal type",
      "CalculationResult can be serialized to JSON"
    ],
    "test_cases": [
      {
        "id": "CR-001",
        "name": "gross_pay equals sum of pay_lines",
        "pay_lines": [100.00, 50.00, 75.50],
        "expected_gross": "225.50"
      }
    ],
    "passes": false
  },
  {
    "id": "US-1.6",
    "category": "models",
    "priority": 6,
    "title": "AuditTrace Model",
    "description": "Create an audit trace that records every calculation decision",
    "acceptance_criteria": [
      "AuditTrace struct has field: steps (Vec<AuditStep>)",
      "AuditTrace struct has field: warnings (Vec<AuditWarning>)",
      "AuditTrace struct has field: duration_us (u64)",
      "AuditStep struct has field: step_number (u32)",
      "AuditStep struct has field: rule_id (String)",
      "AuditStep struct has field: rule_name (String)",
      "AuditStep struct has field: clause_ref (String)",
      "AuditStep struct has field: input (serde_json::Value)",
      "AuditStep struct has field: output (serde_json::Value)",
      "AuditStep struct has field: reasoning (String)",
      "AuditWarning struct has field: code (String)",
      "AuditWarning struct has field: message (String)",
      "AuditWarning struct has field: severity (String)",
      "AuditTrace can be serialized to JSON"
    ],
    "test_cases": [
      {
        "id": "AT-001",
        "name": "steps are ordered by step_number",
        "setup": "Create AuditTrace with multiple steps",
        "expected": "Steps can be iterated in order"
      }
    ],
    "passes": false
  },
  {
    "id": "US-1.7",
    "category": "config",
    "priority": 7,
    "title": "Configuration Loader",
    "description": "Load award configuration from YAML files",
    "acceptance_criteria": [
      "ConfigLoader::load(path) loads configuration from directory",
      "Configuration includes award metadata (code, name, version)",
      "Configuration includes classifications with codes and names",
      "Configuration includes hourly rates by classification and effective date",
      "Configuration includes penalty rates for Saturday (by employment type)",
      "Configuration includes penalty rates for Sunday (by employment type)",
      "Configuration includes overtime thresholds and rates",
      "Configuration includes allowance rates (laundry per shift and weekly cap)",
      "get_classification(code) returns classification or error",
      "get_hourly_rate(classification, date) returns rate for effective date",
      "get_penalty(day_type, employment_type) returns multiplier",
      "Malformed YAML returns descriptive error",
      "Missing file returns error indicating which file"
    ],
    "config_files": [
      {
        "path": "config/ma000018/award.yaml",
        "content": "code: MA000018\nname: Aged Care Award 2010\nversion: \"2025-07-01\"\nsource_url: https://library.fairwork.gov.au/award/?krn=MA000018"
      },
      {
        "path": "config/ma000018/classifications.yaml",
        "content": "classifications:\n  dce_level_3:\n    name: \"Direct Care Employee Level 3 - Qualified\"\n    description: \"Qualified direct care worker\"\n    clause: \"14.2\""
      },
      {
        "path": "config/ma000018/rates/2025-07-01.yaml",
        "content": "effective_date: 2025-07-01\nrates:\n  dce_level_3:\n    weekly: 1084.70\n    hourly: 28.54\nallowances:\n  laundry_per_shift: 0.32\n  laundry_per_week: 1.49"
      },
      {
        "path": "config/ma000018/penalties.yaml",
        "content": "penalties:\n  saturday:\n    clause: \"23.1, 23.2(a)\"\n    full_time: 1.50\n    part_time: 1.50\n    casual: latest stable\n  sunday:\n    clause: \"23.1, 23.2(b)\"\n    full_time: latest stable\n    part_time: latest stable\n    casual: 2.00\novertime:\n  daily_threshold_hours: 8\n  weekday:\n    clause: \"25.1\"\n    first_two_hours:\n      full_time: 1.50\n      part_time: 1.50\n      casual: 1.875\n    after_two_hours:\n      full_time: 2.00\n      part_time: 2.00\n      casual: 2.50"
      }
    ],
    "test_cases": [
      {
        "id": "CL-001",
        "name": "load valid configuration",
        "path": "./config",
        "expected": "Configuration loaded successfully"
      },
      {
        "id": "CL-002",
        "name": "get_hourly_rate for dce_level_3",
        "classification": "dce_level_3",
        "date": "2025-08-01",
        "expected": "28.54"
      },
      {
        "id": "CL-003",
        "name": "get_penalty for Saturday casual",
        "day_type": "saturday",
        "employment_type": "casual",
        "expected": "latest stable"
      },
      {
        "id": "CL-004",
        "name": "unknown classification returns error",
        "classification": "unknown",
        "expected": "ClassificationNotFound error"
      }
    ],
    "passes": false
  },
  {
    "id": "US-1.8",
    "category": "errors",
    "priority": 8,
    "title": "Error Types",
    "description": "Create well-defined error types using thiserror",
    "acceptance_criteria": [
      "EngineError enum exists with thiserror derive",
      "EngineError has variant: ConfigNotFound { path: String }",
      "EngineError has variant: ConfigParseError { path: String, message: String }",
      "EngineError has variant: ClassificationNotFound { code: String }",
      "EngineError has variant: RateNotFound { classification: String, date: NaiveDate }",
      "EngineError has variant: InvalidShift { shift_id: String, message: String }",
      "EngineError has variant: InvalidEmployee { field: String, message: String }",
      "EngineError has variant: CalculationError { message: String }",
      "All variants implement std::error::Error",
      "All variants implement Display with human-readable messages",
      "Errors propagate correctly with ? operator"
    ],
    "test_cases": [
      {
        "id": "ERR-001",
        "name": "ConfigNotFound displays path",
        "error": "ConfigNotFound { path: \"/missing/file.yaml\" }",
        "expected_message": "Configuration file not found: /missing/file.yaml"
      },
      {
        "id": "ERR-002",
        "name": "ClassificationNotFound displays code",
        "error": "ClassificationNotFound { code: \"unknown\" }",
        "expected_message": "Classification not found: unknown"
      }
    ],
    "passes": false
  }
]
```

---

## Exit Condition

Output `<promise>COMPLETE</promise>` when:

- All user stories have `"passes": true`
- `cargo build` succeeds
- `cargo test` passes all tests
- `cargo clippy` produces no warnings
