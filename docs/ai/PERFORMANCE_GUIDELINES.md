# Performance & Query Optimization Guidelines

This document outlines the performance standards, token budget rules, context scanning limits, async pipeline deadlines, and XML rendering performance requirements for `prompt_enhancement_engine`.

---

## 1. Token Budget & Evidence Scanning Limits

The most frequent source of pipeline latency is oversized context evidence or oversized prompt payloads exceeding token budgets. Every pipeline execution (`execute_pipeline`) must respect the configured budget.

### Budget Tracking (`context/src/budget.rs`)
- `BudgetTracker` monitors total tokens consumed by evidence scanning (`context/scanner.rs`, `context/collector.rs`) and final XML payload size.
- The total budget is configured via `Config::budgets.total_deadline_ms` (time-based) and token limits (if configured in `Config`).
- If evidence scanning exceeds the budget, `execute_pipeline` must abort and return `TerminalStatus::DeadlineExceeded` without rendering partial XML.

### Context Scanner Limits (`context/src/scanner.rs`)
- `FileScanner` reads files with line limits and byte limits. Configure limits in `Config` or default safely.
- Never scan files larger than a safe threshold (e.g., `> 1MB`) without explicit approval.
- Scanning must stop (`break`) when budget is exceeded; never accumulate all evidence before checking budget.

---

## 2. Prompt Size Optimization (`XmlRenderer`)

- `XmlRenderer` produces XML strings with `String::with_capacity(2048)`. For very large inputs, the capacity may need adjustment; measure with `tests/test_pe_run.sh` and `cargo test` if performance degrades.
- Avoid redundant `String` allocations (`.trim()` creates temporary `String`; reuse references where possible). Verify with `cargo clippy` for unnecessary allocations.
- XML tag structure (`<task>`, `<acceptance_criteria>`, etc.) must not include unnecessary whitespace or indentation that increases payload size without improving readability for downstream agents.

### Lean Querysets / Evidence Bundles (`context/src/evidence.rs`)
- When collecting repository evidence (`EvidenceItem`, `EvidenceBundle`), use `.only()` and `.defer()` equivalents (if applicable to file scanning) to limit read size.
- Filter evidence by relevance (`relevant_files` from user input) before bundling; do not scan the entire repository (`.`) unless explicitly required.

---

## 3. Async Pipeline & Deadline Enforcement (`cli/src/pipeline.rs`, `cli/src/main.rs`)

- `execute_pipeline` runs inside `tokio::time::timeout(Duration::from_millis(config.budgets.total_deadline_ms), ...)`.
- The deadline supervisor ensures no pipeline execution exceeds the configured time. If exceeded, return `TerminalStatus::DeadlineExceeded` (exit code 4) and abort all sub-tasks.
- Sub-tasks (`context/scanner.rs`, `context/collector.rs`, `adapters/writer.rs`) must check for cancellation tokens or budget exhaustion before completing long operations.
- Never call synchronous blocking file I/O (`fs::read_to_string`) inside `execute_pipeline` without wrapping in `tokio::task::spawn_blocking` or using async file APIs.

---

## 4. XML Rendering Performance (`core/src/render.rs`)

- `XmlRenderer::render()` creates output with `String::with_capacity(2048)`. For large payloads, consider measuring output size and adjusting capacity or chunking if needed.
- `escape_xml_text()` performs `.replace()` operations sequentially. For very large strings (`> 10KB`), measure performance; if degradation is observed, consider an optimized replacement strategy.
- `render()` verifies `required_fields()` before building output. This prevents partial XML generation for invalid inputs, saving time and token budget.
- Do not render XML tags (`<verify>`, `<context>`) that are not required by the `TemplateContract` unless explicitly present in the contract.

---

## 5. Concurrent Execution & Lock Overhead (`cli/src/lock.rs`)

- `SingleFlightLock::acquire()` creates or opens a file (`/tmp/pe_default.lock` or custom). Lock acquisition time must be minimal (`< 10ms`).
- If lock acquisition is slow, verify the file system (`/tmp`) is not overloaded or read-only.
- Lock release (`drop`) must occur promptly when the binary exits; use `std::mem::drop` or rely on RAII (`Drop` trait) if implemented.
- Concurrent executions must exit quickly (`TerminalStatus::Busy`) without consuming budget or rendering partial XML.

---

## 6. Performance Verification Before PR

Before submitting any performance-sensitive change:

```bash
# Build and verify binary size (release builds should be < 10MB for CLI)
ls -lh target/release/pe

# Run full pipeline test with deadline verification
bash tests/test_pe_run.sh

# Verify async pipeline terminates within budget
# (manual: set Config.budgets.total_deadline_ms to a low value, verify exit code 4)

# Check for unnecessary allocations (clippy warnings)
cargo clippy --workspace --all-targets -- -D warnings
```
