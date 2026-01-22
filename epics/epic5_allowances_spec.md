# Epic 5: Automatic Allowances - PRD

## Claude Instructions

You are implementing automatic allowance calculation for the Award Interpretation Engine.
This epic specifically implements the laundry allowance to prove the allowance framework works.

### How To Work On This PRD

1. Review the user stories below to understand all tasks and their status.
2. Review `epic5_allowances_progress.txt` to see what has already been done.
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
7. **APPEND** your progress to `epic5_allowances_progress.txt` (do not modify previous entries).
8. Make a git commit with a descriptive message.

### Quality Requirements

- All monetary calculations must use `rust_decimal::Decimal`
- Allowance eligibility is determined by employee tags
- Weekly caps must be enforced
- Include clause references from the Aged Care Award 2010
- All public functions must have rustdoc comments

### Stop Conditions

- If **ALL** user stories have `"passes": true`, output `<promise>COMPLETE</promise>`
- **ONLY** work on ONE user story per iteration
- Do **NOT** skip ahead or combine multiple stories

### Key Award References

| Allowance | Clause | Per Shift | Weekly Cap |
|-----------|--------|-----------|------------|
| Laundry | 15.2(b) | $0.32 | $1.49 |

### Allowance Eligibility

Employees are eligible for allowances based on their `tags` field:
- `"laundry_allowance"` tag enables laundry allowance

### Dependencies

- Epic 1 must be complete (models and config)
- Epic 2 must be complete (base rate calculation)

---

## Overview

**Epic Owner**: Developer B
**Duration**: 1-2 days
**Dependencies**: Epic 2

Implement automatic allowance calculation, specifically the laundry allowance. This proves the allowance framework works and can be extended to other allowances.

---

## User Stories

```json
[
  {
    "id": "US-5.1",
    "category": "calculation",
    "priority": 1,
    "title": "Laundry Allowance - Per Shift",
    "description": "Apply laundry allowance to employees tagged for it, with weekly cap",
    "acceptance_criteria": [
      "calculate_laundry_allowance(employee, shifts, config) returns AllowancePayment or None",
      "Returns None if employee does not have 'laundry_allowance' tag",
      "Returns AllowancePayment if employee has 'laundry_allowance' tag",
      "AllowancePayment.type is 'laundry'",
      "AllowancePayment.description is 'Laundry Allowance'",
      "AllowancePayment.units equals number of shifts",
      "AllowancePayment.rate is $0.32 per shift (from config)",
      "AllowancePayment.amount is units * rate, capped at weekly maximum",
      "Weekly maximum is $1.49 (from config)",
      "AllowancePayment.clause_ref is '15.2(b)'",
      "Audit step records allowance calculation",
      "Audit reasoning notes when weekly cap is applied"
    ],
    "test_cases": [
      {
        "id": "LA-001",
        "name": "1 shift with laundry tag",
        "has_tag": true,
        "num_shifts": 1,
        "per_shift_rate": "0.32",
        "weekly_cap": "1.49",
        "expected_units": 1,
        "expected_amount": "0.32"
      },
      {
        "id": "LA-002",
        "name": "3 shifts with laundry tag",
        "has_tag": true,
        "num_shifts": 3,
        "per_shift_rate": "0.32",
        "weekly_cap": "1.49",
        "expected_units": 3,
        "expected_amount": "0.96"
      },
      {
        "id": "LA-003",
        "name": "5 shifts hits cap",
        "has_tag": true,
        "num_shifts": 5,
        "per_shift_rate": "0.32",
        "weekly_cap": "1.49",
        "expected_units": 5,
        "expected_amount": "1.49",
        "note": "5 * 0.32 = 1.60, capped at 1.49"
      },
      {
        "id": "LA-004",
        "name": "6 shifts exceeds cap",
        "has_tag": true,
        "num_shifts": 6,
        "per_shift_rate": "0.32",
        "weekly_cap": "1.49",
        "expected_units": 6,
        "expected_amount": "1.49",
        "note": "6 * 0.32 = 1.92, capped at 1.49"
      },
      {
        "id": "LA-005",
        "name": "no laundry tag",
        "has_tag": false,
        "num_shifts": 3,
        "expected_amount": "0.00",
        "expected_result": "None"
      }
    ],
    "passes": false
  },
  {
    "id": "US-5.2",
    "category": "integration",
    "priority": 2,
    "title": "Allowance in Calculation Result",
    "description": "Include allowances in the calculation result with correct totals",
    "acceptance_criteria": [
      "CalculationResult.allowances contains all applicable allowances",
      "CalculationResult.totals.allowances_total equals sum of allowance amounts",
      "CalculationResult.totals.gross_pay includes allowances (pay_lines + allowances)",
      "Allowances appear after pay lines in the result",
      "Audit trace includes allowance calculation steps"
    ],
    "test_cases": [
      {
        "id": "CRAL-001",
        "name": "single shift with laundry allowance",
        "description": "Full-time, 8h Monday shift with laundry tag",
        "employment_type": "FullTime",
        "hours": "8.0",
        "has_laundry_tag": true,
        "expected_pay_lines_total": "228.32",
        "expected_allowances_total": "0.32",
        "expected_gross_pay": "228.64"
      },
      {
        "id": "CRAL-002",
        "name": "multiple shifts hit laundry cap",
        "description": "Casual, 5 shifts with laundry tag",
        "employment_type": "Casual",
        "num_shifts": 5,
        "has_laundry_tag": true,
        "expected_allowances_total": "1.49",
        "note": "Capped at weekly maximum"
      },
      {
        "id": "CRAL-003",
        "name": "no allowances",
        "description": "Full-time, 8h shift without laundry tag",
        "employment_type": "FullTime",
        "hours": "8.0",
        "has_laundry_tag": false,
        "expected_allowances_total": "0.00",
        "expected_allowances_array_length": 0
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
- Laundry allowance is correctly calculated and capped
- Allowances are included in gross pay totals
