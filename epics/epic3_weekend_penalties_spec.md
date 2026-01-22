# Epic 3: Weekend Penalty Rates - PRD

## Claude Instructions

You are implementing Saturday and Sunday penalty rate calculations for the Award Interpretation Engine.
This epic introduces shift segmentation for overnight shifts and the "most specific rule wins" principle.

### How To Work On This PRD

1. Review the user stories below to understand all tasks and their status.
2. Review `epic3_weekend_penalties_progress.txt` to see what has already been done.
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
7. **APPEND** your progress to `epic3_weekend_penalties_progress.txt` (do not modify previous entries).
8. Make a git commit with a descriptive message.

### Quality Requirements

- All monetary calculations must use `rust_decimal::Decimal`
- Penalty rates are NOT cumulative with casual loading (rate already includes it)
- Overnight shifts must be split at midnight boundaries
- Include clause references from the Aged Care Award 2010
- All public functions must have rustdoc comments

### Stop Conditions

- If **ALL** user stories have `"passes": true`, output `<promise>COMPLETE</promise>`
- **ONLY** work on ONE user story per iteration
- Do **NOT** skip ahead or combine multiple stories

### Key Award References

| Rule | Clause | Rate (Non-Casual) | Rate (Casual) |
|------|--------|-------------------|---------------|
| Saturday | 23.1, 23.2(a) | 150% | 175% |
| Sunday | 23.1, 23.2(b) | 175% | 200% |

### Important Note on Casual Rates

Weekend penalty rates for casuals are **NOT** ordinary rate + casual loading + penalty.
The casual rate already includes the loading. Per clause 23.2:
- Saturday casual: 175% of ordinary hourly rate (not 150% + 25%)
- Sunday casual: 200% of ordinary hourly rate (not 175% + 25%)

### Dependencies

- Epic 1 must be complete (models and config)
- Epic 2 must be complete (base rate and casual loading)

---

## Overview

**Epic Owner**: Developer B
**Duration**: 2-3 days
**Dependencies**: Epic 2

Implement Saturday and Sunday penalty rate calculations. This introduces the concept of shift segmentation for overnight shifts crossing day boundaries.

---

## User Stories

