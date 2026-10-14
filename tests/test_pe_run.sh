#!/usr/bin/env bash
# ==============================================================================
# Prompt Enhancement Engine — Shell Integration Verification Suite
# ==============================================================================
set -euo pipefail

echo "=== 1. Testing Karabiner JSON validity ==="
python3 -m json.tool scripts/karabiner/pe.json > /dev/null
echo "✓ scripts/karabiner/pe.json is valid JSON"

echo "=== 2. Testing scripts/pe_run.sh bash syntax ==="
bash -n scripts/pe_run.sh
echo "✓ scripts/pe_run.sh passes bash syntax check"

echo "=== 3. Testing 'pe' binary discovery ==="
REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [ -x "$REPO_DIR/target/debug/pe" ]; then
    echo "✓ Found debug binary: $REPO_DIR/target/debug/pe"
else
    echo "Building debug binary for verification..."
    cargo build -p pe-cli
    echo "✓ Built and verified debug binary"
fi

echo "=== 4. Testing binary output and exit-code mapping ==="
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

LOCK_FILE="$TMP_DIR/test_pe.lock"

# Test success (code 0)
OUTPUT=$(echo "Fix null pointer crash in user login handler" | "$REPO_DIR/target/debug/pe" --no-llm --lock-path "$LOCK_FILE" 2>/dev/null)
CODE=$?
if [ "$CODE" -eq 0 ] && echo "$OUTPUT" | grep -q "<task>"; then
    echo "✓ Success scenario: exit code 0, rendered XML"
else
    echo "✗ Success scenario failed with exit code $CODE"
    exit 1
fi

# Test empty input (code 1)
set +e
echo "" | "$REPO_DIR/target/debug/pe" --no-llm --lock-path "$LOCK_FILE" >/dev/null 2>/dev/null
CODE=$?
set -e
if [ "$CODE" -eq 1 ]; then
    echo "✓ Empty input scenario: exit code 1"
else
    echo "✗ Empty input scenario failed with code $CODE"
    exit 1
fi

echo "All shell and macOS integration checks passed!"
