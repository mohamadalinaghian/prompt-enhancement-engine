# Quick Commands Cheat Sheet

This document compiles common CLI commands used for local development, testing, template verification, formatting, security auditing, and binary operations in `prompt_enhancement_engine`.

---

## 1. Build & Development Stack

```bash
# Build debug binary (entry point: crates/cli/src/main.rs)
cargo build

# Build release binary (optimized, stripped debug symbols)
cargo build --release

# Check compilation without producing binary artifacts
cargo check --workspace

# Check all workspace members with verbose diagnostics
cargo check --workspace --verbose
```

---

## 2. Testing & Quality Assurance

```bash
# Run full workspace test suite (single-thread execution for determinism)
cargo test --workspace -- --test-threads=1

# Run specific package tests (e.g., cli, core, context)
cargo test -p pe-cli
cargo test -p pe-core -p pe-context

# Run golden prompt tests (verify rendered XML matches expected payloads)
cargo test -p pe-cli tests::golden_prompts_test

# Run concurrency / single-flight lock tests
cargo test -p pe-cli tests::concurrency_tests

# Run security tests (secrets, XML injection, lock behavior)
cargo test -p pe-cli tests::security_tests

# Run shell integration verification suite
bash tests/test_pe_run.sh

# Verify binary output and exit-code mapping (manual smoke test)
echo "Fix null pointer crash in user login handler" | target/debug/pe --no-llm --lock-path /tmp/test_pe.lock
```

---

## 3. Code Formatting & Security Scans

```bash
# Format all source files with default Rustfmt settings
cargo fmt

# Check formatting without modifying files
cargo fmt -- --check

# Static analysis / linting (treat warnings as errors)
cargo clippy --workspace --all-targets -- -D warnings

# Security / secrets verification (manual inspection of SecretScanner usage)
# Verify escape_xml_text handles injection attempts in templates
# Inspect scripts/karabiner/pe.json for syntax validity
python3 -m json.tool scripts/karabiner/pe.json > /dev/null && echo "Karabiner JSON valid"
```

---

## 4. Template & Contract Inspection

```bash
# Inspect template definitions and contracts
cat templates/README.md
cat crates/core/src/templates/definitions.rs
cat crates/core/src/templates/contract.rs

# View a single session archetype
cat templates/implementation_refactoring.md

# Verify XML tag pairing and contract completeness (manual review)
# Check all 15 TemplateId values in types.rs match files in templates/
```

---

## 5. Binary Execution & Pipeline Verification

```bash
# Execute with stdin input and deterministic offline mode
cat prompts/example.md | target/debug/pe --no-llm

# Execute with custom config override
cat input.txt | target/debug/pe --no-llm --config ./my_config.toml

# Execute with custom lockfile path
cat input.txt | target/debug/pe --no-llm --lock-path /tmp/custom.lock

# Capture stdout XML payload and stderr status separately
OUTPUT=$(cat input.md | target/debug/pe --no-llm 2>/dev/null)
STATUS=$(cat input.md | target/debug/pe --no-llm 2>&1 > /dev/null; echo $?)

# Verify script syntax (no execution)
bash -n scripts/pe_run.sh
```

---

## 6. Script Integration & MacOS Workflow

```bash
# Test clipboard capture and paste replacement (manual)
# Requires macOS System Events and active text editor
bash scripts/pe_run.sh

# Inspect Karabiner-Elements JSON binding
cat scripts/karabiner/pe.json

# List all registered templates (manual grep)
grep "TemplateId" crates/core/src/types.rs
```
