# Async Pipeline & Concurrent Execution Guide

This repository uses `tokio::main` and `tokio::time::timeout` for async execution (`cli/src/main.rs`, `cli/src/pipeline.rs`). There is no WebSocket server (`Daphne`, `Channels`, `Redis`) or real-time bidirectional messaging. Instead, "real-time" refers to the async pipeline's concurrent execution behavior (`execute_pipeline`), deadline supervision, and single-flight lock management (`SingleFlightLock`).

---

## 1. Async Stack & Pipeline Architecture

The pipeline is powered by:
- **Runtime**: `tokio::main` (`cli/src/main.rs`).
- **Pipeline Execution**: `execute_pipeline` (`cli/src/pipeline.rs`) — async function that processes stdin input through `context/scanner.rs`, `context/collector.rs`, `adapters/writer.rs`, and `core/src/render.rs`.
- **Deadline Supervisor**: `tokio::time::timeout(Duration::from_millis(config.budgets.total_deadline_ms), execute_pipeline(...))` ensures no execution exceeds the configured deadline.
- **Lock Management**: `SingleFlightLock` (`cli/src/lock.rs`) prevents concurrent executions using file-based locking (`/tmp/pe_default.lock` or custom `--lock-path`).

---

## 2. Lock Middleware (`cli/src/lock.rs`)

`SingleFlightLock` manages concurrent access to the binary:
- **Lock Acquisition (`SingleFlightLock::acquire`)**: Opens or creates the lock file. If the file exists and is held by another process, acquisition fails.
- **Lock Release (`Drop`)**: The lock is released when the `SingleFlightLock` instance is dropped (end of scope or binary exit).
- **Unauthenticated / Concurrent Sockets**: Not applicable. However, concurrent executions must exit quickly (`TerminalStatus::Busy`, exit code 3) without consuming budget or rendering partial XML.

### Lock Safety Rules
- Never disable `SingleFlightLock` in production scripts (`scripts/pe_run.sh`).
- If a stale lock remains (previous process crashed without releasing), the next invocation must either clean it (`rm -f`) or verify it is stale (process no longer exists) before acquiring.
- Verify lock behavior with `tests/concurrency_tests.rs` before committing any change to `cli/src/lock.rs`.

---

## 3. Pipeline Execution & Concurrent Rules (`cli/src/pipeline.rs`)

- `execute_pipeline` must complete all sub-tasks (`context/scanner.rs`, `adapters/writer.rs`, `core/src/render.rs`) within `Config::budgets.total_deadline_ms`.
- If any sub-task exceeds the deadline, the supervisor (`tokio::time::timeout`) aborts the pipeline and returns `TerminalStatus::DeadlineExceeded` (exit code 4).
- Concurrent executions (`tests/concurrency_tests.rs`) verify that multiple instances of `execute_pipeline` cannot run simultaneously; the second instance exits with `TerminalStatus::Busy`.

---

## 4. Async / Sync Bridging (`core/src/render.rs`, `context/src/`)

- `XmlRenderer::render()` is synchronous (`String`-based). It does not use async I/O. It operates on in-memory data (`fields: HashMap<String, String>`) produced by `execute_pipeline`.
- `context/scanner.rs` (`FileScanner`) performs synchronous file I/O (`fs::read_to_string` or equivalent). In an async pipeline, long scanning operations should be wrapped in `tokio::task::spawn_blocking` to avoid blocking the async event loop. Verify this if scanning large repositories (`> 100 files`).
- `adapters/src/writer.rs` (`LlmWriter`) and `adapters/src/summarizer.rs` (`LlmSummarizer`) may invoke external LLM APIs (if configured). These must use async HTTP clients (`reqwest` or equivalent) with their own timeouts, not block the pipeline indefinitely.

---

## 5. Testing Async Pipeline & Concurrent Execution (`tests/`)

```bash
# Run concurrency tests (verify SingleFlightLock behavior)
cargo test -p pe-cli -- concurrency_tests

# Verify pipeline completes within budget
# (manual: modify Config::budgets.total_deadline_ms to a low value, verify exit code 4)
cat input.md | target/debug/pe --no-llm --lock-path /tmp/test_concurrent.lock
```
