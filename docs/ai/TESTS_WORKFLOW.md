# Tests Workflow & Quality Assurance Guide

This document outlines the testing conventions, runner configuration, test categories, and best practices for `prompt_enhancement_engine`.

---

## 1. Testing Philosophy & Invariants

- **Real Template Contracts Enforced**: Tests must verify that `TemplateContract`, `XmlRenderer`, and `FieldContract` behave exactly as specified in `definitions.rs` and `templates/*.md`. There is no mock contract override; broken contracts fail immediately.
- **Isolation**: Each `cargo test` must be completely isolated and reproducible. Avoid inter-test dependencies (`tests/test_pe_run.sh` creates temporary directories using `mktemp`).
- **XML Format Integrity**: Tests must verify that `XmlRenderer` output conforms to the XML wire format defined in `API_CONTRACTS.md` (`<task>`, `<acceptance_criteria>`, `<scope>`, `<rules>`, `<verify>`, `<report>`).
- **Anti-Hallucination Verification**: Golden prompt tests verify that templates do not invent content, bypass `escape_xml_text`, or alter `StaticSections`.

---

## 2. Configuration & Test Runners

Configured implicitly by `Cargo.toml` workspace and `tests/` directory structure:
- **Workspace**: `members = ["crates/core", "crates/context", "crates/adapters", "crates/cli"]`
- **Async Mode**: `tokio::main` and `tokio::time::timeout` are tested synchronously (`cargo test`) unless explicitly using `#[tokio::test]`.
- **File Matching**: `tests/*.rs` (crate-level), `tests/*.sh` (shell integration).

### Running Tests

```bash
# Run all workspace tests (deterministic single-thread)
cargo test --workspace -- --test-threads=1

# Run specific crate tests
cargo test -p pe-core
cargo test -p pe-context -p pe-adapters
cargo test -p pe-cli

# Run specific test file (if organized by module)
cargo test -p pe-cli --lib

# Run golden prompt verification tests (XML payload matches expected output)
cargo test -p pe-cli -- golden_prompts_test

# Run concurrency / lock tests
cargo test -p pe-cli -- concurrency_tests

# Run security / injection tests
cargo test -p pe-cli -- security_tests

# Run shell integration verification
bash tests/test_pe_run.sh
```

---

## 3. Test Markers & Categories

The repository defines implicit categories via file organization and module names (no `pytest` markers, but analogous structure):

### Golden Prompt Tests (`tests/golden_prompts_test.rs`)
These verify `XmlRenderer` output against expected XML payloads for standard input scenarios:
- Input text passes through `execute_pipeline`
- Output XML contains all required tags (`<task>`, `<scope>`, `<rules>`, `<report>`)
- `StaticSections` (`rules`, `out_of_scope`, `report`) remain unchanged
- Dynamic fields (`task`) are properly escaped (`escape_xml_text`)

### Security Tests (`tests/security_tests.rs`)
These verify anti-injection, secret scanning, and lock behavior:
- `escape_xml_text` neutralizes `<script>` payloads
- `SecretScanner` redacts `[[SECRET_PE_*]]` placeholders
- `SingleFlightLock` prevents concurrent executions
- Lockfile is cleaned up after exit (`drop`)

### Concurrency Tests (`tests/concurrency_tests.rs`)
These verify `SingleFlightLock` under concurrent invocation:
- Multiple `execute_pipeline` instances cannot run simultaneously
- Second invocation exits with `TerminalStatus::Busy` (exit code 3)
- Lockfile is released after first instance exits

### Shell Integration Tests (`tests/test_pe_run.sh`)
This verifies the macOS workflow:
- `scripts/karabiner/pe.json` is valid JSON (`python3 -m json.tool`)
- `scripts/pe_run.sh` passes bash syntax check (`bash -n`)
- Binary discovery (`target/debug/pe` or `target/release/pe`) succeeds
- Binary output (exit code 0) produces XML payload containing `<task session=`
- Empty input produces exit code 1 (`TerminalStatus::InputEmpty`)

---

## 4. Test Data Management & Fixtures

- **No Factory-Boy Equivalent**: This repository uses `String` inputs (stdin) rather than database models. Create temporary directories (`mktemp`) for lockfile and payload tests.
- **Temporary Directories**: All temporary resources must use `trap 'rm -rf "..."' EXIT` to ensure cleanup, even on failure.
- **Time Control**: `tokio::time::timeout` is controlled via `Config::budgets.total_deadline_ms`. For deterministic tests, set a fixed budget value (e.g., `Duration::from_millis(10000)`) rather than relying on system time.

---

## 5. Verification Checklist Before Opening a PR

Before submitting any change (template, `.rs`, script, or test), all agents must execute:

1. `cargo check --workspace` — zero compilation errors across all crates.
2. `cargo fmt -- --check` — formatting is compliant.
3. `cargo clippy --workspace --all-targets -- -D warnings` — no warnings.
4. `cargo test --workspace -- --test-threads=1` — full suite passes with 0 failures.
5. `bash tests/test_pe_run.sh` — shell integration passes.
6. `python3 -m json.tool scripts/karabiner/pe.json > /dev/null` — Karabiner JSON valid.
7. Manual verification: If `templates/*.md` or `definitions.rs` changed, verify `TemplateId` count matches file count and `FieldContract` names match dynamic placeholders.
