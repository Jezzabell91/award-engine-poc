# Award Interpretation Engine - Proof of Concept

A high-performance Award Interpretation Engine for the Aged Care Award 2010 (MA000018), built in Rust.

## Project Structure

```
award-engine-poc/
├── README.md                    # This file
├── ralph.sh                     # Autonomous coding loop script
├── ralph-all.sh                 # Run all epics sequentially
├── epics/                       # Epic specifications and progress
│   ├── epic1_foundation_spec.md
│   ├── epic1_foundation_progress.txt
│   ├── epic2_base_rate_spec.md
│   ├── epic2_base_rate_progress.txt
│   ├── epic3_weekend_penalties_spec.md
│   ├── epic3_weekend_penalties_progress.txt
│   ├── epic4_overtime_spec.md
│   ├── epic4_overtime_progress.txt
│   ├── epic5_allowances_spec.md
│   ├── epic5_allowances_progress.txt
│   ├── epic6_api_spec.md
│   └── epic6_api_progress.txt
├── config/                      # Award configuration (YAML)
│   └── ma000018/
├── src/                         # Source code (to be created)
├── tests/                       # Integration tests
└── benches/                     # Performance benchmarks
```

## Epics Overview

| Epic | Name | Duration | Description |
|------|------|----------|-------------|
| 1 | Project Foundation | 2-3 days | Rust project setup, core types, config loader |
| 2 | Base Rate & Casual Loading | 2-3 days | Base rate lookup, 25% casual loading |
| 3 | Weekend Penalty Rates | 2-3 days | Saturday (150%/175%), Sunday (175%/200%) |
| 4 | Daily Overtime Rules | 2-3 days | 8-hour threshold, tiered OT rates |
| 5 | Automatic Allowances | 1-2 days | Laundry allowance with weekly cap |
| 6 | API & Integration | 2-3 days | REST API, integration tests, benchmarks |

## Using Ralph (Autonomous Coding)

Ralph is an autonomous coding loop that works through user stories one at a time.

### Run a Single Epic

```bash
# Run Epic 1: Project Foundation
./ralph.sh epics/epic1_foundation_spec.md epics/epic1_foundation_progress.txt

# Run Epic 2 with 20 iterations
./ralph.sh epics/epic2_base_rate_spec.md epics/epic2_base_rate_progress.txt 20
```

### Run All Epics Sequentially

```bash
./ralph-all.sh
```

### How Ralph Works

1. Reads the spec file (`*_spec.md`) containing user stories in JSON format
2. Reads the progress file (`*_progress.txt`) to see what's done
3. Picks ONE incomplete user story (by priority)
4. Implements that feature
5. Runs tests to verify
6. Updates the spec (marks story as passing)
7. Appends to progress file
8. Makes a git commit
9. Repeats until all stories pass or max iterations reached

### Completion Signal

When all user stories in an epic have `"passes": true`, Ralph outputs:
```
<promise>COMPLETE</promise>
```

## Award Rules Implemented

### Base Rates (Clause 14.2)
- Direct Care Employee Level 3: $28.54/hour (effective 2025-07-01)

### Casual Loading (Clause 10.4(b))
- 25% loading on base rate for casual employees

### Weekend Penalties (Clause 23.1, 23.2)
| Day | Full-time/Part-time | Casual |
|-----|---------------------|--------|
| Saturday | 150% | 175% |
| Sunday | 175% | 200% |

### Daily Overtime (Clause 25.1)
- Threshold: 8 hours per day
- First 2 hours: 150% (non-casual), 187.5% (casual)
- After 2 hours: 200% (non-casual), 250% (casual)
- Weekend overtime: 200% from first hour

### Allowances (Clause 15.2(b))
- Laundry: $0.32 per shift, capped at $1.49 per week

## Performance Targets

| Metric | Target |
|--------|--------|
| Single shift calculation | < 1ms p99 |
| 1000 shift batch | < 100ms |
| Memory per calculation | < 1KB |
| Test coverage | > 90% |

## Technology Stack

- **Language**: Rust (latest stable)
- **Decimal Math**: rust_decimal
- **Date/Time**: chrono
- **HTTP Server**: axum
- **Serialization**: serde, serde_json, serde_yaml
- **Testing**: built-in + proptest
- **Benchmarking**: criterion

## Getting Started

1. Ensure Rust latest stable is installed
2. Clone the repository
3. Run Epic 1 to scaffold the project:
   ```bash
   ./ralph.sh epics/epic1_foundation_spec.md epics/epic1_foundation_progress.txt
   ```
4. Continue with remaining epics in order

## API Endpoints (Epic 6)

| Method | Path | Description |
|--------|------|-------------|
| POST | /calculate | Submit timesheet, receive calculated pay |
| GET | /health | Service health check |
| GET | /info | Supported awards and classifications |

## License

Proprietary - Tanda
