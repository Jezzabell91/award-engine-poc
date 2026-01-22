#!/bin/bash
# ralph-all.sh - Run all epics sequentially with streaming output
#
# Usage: ./ralph-all.sh [iterations_per_epic]
#
# This script runs Ralph for each epic in order (1-6).
# Epics must be completed in order due to dependencies.

set -e

# Configuration
DEFAULT_ITERATIONS=15

ITERATIONS=${1:-$DEFAULT_ITERATIONS}

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║         Award Interpretation Engine - Full Build             ║${NC}"
echo -e "${CYAN}║              Running All Epics Sequentially                  ║${NC}"
echo -e "${CYAN}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}Iterations per epic: $ITERATIONS${NC}"
echo ""

# Define epics in order
declare -a EPICS=(
    "epic1_foundation:Epic 1 - Project Foundation & Core Types"
    "epic2_base_rate:Epic 2 - Base Rate & Casual Loading"
    "epic3_weekend_penalties:Epic 3 - Weekend Penalty Rates"
    "epic4_overtime:Epic 4 - Daily Overtime Rules"
    "epic5_allowances:Epic 5 - Automatic Allowances"
    "epic6_api:Epic 6 - API & Integration"
)

FAILED_EPICS=()
COMPLETED_EPICS=()

for epic_entry in "${EPICS[@]}"; do
    # Parse epic name and description
    epic_name="${epic_entry%%:*}"
    epic_desc="${epic_entry#*:}"

    SPEC_FILE="epics/${epic_name}_spec.md"
    PROGRESS_FILE="epics/${epic_name}_progress.txt"

    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  Starting: $epic_desc${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo ""

    # Run Ralph for this epic (ralph.sh already streams output)
    if ./ralph.sh "$SPEC_FILE" "$PROGRESS_FILE" "$ITERATIONS"; then
        COMPLETED_EPICS+=("$epic_desc")
        echo -e "${GREEN}✓ $epic_desc completed successfully${NC}"
    else
        FAILED_EPICS+=("$epic_desc")
        echo -e "${RED}✗ $epic_desc did not complete within $ITERATIONS iterations${NC}"
        echo ""
        echo -e "${RED}═══════════════════════════════════════════════════════════════${NC}"
        echo -e "${RED}  STOPPING: Cannot continue - subsequent epics depend on this one${NC}"
        echo -e "${RED}═══════════════════════════════════════════════════════════════${NC}"
        echo ""
        echo -e "${YELLOW}To resume, run with more iterations:${NC}"
        echo -e "  ./ralph.sh $SPEC_FILE $PROGRESS_FILE 30"
        echo ""
        break
    fi

    echo ""
done

# Summary
echo ""
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}                         SUMMARY                               ${NC}"
echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

if [ ${#COMPLETED_EPICS[@]} -gt 0 ]; then
    echo -e "${GREEN}Completed Epics:${NC}"
    for epic in "${COMPLETED_EPICS[@]}"; do
        echo -e "  ${GREEN}✓${NC} $epic"
    done
    echo ""
fi

if [ ${#FAILED_EPICS[@]} -gt 0 ]; then
    echo -e "${RED}Incomplete Epics (may need more iterations):${NC}"
    for epic in "${FAILED_EPICS[@]}"; do
        echo -e "  ${RED}✗${NC} $epic"
    done
    echo ""
    echo -e "${YELLOW}Tip: Run individual epics with more iterations:${NC}"
    echo -e "  ./ralph.sh epics/<epic>_spec.md epics/<epic>_progress.txt 30"
    exit 1
fi

echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              ALL EPICS COMPLETE! 🎉                          ║${NC}"
echo -e "${GREEN}║         Award Interpretation Engine PoC is ready             ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
