# Epic 4: Daily Overtime Rules - PRD

## Claude Instructions

You are implementing daily overtime calculation (hours exceeding 8 per day) for the Award Interpretation Engine.
This epic introduces tiered overtime rates (first 2 hours at 150%, thereafter at 200%).

### How To Work On This PRD

1. Review the user stories below to understand all tasks and their status.
2. Review `epic4_overtime_progress.txt` to see what has already been done.
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
7. **APPEND** your progress to `epic4_overtime_progress.txt` (do not modify previous entries).
8. Make a git commit with a descriptive message.

### Quality Requirements

- All monetary calculations must use `rust_decimal::Decimal`
- Casual overtime rates include casual loading (e.g., 1.5 * 1.25 = 1.875)
- Weekend overtime is 200% from the first hour
- Include clause references from the Aged Care Award 2010
- All public functions must have rustdoc comments

### Stop Conditions

- If **ALL** user stories have `"passes": true`, output `<promise>COMPLETE</promise>`
- **ONLY** work on ONE user story per iteration
- Do **NOT** skip ahead or combine multiple stories

### Key Award References

| Rule | Clause | Description |
|------|--------|-------------|
| Daily OT Threshold | 22.1(c), 25.1 | Overtime after 8 hours per day |
| Weekday OT First 2h | 25.1(a)(i)(A) | 150% (non-casual), 187.5% (casual) |
| Weekday OT After 2h | 25.1(a)(i)(A) | 200% (non-casual), 250% (casual) |
| Weekend OT | 25.1(a)(i)(B) | 200% from first hour |

### Overtime Rate Calculation

**Weekday Overtime:**
- Non-casual: First 2 hours @ 150%, then @ 200%
- Casual: First 2 hours @ 187.5% (1.5 * 1.25), then @ 250% (2.0 * 1.25)

**Weekend Overtime:**
- Non-casual: All overtime @ 200%
- Casual Saturday: All overtime @ 250% (2.0 * 1.25)
- Casual Sunday: All overtime @ 250% (2.0 * 1.25)

### Dependencies

- Epic 1 must be complete (models and config)
- Epic 2 must be complete (base rate and casual loading)
- Epic 3 must be complete (weekend penalties)

---

## Overview

**Epic Owner**: Developer A
**Duration**: 2-3 days
**Dependencies**: Epic 2, Epic 3

Implement daily overtime calculation (hours exceeding 8 per day). This introduces the concept of tiered rates (first 2 hours at 150%, thereafter at 200%).

---

## User Stories

