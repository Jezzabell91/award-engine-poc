# Epic 2: Base Rate & Casual Loading Calculation - PRD

## Claude Instructions

You are implementing the base rate lookup and casual loading calculations for the Award Interpretation Engine.
This epic builds upon Epic 1 (Project Foundation) and implements the first layer of the calculation pipeline.

### How To Work On This PRD

1. Review the user stories below to understand all tasks and their status.
2. Review `epic2_base_rate_progress.txt` to see what has already been done.
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
7. **APPEND** your progress to `epic2_base_rate_progress.txt` (do not modify previous entries).
8. Make a git commit with a descriptive message.

### Quality Requirements

- All monetary calculations must use `rust_decimal::Decimal`
- Audit steps must be recorded for every calculation decision
- Include clause references from the Aged Care Award 2010
- All public functions must have rustdoc comments
- Run `cargo fmt` and `cargo clippy` before committing

### Stop Conditions

- If **ALL** user stories have `"passes": true`, output `<promise>COMPLETE</promise>`
- **ONLY** work on ONE user story per iteration
- Do **NOT** skip ahead or combine multiple stories

### Key Award References

| Rule | Clause | Description |
|------|--------|-------------|
| Base Rate | 14.2 | Classification rates |
| Casual Loading | 10.4(b) | 25% loading for casual employees |
| Ordinary Hours | 22.1 | Definition of ordinary hours |

### Dependencies

- Epic 1 must be complete (all models and config loader available)

---

## Overview

**Epic Owner**: Developer A
**Duration**: 2-3 days
**Dependencies**: Epic 1

Implement the foundational calculation logic: determining base hourly rates from configuration and applying casual loading. This is the first layer of the calculation pipeline.

---

## User Stories

```json
[
  {
    "id": "US-2.1",
    "category": "calculation",
    "priority": 1,
    "title": "Base Rate Lookup",
    "description": "Determine an employee's base hourly rate from configuration or override",
    "acceptance_criteria": [
      "get_base_rate(employee, effective_date, config) returns the correct rate",
      "Uses employee.base_hourly_rate override if Some",
      "Falls back to config lookup by classification_code if None",
      "Returns ClassificationNotFound error for unknown classification",
      "Returns RateNotFound error if no rate exists for the effective date",
      "Records audit step with rule_id: 'base_rate_lookup'",
      "Audit step includes clause_ref: '14.2'",
      "Audit step input contains classification_code",
      "Audit step output contains the determined rate"
    ],
    "test_cases": [
      {
        "id": "BR-001",
        "name": "config rate for dce_level_3",
        "employee_classification": "dce_level_3",
        "override_rate": null,
        "effective_date": "2025-08-01",
        "expected": "28.54"
      },
      {
        "id": "BR-002",
        "name": "override rate takes precedence",
        "employee_classification": "dce_level_3",
        "override_rate": "32.00",
        "effective_date": "2025-08-01",
        "expected": "32.00"
      },
      {
        "id": "BR-003",
        "name": "unknown classification returns error",
        "employee_classification": "unknown",
        "override_rate": null,
        "effective_date": "2025-08-01",
        "expected": "ClassificationNotFound error"
      },
      {
        "id": "BR-004",
        "name": "no rate for early date returns error",
        "employee_classification": "dce_level_3",
        "override_rate": null,
        "effective_date": "2020-01-01",
        "expected": "RateNotFound error"
      }
    ],
    "passes": true
  },
  {
    "id": "US-2.2",
    "category": "calculation",
    "priority": 2,
    "title": "Casual Loading Application",
    "description": "Apply 25% casual loading to base rates for casual employees",
    "acceptance_criteria": [
      "apply_casual_loading(base_rate, employee) applies 25% loading for casuals",
      "Loading multiplier is exactly 1.25",
      "Returns unchanged rate for FullTime employees",
      "Returns unchanged rate for PartTime employees",
      "Records audit step with rule_id: 'casual_loading'",
      "Audit step includes clause_ref: '10.4(b)'",
      "Audit step input contains base_rate and employment_type",
      "Audit step output contains loaded_rate",
      "Audit reasoning explains the calculation (e.g., '$28.54 x 1.25 = $35.68')"
    ],
    "test_cases": [
      {
        "id": "CL-001",
        "name": "casual gets 25% loading",
        "employment_type": "Casual",
        "base_rate": "28.54",
        "expected": "35.675"
      },
      {
        "id": "CL-002",
        "name": "fulltime gets no loading",
        "employment_type": "FullTime",
        "base_rate": "28.54",
        "expected": "28.54"
      },
      {
        "id": "CL-003",
        "name": "parttime gets no loading",
        "employment_type": "PartTime",
        "base_rate": "28.54",
        "expected": "28.54"
      },
      {
        "id": "CL-004",
        "name": "casual loading on different rate",
        "employment_type": "Casual",
        "base_rate": "25.00",
        "expected": "31.25"
      },
      {
        "id": "CL-005",
        "name": "casual loading on zero rate",
        "employment_type": "Casual",
        "base_rate": "0.00",
        "expected": "0.00"
      }
    ],
    "passes": false
  },
  {
    "id": "US-2.3",
    "category": "calculation",
    "priority": 3,
    "title": "Ordinary Hours Calculation",
    "description": "Calculate pay for ordinary (non-penalty, non-overtime) hours worked",
    "acceptance_criteria": [
      "calculate_ordinary_hours(shift, employee, config) returns PayLine",
      "PayLine has category: Ordinary for non-casuals",
      "PayLine has category: OrdinaryCasual for casuals",
      "PayLine.hours equals shift.worked_hours()",
      "PayLine.base_rate is from config or employee override",
      "PayLine.multiplier is 1.0 for non-casuals, 1.25 for casuals",
      "PayLine.amount = hours * base_rate * multiplier",
      "PayLine.clause_ref references ordinary hours clause",
      "Audit trace includes all calculation steps in order",
      "Audit trace includes: base rate lookup, casual loading (if applicable), pay line generation"
    ],
    "test_cases": [
      {
        "id": "OH-001",
        "name": "fulltime 8 hour weekday shift",
        "day": "Monday",
        "hours": "8.0",
        "employment_type": "FullTime",
        "base_rate": "28.54",
        "expected_amount": "228.32",
        "expected_multiplier": "1.0"
      },
      {
        "id": "OH-002",
        "name": "parttime 8 hour weekday shift",
        "day": "Tuesday",
        "hours": "8.0",
        "employment_type": "PartTime",
        "base_rate": "28.54",
        "expected_amount": "228.32",
        "expected_multiplier": "1.0"
      },
      {
        "id": "OH-003",
        "name": "casual 8 hour weekday shift",
        "day": "Wednesday",
        "hours": "8.0",
        "employment_type": "Casual",
        "base_rate": "28.54",
        "expected_amount": "285.40",
        "expected_multiplier": "1.25"
      },
      {
        "id": "OH-004",
        "name": "fulltime 4 hour shift",
        "day": "Thursday",
        "hours": "4.0",
        "employment_type": "FullTime",
        "base_rate": "28.54",
        "expected_amount": "114.16",
        "expected_multiplier": "1.0"
      },
      {
        "id": "OH-005",
        "name": "casual 7.5 hour shift",
        "day": "Friday",
        "hours": "7.5",
        "employment_type": "Casual",
        "base_rate": "28.54",
        "expected_amount": "267.56",
        "expected_multiplier": "1.25"
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
- All audit steps are correctly recorded
