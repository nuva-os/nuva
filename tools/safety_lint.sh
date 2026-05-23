#!/bin/bash
#
# SAFETY Annotation Lint Tool
#
# Scans all .rs files in kernel/ and hal/ directories for unsafe blocks
# that lack a preceding // SAFETY: comment. Outputs violations as
# file:line pairs and exits with non-zero status if any are found.
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

SCAN_DIRS=("kernel" "hal")
VIOLATIONS=0
TOTAL_UNSAFE=0
ANNOTATED=0

echo "=== SAFETY Annotation Lint ==="
echo "Scanning directories: ${SCAN_DIRS[*]}"
echo ""

for dir in "${SCAN_DIRS[@]}"; do
    find "$PROJECT_ROOT/$dir" -name "*.rs" -type f 2>/dev/null | while read -r file; do
        line_num=0
        prev_line=""
        while IFS= read -r line; do
            line_num=$((line_num + 1))

            if echo "$line" | grep -qE 'unsafe\s*\{'; then
                TOTAL_UNSAFE=$((TOTAL_UNSAFE + 1))
                if echo "$prev_line" | grep -qE '//\s*SAFETY:'; then
                    ANNOTATED=$((ANNOTATED + 1))
                else
                    rel_path="${file#$PROJECT_ROOT/}"
                    echo "MISSING: $rel_path:$line_num"
                    VIOLATIONS=$((VIOLATIONS + 1))
                fi
            fi
            prev_line="$line"
        done < "$file"
    done
done

echo ""
echo "=== Summary ==="
echo "Total unsafe blocks: $TOTAL_UNSAFE"
echo "Annotated with SAFETY: $ANNOTATED"
echo "Missing SAFETY annotation: $VIOLATIONS"

if [ "$VIOLATIONS" -gt 0 ]; then
    echo ""
    echo "FAIL: $VIOLATIONS unsafe block(s) missing SAFETY annotation"
    exit 1
else
    echo ""
    echo "PASS: All unsafe blocks have SAFETY annotations"
    exit 0
fi