```json
[
  {
    "id": "US-4.1",
    "category": "detection",
    "priority": 1,
    "title": "Daily Overtime Detection",
    "description": "Detect when a shift exceeds 8 ordinary hours",
    "acceptance_criteria": [
      "detect_daily_overtime(shift, threshold) splits hours into ordinary and overtime",
      "Default threshold is 8 hours (configurable)",
      "Shifts <= threshold return 0 overtime hours",
      "Shifts > threshold return excess as overtime",
      "Returns struct with ordinary_hours and overtime_hours fields",
      "Audit step records detection with rule_id: 'daily_overtime_detection'",
      "Audit step includes clause_ref: '22.1(c), 25.1'",
      "Audit step input contains worked_hours and threshold",
      "Audit step output contains ordinary_hours and overtime_hours"
    ],
    "test_cases": [
      {
        "id": "DOD-001",
        "name": "exactly 8 hours - no overtime",
        "worked_hours": "8.0",
        "threshold": "8.0",
        "expected_ordinary": "8.0",
        "expected_overtime": "0.0"
      },
      {
        "id": "DOD-002",
        "name": "10 hours - 2 hours overtime",
        "worked_hours": "10.0",
        "threshold": "8.0",
        "expected_ordinary": "8.0",
        "expected_overtime": "2.0"
      },
      {
        "id": "DOD-003",
        "name": "12 hours - 4 hours overtime",
        "worked_hours": "12.0",
        "threshold": "8.0",
        "expected_ordinary": "8.0",
        "expected_overtime": "4.0"
      },
      {
        "id": "DOD-004",
        "name": "6 hours - no overtime",
        "worked_hours": "6.0",
        "threshold": "8.0",
        "expected_ordinary": "6.0",
        "expected_overtime": "0.0"
      },
      {
        "id": "DOD-005",
        "name": "8.5 hours - 0.5 hours overtime",
        "worked_hours": "8.5",
        "threshold": "8.0",
        "expected_ordinary": "8.0",
        "expected_overtime": "0.5"
      },
      {
        "id": "DOD-006",
        "name": "11.25 hours - 3.25 hours overtime",
        "worked_hours": "11.25",
        "threshold": "8.0",
        "expected_ordinary": "8.0",
        "expected_overtime": "3.25"
      }
    ],
    "passes": true
  },
  {
    "id": "US-4.2",
    "category": "calculation",
    "priority": 2,
    "title": "Weekday Overtime Rate Calculation",
    "description": "Calculate overtime pay at tiered rates for weekday shifts",
    "acceptance_criteria": [
      "calculate_weekday_overtime(overtime_hours, employee, config) returns Vec<PayLine>",
      "First 2 hours: 150% for non-casuals, 187.5% for casuals",
      "After 2 hours: 200% for non-casuals, 250% for casuals",
      "Returns separate PayLines for each tier",
      "PayLine.category is Overtime150 or Overtime200 as appropriate",
      "Casual overtime rates include casual loading (1.5*1.25, 2.0*1.25)",
      "Audit trace records each tier calculation separately"
    ],
    "test_cases": [
      {
        "id": "WOT-001",
        "name": "fulltime 8h weekday - no overtime",
        "worked_hours": "8.0",
        "employment_type": "FullTime",
        "base_rate": "28.54",
        "expected_ordinary": "228.32",
        "expected_ot150": "0.00",
        "expected_ot200": "0.00",
        "expected_total": "228.32"
      },
      {
        "id": "WOT-002",
        "name": "fulltime 9h weekday - 1h overtime",
        "worked_hours": "9.0",
        "employment_type": "FullTime",
        "base_rate": "28.54",
        "expected_ordinary": "228.32",
        "expected_ot150": "42.81",
        "expected_ot200": "0.00",
        "expected_total": "271.13"
      },
      {
        "id": "WOT-003",
        "name": "fulltime 10h weekday - 2h overtime",
        "worked_hours": "10.0",
        "employment_type": "FullTime",
        "base_rate": "28.54",
        "expected_ordinary": "228.32",
        "expected_ot150": "85.62",
        "expected_ot200": "0.00",
        "expected_total": "313.94"
      },
      {
        "id": "WOT-004",
        "name": "fulltime 11h weekday - 3h overtime (2h@150%, 1h@200%)",
        "worked_hours": "11.0",
        "employment_type": "FullTime",
        "base_rate": "28.54",
        "expected_ordinary": "228.32",
        "expected_ot150": "85.62",
        "expected_ot200": "57.08",
        "expected_total": "371.02"
      },
      {
        "id": "WOT-005",
        "name": "fulltime 12h weekday - 4h overtime (2h@150%, 2h@200%)",
        "worked_hours": "12.0",
        "employment_type": "FullTime",
        "base_rate": "28.54",
        "expected_ordinary": "228.32",
        "expected_ot150": "85.62",
        "expected_ot200": "114.16",
        "expected_total": "428.10"
      },
      {
        "id": "WCOT-001",
        "name": "casual 8h weekday - no overtime",
        "worked_hours": "8.0",
        "employment_type": "Casual",
        "base_rate": "28.54",
        "expected_ordinary": "285.40",
        "expected_ot_first_2h": "0.00",
        "expected_ot_after_2h": "0.00",
        "expected_total": "285.40"
      },
      {
        "id": "WCOT-002",
        "name": "casual 10h weekday - 2h overtime @ 187.5%",
        "worked_hours": "10.0",
        "employment_type": "Casual",
        "base_rate": "28.54",
        "expected_ordinary": "285.40",
        "expected_ot_first_2h": "107.03",
        "expected_ot_after_2h": "0.00",
        "expected_total": "392.43"
      },
      {
        "id": "WCOT-003",
        "name": "casual 12h weekday - 4h overtime (2h@187.5%, 2h@250%)",
        "worked_hours": "12.0",
        "employment_type": "Casual",
        "base_rate": "28.54",
        "expected_ordinary": "285.40",
        "expected_ot_first_2h": "107.03",
        "expected_ot_after_2h": "142.70",
        "expected_total": "535.13"
      }
    ],
    "passes": true
  },
  {
    "id": "US-4.3",
    "category": "calculation",
    "priority": 3,
    "title": "Weekend Overtime (Saturday/Sunday)",
    "description": "Calculate overtime on weekend days at the correct higher rate",
    "acceptance_criteria": [
      "Weekend overtime is 200% from the first hour (no tiered rates)",
      "Saturday overtime for casuals is 250% (2.0 * 1.25)",
      "Sunday overtime for casuals is 250% (2.0 * 1.25)",
      "Returns single PayLine for all weekend overtime (not tiered)",
      "Audit step records weekend overtime with correct clause reference"
    ],
    "test_cases": [
      {
        "id": "SATOT-001",
        "name": "fulltime 10h Saturday - 2h overtime",
        "day": "Saturday",
        "worked_hours": "10.0",
        "employment_type": "FullTime",
        "base_rate": "28.54",
        "expected_ordinary": "342.48",
        "expected_overtime": "114.16",
        "expected_total": "456.64",
        "note": "Ordinary 8h @ 1.50 = 342.48, OT 2h @ 2.0 = 114.16"
      },
      {
        "id": "SATOT-002",
        "name": "casual 10h Saturday - 2h overtime",
        "day": "Saturday",
        "worked_hours": "10.0",
        "employment_type": "Casual",
        "base_rate": "28.54",
        "expected_ordinary": "399.56",
        "expected_overtime": "142.70",
        "expected_total": "542.26",
        "note": "Ordinary 8h @ 1.75 = 399.56, OT 2h @ 2.5 = 142.70"
      },
      {
        "id": "SUNOT-001",
        "name": "fulltime 10h Sunday - 2h overtime",
        "day": "Sunday",
        "worked_hours": "10.0",
        "employment_type": "FullTime",
        "base_rate": "28.54",
        "expected_ordinary": "399.56",
        "expected_overtime": "114.16",
        "expected_total": "513.72",
        "note": "Ordinary 8h @ 1.75 = 399.56, OT 2h @ 2.0 = 114.16"
      },
      {
        "id": "SUNOT-002",
        "name": "casual 10h Sunday - 2h overtime",
        "day": "Sunday",
        "worked_hours": "10.0",
        "employment_type": "Casual",
        "base_rate": "28.54",
        "expected_ordinary": "456.64",
        "expected_overtime": "142.70",
        "expected_total": "599.34",
        "note": "Ordinary 8h @ 2.0 = 456.64, OT 2h @ 2.5 = 142.70"
      }
    ],
    "passes": true
  },
  {
    "id": "US-4.4",
    "category": "audit",
    "priority": 4,
    "title": "Overtime Audit Trail",
    "description": "Create detailed audit records for all overtime calculations",
    "acceptance_criteria": [
      "Audit trace includes steps in order: base rate, casual loading (if applicable), worked hours, day type, overtime detection, pay lines",
      "OT tier 1 step has rule_id: 'overtime_tier_1'",
      "OT tier 2 step has rule_id: 'overtime_tier_2'",
      "Weekend OT step has rule_id: 'weekend_overtime'",
      "Each step includes clause_ref to Award clause",
      "Input contains hours and base_rate",
      "Output contains multiplier and amount",
      "Reasoning explains the calculation in plain English"
    ],
    "test_cases": [
      {
        "id": "AT-OT-001",
        "name": "12h weekday shift audit trace",
        "worked_hours": "12.0",
        "expected_steps": [
          {"rule_id": "base_rate_lookup", "clause_ref": "14.2"},
          {"rule_id": "daily_overtime_detection", "clause_ref": "22.1(c), 25.1"},
          {"rule_id": "ordinary_hours", "clause_ref": "22.1"},
          {"rule_id": "overtime_tier_1", "clause_ref": "25.1(a)(i)(A)"},
          {"rule_id": "overtime_tier_2", "clause_ref": "25.1(a)(i)(A)"}
        ]
      },
      {
        "id": "AT-OT-002",
        "name": "OT tier 1 step content",
        "expected_input": {"hours": 2.0, "base_rate": 28.54},
        "expected_output": {"multiplier": 1.5, "amount": 85.62},
        "expected_reasoning": "First 2 hours of weekday overtime at 150%"
      },
      {
        "id": "AT-OT-003",
        "name": "OT tier 2 step content",
        "expected_input": {"hours": 2.0, "base_rate": 28.54},
        "expected_output": {"multiplier": 2.0, "amount": 114.16},
        "expected_reasoning": "Overtime after first 2 hours at 200%"
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
- Tiered overtime calculation works correctly
- Audit trail captures all overtime decisions
