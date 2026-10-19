# Deployment, Binary Operations & Observability Guide

This document details binary build rules, macOS script integration, `SingleFlightLock` management, `Config` overrides, and the deployment workflow for `prompt_enhancement_engine`.

---

## 1. Environments & Infrastructure Overview

### Development Environment
- **Host**: macOS (current local system).
- **Local Build Tooling**: `cargo` native build (`target/debug/pe`, `target/release/pe`). No Docker containers; no external package manager (`uv` not used).
- **Script Integration**: `scripts/pe_run.sh` (clipboard capture/paste), `scripts/karabiner/pe.json` (keyboard shortcut binding).
- **Lockfile**: `SingleFlightLock` uses `/tmp/pe_default.lock` (default) or custom (`--lock-path`). Lockfile persistence must not survive system restarts unless explicitly configured.
- **Config**: `Config::load_or_default()` loads from default settings or `--config` override path.

### Binary Distribution / Usage
- **Binary Path**: `target/release/pe` (production) or `target/debug/pe` (development).
- **No Automated CI/CD**: There is no automated pipeline (`GitHub Actions`, `GitLab CI`). All builds and tests are manual (`cargo build`, `cargo test`).
- **Deployment Process**: Manual binary build (`cargo build --release`) and script verification (`bash -n scripts/pe_run.sh`).

---

## 2. Critical Gotchas & Constraints

### 1. Binary Must Be Built for Local Architecture Only
- The engine is a native `rustc` binary compiled for the host architecture (`aarch64-apple-darwin` on macOS). There is no cross-compilation (`linux/amd64`, `windows`) configured in `Cargo.toml`.
- If deploying to a different architecture, rebuild from source (`cargo build --release`) rather than copying the binary.

### 2. `SingleFlightLock` File Path Must Be Writable
- The default lock file (`/tmp/pe_default.lock`) requires a writable `/tmp` directory. If `/tmp` is read-only or missing, `SingleFlightLock::acquire()` will fail, and the binary will exit with `TerminalStatus::Busy` (exit code 3) or a file-system error.
- Always provide `--lock-path` pointing to a writable directory (`/tmp/`, `~/tmp/`, or project-local `.lock/`) in restricted environments.

### 3. `scripts/pe_run.sh` Argument Preservation
- The script passes clipboard content to `pe` via stdin (`printf "%s" "$COPIED" | "$PE_BIN" ...`).
- Never modify `scripts/pe_run.sh` to redirect stdin incorrectly or lose quoting (`"$COPIED"`). Incorrect quoting destroys input payload integrity.
- The script ends with `exec "$@"` equivalent behavior (implicit via direct execution). Always preserve the binary's exit code for the caller (`exit` mapping to notification sounds: `Glass` for 0, `Basso` for 2 / clarification).

### 4. Clipboard Workflow Integrity (`scripts/pe_run.sh`)
- The script saves previous clipboard (`PREV_CLIPBOARD`) and restores it after the operation.
- If the application focus changes (`INITIAL_APP != CURRENT_APP`), the result is placed on the clipboard (`pbcopy`) but not pasted (`keystroke "v"` is skipped) to prevent pasting into the wrong window.
- Never disable the focus-change guard (`[ "$CURRENT_APP" != "$INITIAL_APP" ]`) without explicit user approval.

---

## 3. Script Integration & MacOS Workflow

### Clipboard Capture Workflow (`scripts/pe_run.sh`)
```bash
# Execute the full workflow (clipboard capture -> enhance -> paste)
bash scripts/pe_run.sh
```

### Binary Execution with Custom Config
```bash
# Run with custom config file
cat input.md | target/release/pe --no-llm --config ./custom_config.toml
```

### Binary Execution with Custom Lock Path
```bash
# Use a project-local lock file to avoid conflicts with other users
cat input.md | target/release/pe --no-llm --lock-path ./.local/pe_custom.lock
```

---

## 4. Config Overrides (`core/src/config.rs`)

- `Config` defines default budgets, timeout durations, and behavior flags.
- Override any config value using `--config <path>` pointing to a valid TOML or JSON file (format defined by `serde` and `Config` structure).
- Never commit custom `.env` or `.config` files containing user-specific paths or secrets to `git`.
- The `Config` must not contain `secret` fields (`SecretMap` handles secrets separately); secret scanning (`SecretScanner`) must run independently of config loading.

---

## 5. Observability & Logging

The engine uses structured stderr (`TerminalStatus::format_stderr()`) and stdout (`XmlRenderer` output) for all communication. There is no external logging framework (`fluent-bit`, `loki`, `grafana` not applicable).

- **Application Output**: All binary executions output XML payload (`stdout`) and status (`stderr`).
- **Error Tracking**: No `Sentry` or external APM integration. Errors (`CoreError`) are mapped to `TerminalStatus` exit codes and formatted to stderr.
- **Log Aggregation**: Not implemented. If needed, redirect stderr to file manually (`>> log.txt 2>&1`).
- **Performance Monitoring**: Measure pipeline execution time manually (`time target/release/pe --no-llm < input.md`) or via `tests/test_pe_run.sh` timing.

---

## 6. Binary Build & Distribution Workflow

```bash
# Build for production (release binary)
cargo build --release

# Build for development (debug binary with symbols)
cargo build

# Verify binary exists and is executable
ls -lh target/release/pe

# Verify binary output and exit-code mapping
bash tests/test_pe_run.sh

# Clean previous builds (if necessary)
cargo clean
```

---

## 7. Script Maintenance (`scripts/karabiner/pe.json`)

- The Karabiner JSON (`scripts/karabiner/pe.json`) defines keyboard shortcuts (`key_code`, `modifiers`) that trigger `scripts/pe_run.sh`.
- Any change to the shortcut binding must be validated by `python3 -m json.tool` and tested manually (press shortcut in a text editor).
- Never modify `key_code` or `modifiers` without confirming the new binding does not conflict with existing macOS shortcuts.
- Registry maintenance (`~/docker-registry/`): Not applicable to this repository.
