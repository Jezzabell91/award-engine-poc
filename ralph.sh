#!/bin/bash
# ralph.sh - Autonomous AI Coding Loop for Award Interpretation Engine
#
# Usage: ./ralph.sh <spec_file> <progress_file> [iterations]
#
# Examples:
#   ./ralph.sh epics/epic1_foundation_spec.md epics/epic1_foundation_progress.txt
#   ./ralph.sh epics/epic2_base_rate_spec.md epics/epic2_base_rate_progress.txt 20
#
# This script implements the Ralph Wiggum technique for autonomous AI coding.
# See: https://www.aihero.dev/tips-for-ai-coding-with-ralph-wiggum

set -e

# Configuration
DEFAULT_ITERATIONS=10

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Print usage
usage() {
    echo -e "${CYAN}Usage: $0 <spec_file> <progress_file> [iterations]${NC}"
    echo ""
    echo "Arguments:"
    echo "  spec_file      Path to the epic specification markdown file"
    echo "  progress_file  Path to the progress tracking text file"
    echo "  iterations     Maximum iterations (default: $DEFAULT_ITERATIONS)"
    echo ""
    echo "Examples:"
    echo "  $0 epics/epic1_foundation_spec.md epics/epic1_foundation_progress.txt"
    echo "  $0 epics/epic2_base_rate_spec.md epics/epic2_base_rate_progress.txt 20"
    echo ""
    echo "Available Epics:"
    echo "  epic1_foundation     - Project Foundation & Core Types"
    echo "  epic2_base_rate      - Base Rate & Casual Loading Calculation"
    echo "  epic3_weekend        - Weekend Penalty Rates"
    echo "  epic4_overtime       - Daily Overtime Rules"
    echo "  epic5_allowances     - Automatic Allowances"
    echo "  epic6_api            - API & Integration"
    exit 1
}

# Check arguments
if [ $# -lt 2 ]; then
    usage
fi

SPEC_FILE="$1"
PROGRESS_FILE="$2"
ITERATIONS=${3:-$DEFAULT_ITERATIONS}

# Validate files exist
if [ ! -f "$SPEC_FILE" ]; then
    echo -e "${RED}Error: Spec file not found: $SPEC_FILE${NC}"
    exit 1
fi

if [ ! -f "$PROGRESS_FILE" ]; then
    echo -e "${YELLOW}Warning: Progress file not found, creating: $PROGRESS_FILE${NC}"
    touch "$PROGRESS_FILE"
fi

# Extract epic name from spec file
EPIC_NAME=$(basename "$SPEC_FILE" _spec.md)

# Temp file for capturing output (to check for completion)
TEMP_OUTPUT=$(mktemp)
trap "rm -f $TEMP_OUTPUT" EXIT

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║          Ralph Wiggum - Autonomous AI Coding Loop           ║${NC}"
echo -e "${BLUE}║            Award Interpretation Engine PoC                   ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}Configuration:${NC}"
echo "  Epic: $EPIC_NAME"
echo "  Spec File: $SPEC_FILE"
echo "  Progress File: $PROGRESS_FILE"
echo "  Max Iterations: $ITERATIONS"
echo ""

# Prompt includes both files
RALPH_PROMPT="@$SPEC_FILE @$PROGRESS_FILE Read the PRD file and follow the Claude Instructions section. Work on ONE user story."

# Main loop
echo -e "${GREEN}Starting Ralph loop...${NC}"
echo ""

for ((i=1; i<=$ITERATIONS; i++)); do
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  Epic: $EPIC_NAME${NC}"
    echo -e "${BLUE}  Iteration $i of $ITERATIONS${NC}"
    echo -e "${BLUE}  $(date '+%Y-%m-%d %H:%M:%S')${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
    echo ""

    # Run Claude Code with streaming output using tee to capture and display
    # --dangerously-skip-permissions allows autonomous operation
    claude --dangerously-skip-permissions -p "$RALPH_PROMPT" 2>&1 | tee "$TEMP_OUTPUT" || true

    echo ""

    # Check for completion signal in captured output
    if grep -q "<promise>COMPLETE</promise>" "$TEMP_OUTPUT"; then
        echo -e "${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
        echo -e "${GREEN}║                    EPIC COMPLETE! 🎉                         ║${NC}"
        echo -e "${GREEN}║           All user stories have been implemented             ║${NC}"
        echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"
        exit 0
    fi

    echo -e "${YELLOW}Iteration $i complete. Continuing...${NC}"
    echo ""

    # Small delay between iterations
    sleep 2
done

echo -e "${YELLOW}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${YELLOW}║              Max iterations ($ITERATIONS) reached                      ║${NC}"
echo -e "${YELLOW}║         Review progress file and run again if needed          ║${NC}"
echo -e "${YELLOW}╚══════════════════════════════════════════════════════════════╝${NC}"