```json
[
  {
    "id": "US-3.1",
    "category": "calculation",
    "priority": 1,
    "title": "Saturday Penalty Rate",
    "description": "Apply Saturday penalty rates for work performed on Saturdays",
    "acceptance_criteria": [
      "calculate_saturday_pay(shift_segment, employee, config) returns PayLine",
      "Full-time employees get 150% multiplier",
      "Part-time employees get 150% multiplier",
      "Casual employees get 175% multiplier (NOT 150% + casual loading)",
      "PayLine.category is Saturday for non-casuals",
      "PayLine.category is SaturdayCasual for casuals",
      "PayLine.clause_ref is '23.1' for non-casuals",
      "PayLine.clause_ref is '23.2(a)' for casuals",
      "Audit step records the penalty calculation with correct multiplier"
    ],
    "test_cases": [
      {
        "id": "SAT-001",
        "name": "fulltime 8h Saturday",
        "day": "Saturday",
        "hours": "8.0",
        "employment_type": "FullTime",
        "base_rate": "28.54",
        "expected_multiplier": "1.50",
        "expected_amount": "342.48"
      },
      {
        "id": "SAT-002",
        "name": "parttime 8h Saturday",
        "day": "Saturday",
        "hours": "8.0",
        "employment_type": "PartTime",
        "base_rate": "28.54",
        "expected_multiplier": "1.50",
        "expected_amount": "342.48"
      },
      {
        "id": "SAT-003",
        "name": "casual 8h Saturday",
        "day": "Saturday",
        "hours": "8.0",
        "employment_type": "Casual",
        "base_rate": "28.54",
        "expected_multiplier": "latest stable",
        "expected_amount": "399.56"
      },
      {
        "id": "SAT-004",
        "name": "fulltime 4h Saturday",
        "day": "Saturday",
        "hours": "4.0",
        "employment_type": "FullTime",
        "base_rate": "28.54",
        "expected_multiplier": "1.50",
        "expected_amount": "171.24"
      },
      {
        "id": "SAT-005",
        "name": "casual 6.5h Saturday",
        "day": "Saturday",
        "hours": "6.5",
        "employment_type": "Casual",
        "base_rate": "28.54",
        "expected_multiplier": "latest stable",
        "expected_amount": "324.64"
      }
    ],
    "passes": false
  },
  {
    "id": "US-3.2",
    "category": "calculation",
    "priority": 2,
    "title": "Sunday Penalty Rate",
    "description": "Apply Sunday penalty rates for work performed on Sundays",
    "acceptance_criteria": [
      "calculate_sunday_pay(shift_segment, employee, config) returns PayLine",
      "Full-time employees get 175% multiplier",
      "Part-time employees get 175% multiplier",
      "Casual employees get 200% multiplier (NOT 175% + casual loading)",
      "PayLine.category is Sunday for non-casuals",
      "PayLine.category is SundayCasual for casuals",
      "PayLine.clause_ref is '23.1' for non-casuals",
      "PayLine.clause_ref is '23.2(b)' for casuals",
      "Audit step records the penalty calculation"
    ],
    "test_cases": [
      {
        "id": "SUN-001",
        "name": "fulltime 8h Sunday",
        "day": "Sunday",
        "hours": "8.0",
        "employment_type": "FullTime",
        "base_rate": "28.54",
        "expected_multiplier": "latest stable",
        "expected_amount": "399.56"
      },
      {
        "id": "SUN-002",
        "name": "parttime 8h Sunday",
        "day": "Sunday",
        "hours": "8.0",
        "employment_type": "PartTime",
        "base_rate": "28.54",
        "expected_multiplier": "latest stable",
        "expected_amount": "399.56"
      },
      {
        "id": "SUN-003",
        "name": "casual 8h Sunday",
        "day": "Sunday",
        "hours": "8.0",
        "employment_type": "Casual",
        "base_rate": "28.54",
        "expected_multiplier": "2.00",
        "expected_amount": "456.64"
      },
      {
        "id": "SUN-004",
        "name": "fulltime 4h Sunday",
        "day": "Sunday",
        "hours": "4.0",
        "employment_type": "FullTime",
        "base_rate": "28.54",
        "expected_multiplier": "latest stable",
        "expected_amount": "199.78"
      },
      {
        "id": "SUN-005",
        "name": "casual 6.5h Sunday",
        "day": "Sunday",
        "hours": "6.5",
        "employment_type": "Casual",
        "base_rate": "28.54",
        "expected_multiplier": "2.00",
        "expected_amount": "371.02"
      }
    ],
    "passes": false
  },
  {
    "id": "US-3.3",
    "category": "calculation",
    "priority": 3,
    "title": "Overnight Shift Spanning Saturday/Sunday",
    "description": "Split overnight shifts at midnight and apply correct penalty to each segment",
    "acceptance_criteria": [
      "Shifts crossing midnight are split into segments",
      "Each segment gets the correct penalty for its day",
      "Saturday portion (before midnight) gets Saturday rate",
      "Sunday portion (after midnight) gets Sunday rate",
      "Total pay equals sum of segment pay lines",
      "Audit trace shows shift segmentation step",
      "Audit trace shows separate penalty calculations per segment"
    ],
    "test_cases": [
      {
        "id": "OVN-001",
        "name": "fulltime Sat 22:00 to Sun 06:00",
        "start": "Saturday 22:00",
        "end": "Sunday 06:00",
        "employment_type": "FullTime",
        "base_rate": "28.54",
        "expected_segments": [
          {"day": "Saturday", "hours": "2.0", "multiplier": "1.50", "amount": "85.62"},
          {"day": "Sunday", "hours": "6.0", "multiplier": "latest stable", "amount": "299.67"}
        ],
        "expected_total": "385.29"
      },
      {
        "id": "OVN-002",
        "name": "casual Sat 22:00 to Sun 06:00",
        "start": "Saturday 22:00",
        "end": "Sunday 06:00",
        "employment_type": "Casual",
        "base_rate": "28.54",
        "expected_segments": [
          {"day": "Saturday", "hours": "2.0", "multiplier": "latest stable", "amount": "99.89"},
          {"day": "Sunday", "hours": "6.0", "multiplier": "2.00", "amount": "342.48"}
        ],
        "expected_total": "442.37"
      },
      {
        "id": "OVN-003",
        "name": "fulltime Fri 22:00 to Sat 06:00",
        "start": "Friday 22:00",
        "end": "Saturday 06:00",
        "employment_type": "FullTime",
        "base_rate": "28.54",
        "expected_segments": [
          {"day": "Friday", "hours": "2.0", "multiplier": "1.00", "amount": "57.08"},
          {"day": "Saturday", "hours": "6.0", "multiplier": "1.50", "amount": "256.86"}
        ],
        "expected_total": "313.94"
      },
      {
        "id": "OVN-004",
        "name": "casual Fri 22:00 to Sat 06:00",
        "start": "Friday 22:00",
        "end": "Saturday 06:00",
        "employment_type": "Casual",
        "base_rate": "28.54",
        "expected_segments": [
          {"day": "Friday", "hours": "2.0", "multiplier": "1.25", "amount": "71.35"},
          {"day": "Saturday", "hours": "6.0", "multiplier": "latest stable", "amount": "299.67"}
        ],
        "expected_total": "371.02"
      },
      {
        "id": "OVN-005",
        "name": "fulltime Sun 22:00 to Mon 06:00",
        "start": "Sunday 22:00",
        "end": "Monday 06:00",
        "employment_type": "FullTime",
        "base_rate": "28.54",
        "expected_segments": [
          {"day": "Sunday", "hours": "2.0", "multiplier": "latest stable", "amount": "99.89"},
          {"day": "Monday", "hours": "6.0", "multiplier": "1.00", "amount": "171.24"}
        ],
        "expected_total": "271.13"
      }
    ],
    "passes": false
  },
  {
    "id": "US-3.4",
    "category": "utility",
    "priority": 4,
    "title": "Day Detection Logic",
    "description": "Correctly identify which day each hour of a shift belongs to",
    "acceptance_criteria": [
      "DayType enum has variants: Weekday, Saturday, Sunday",
      "get_day_type(datetime) returns correct DayType for any datetime",
      "Monday through Friday return DayType::Weekday",
      "Saturday returns DayType::Saturday",
      "Sunday returns DayType::Sunday",
      "ShiftSegment struct has: start_time, end_time, day_type, hours",
      "segment_by_day(shift) returns Vec<ShiftSegment>",
      "Segments are ordered chronologically",
      "Sum of segment hours equals shift.worked_hours()",
      "No segment crosses midnight",
      "Weekday shift returns single segment with DayType::Weekday"
    ],
    "test_cases": [
      {
        "id": "DD-001",
        "name": "Monday is Weekday",
        "datetime": "2026-01-12 09:00",
        "expected": "Weekday"
      },
      {
        "id": "DD-002",
        "name": "Saturday is Saturday",
        "datetime": "2026-01-17 15:00",
        "expected": "Saturday"
      },
      {
        "id": "DD-003",
        "name": "Sunday is Sunday",
        "datetime": "2026-01-18 08:00",
        "expected": "Sunday"
      },
      {
        "id": "DD-004",
        "name": "Saturday 23:59 is Saturday",
        "datetime": "2026-01-17 23:59",
        "expected": "Saturday"
      },
      {
        "id": "DD-005",
        "name": "Sunday 00:00 is Sunday",
        "datetime": "2026-01-18 00:00",
        "expected": "Sunday"
      },
      {
        "id": "DD-006",
        "name": "weekday shift returns single segment",
        "shift": "Wednesday 09:00 to 17:00",
        "expected_segments": 1
      },
      {
        "id": "DD-007",
        "name": "overnight shift returns two segments",
        "shift": "Saturday 22:00 to Sunday 06:00",
        "expected_segments": 2
      }
    ],
    "passes": true
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
- Overnight shift splitting works correctly
