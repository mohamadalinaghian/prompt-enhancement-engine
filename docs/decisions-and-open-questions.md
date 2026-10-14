# Decisions and Open Questions Log

This document records all architectural choices, owner decisions, provisional defaults, and unresolved questions for the Prompt Enhancement Engine.

---

## 1. Decision Tags & Governance

Every entry in this registry is classified under one tag:

- **LOCKED**: Chosen authoritatively by the project owner. Must **not** be changed without explicit owner approval.
- **ADOPTED**: An architectural decision formally agreed upon during project planning.
- **DEFAULT / PROVISIONAL**: A working default proposed so implementation can proceed deterministically. Marked `PROVISIONAL` until verified on representative workloads.
- **OPEN**: Unresolved questions requiring owner input or empirical benchmarking. Agents must not guess.
- **VERIFIED (date, source)**: Verified facts regarding external systems (macOS APIs, Jev, Karabiner-Elements).

---

## 2. Locked Decisions Log (D-001 through D-019)

| ID | Decision | Category | Rationale & Implications |
|---|---|---|---|
| **D-001** | Platform: macOS only. | Platform | Tailored to macOS desktop workflows; relies on macOS clipboard (`pbpaste`/`pbcopy`) and Accessibility keystrokes. |
| **D-002** | Karabiner-Elements invokes a shell script from a keyboard shortcut. | Integration | Decouples keystroke listening from the Rust binary. Shell script handles macOS event dispatch. |
| **D-003** | The main application is written in Rust. | Implementation | Ensures deterministic memory safety, zero-cost abstractions, rapid cold-start times (<10ms), and predictable execution. |
| **D-004** | The first line of the input may be a tilde-prefixed project path (`~/...`). It is **dropped** from the final output. | Parsing | Enables ad-hoc context jailing without complex project discovery or UI picker. Must not be re-emitted in the final prompt. |
| **D-005** | v1 context comes from the project filesystem only. Session/agent history is deferred to a later phase. | Scope | Minimizes stateful complexity. Context collector sits behind a trait interface so history sources can be added later. |
| **D-006** | A separate summarizer model condenses evidence that is too large for the prompt-writer. | Architecture | Prevents token window exhaustion and keeps prompt-writer payload tightly focused. |
| **D-007** | When material ambiguity cannot be resolved locally, a marked clarification block is prepended. The user deletes it manually before re-running. | UX / Safety | Prevents hallucinated assumptions; forces user alignment on ambiguous constraints. |
| **D-008** | The 15 prompt templates already exist and must not be redesigned. | Templates | Templates in `templates/*.md` are authoritative standards. Agents fill dynamic tags; static tags remain fixed. |
| **D-009** | Language models fill variable fields only. Rust renders the template. | Separation | Guarantees well-formed XML syntax, prevents prompt injection, and protects XML structure from LLM deviations. |
| **D-010** | Jev makes semantic decisions; deterministic rules are fallback. If Jev is unavailable/invalid, rules are used and run continues (`JEV_DEGRADED`). | Decisions | Jev provides sub-100ms classification. If offline, the tool remains 100% functional via rule fallback. |
| **D-011** | This design work defines tests, not test code. | Specification | Specifications establish acceptance criteria, test scenarios, and error boundaries. |
| **D-012** | Documentation is in English. | Documentation | Standardizes documentation across the codebase. |
| **D-013** | Recovery of original text relies on editor undo. No copy is stored. | Recovery | Zero disk persistence of raw user prompts; single-paste replacement ensures one `Cmd+Z` restores original text. |
| **D-014** | A second shortcut press during a run is rejected with a notification (`BUSY`). | Concurrency | Single-flight execution prevents overlapping clipboard collisions or races. |
| **D-015** | No bypass or cancel shortcut in v1. | Concurrency | Simplifies signal handling. Stuck executions terminate exclusively via the hard deadline. |
| **D-016** | Secrets are replaced by placeholders before any model call and restored in the output. | Privacy | Eliminates credential leakage to external LLM APIs. |
| **D-017** | Model providers are configured generically. No hardcoded default provider. | Portability | Supports OpenAI-compatible, Anthropic, or local endpoints via configuration. |
| **D-018** | Paths are canonicalized; reads must stay inside project root; escaping symlinks are blocked. | Security | Prevents arbitrary filesystem traversal and directory escape attacks (`SAFETY_BLOCKED`). |
| **D-019** | Each template's variable-field contract is a hand-written copy kept in this project. | Contracts | Decouples runtime schema validation from loose markdown parsing. |

---

## 3. Adopted Architectural Decisions

### AD-001: Multi-Crate Workspace Layout
- **Decision**: Partition the codebase into four dedicated crates: `crates/core`, `crates/context`, `crates/adapters`, and `crates/cli`.
- **Rationale**: Enforces strict compile-time boundaries between domain models, jailed filesystem operations, external adapters, and CLI orchestration. Facilitates focused testing (`cargo test -p pe-context`).
- **Owner Approval**: Confirmed during project initiation (2026-10-14).

### AD-002: Field-Driven Context Collection
- **Decision**: Context collection is driven strictly by the variable fields of the selected template (`context-collection.md` §3). Fields tagged `user_text_only` prohibit collection; fields tagged `evidence_allowed` define explicit collection targets.
- **Rationale**: Eliminates broad, unfocused repository scans; minimizes token consumption and latency.

### AD-003: Pure In-Process Filesystem Operations for v1
- **Decision**: No subprocesses (`git`, `grep`, `find`) are spawned by Rust during v1 collection. All directory traversal and text matching use standard Rust libraries (`std::fs`, `walkdir`, `regex`).
- **Rationale**: Prevents command injection vulnerabilities, environment variable contamination, and untrusted repository hook execution.

### AD-004: Line-Anchored Clarification Block
- **Decision**: Clarification blocks use strict versioned sentinels (`[[PE-CLARIFY-v1:BEGIN]]` ... `[[PE-CLARIFY-v1:END]]`) anchored to the start of the input.
- **Rationale**: Allows deterministic, unambiguous detection. If a user re-runs with the marker still present, the engine immediately halts with `STALE_CLARIFICATION_MARKER` without modifying the editor.

---

## 4. Provisional Values & Working Defaults

These values are established as reasonable defaults and marked **PROVISIONAL**. Implementing agents should benchmark them against real-world repositories before hardcoding.

| ID | Parameter | Provisional Value | Rationale | Status |
|---|---|:---:|---|:---:|
| **P-001** | Total Pipeline Deadline | `10,000 ms` | Balances editor responsiveness with LLM completion latency. | PROVISIONAL |
| **P-002** | Writer Per-Call Timeout | `6,000 ms` | Leaves 4,000 ms for Jev, context collection, and validation. | PROVISIONAL |
| **P-003** | Max Inspected Files | `10 files` | Prevents runaway disk scanning on broad queries. | PROVISIONAL |
| **P-004** | Max Total Bytes Read | `128 KB` | Limits in-memory buffering and secret scanning overhead. | PROVISIONAL |
| **P-005** | Max Payload Tokens | `2,500 tokens` | Caps prompt-writer input payload. Oversized bundles trigger summarizer. | PROVISIONAL |
| **P-006** | Max Collection Iterations | `3 iterations` | Bounds candidate expansion loops. | PROVISIONAL |
| **P-007** | Max Input Prompt Size | `64 KB` | Rejects oversized inputs before allocating memory. | PROVISIONAL |
| **P-008** | Lockfile Location | `~/.local/state/prompt_enhancer/pe.lock` | Standard XDG macOS runtime state path. | PROVISIONAL |
| **P-009** | Clipboard Restore Delay | `100 ms` | Short delay to ensure target application finishes processing paste event. | PROVISIONAL |

---

## 5. Open Questions (Unresolved Items)

| ID | Question | Impacted Component | Proposed Options / Notes |
|---|---|---|---|
| **OQ-001** | Should the first-line project path parser support absolute paths (`/...`) in addition to `~/...`? | `pe-core::parser` | **Option A**: Strictly `~/` only (v1 specification).<br/>**Option B**: Allow `/Users/...` if canonicalized and owned by current user.<br/>*Recommendation*: Keep `~/` only in v1 per D-004. |
| **OQ-002** | Should clipboard preservation restore rich clipboard content (RTF, images) or plain text only? | `scripts/pe_run.sh` | macOS `pbpaste`/`pbcopy` handles plain text easily. Preserving multi-MIME pasteboard requires Swift/Objective-C helper or `osascript`.<br/>*Recommendation*: Plain text for v1; log warning if rich data was present. |
| **OQ-003** | How should LLM credentials be supplied? | `pe-adapters` | **Option A**: Environment variables (`OPENAI_API_KEY`, etc.).<br/>**Option B**: Configuration file (`~/.config/prompt_enhancer/config.toml`).<br/>*Recommendation*: Prioritize env vars, fall back to config file. |
| **OQ-004** | How should notifications be presented on macOS? | `scripts/pe_run.sh` | **Option A**: `osascript -e 'display notification ...'` (native macOS banner).<br/>**Option B**: Terminal bell / sound effect.<br/>*Recommendation*: Native macOS notifications via `osascript`. |
| **OQ-005** | If collection exhausts budget without sufficient context, should it emit a clarification block or halt with `BUDGET_EXHAUSTED_INSUFFICIENT`? | `pe-cli` orchestrator | *Recommendation*: If a specific question can unblock the task, emit clarification block; otherwise halt with failure and leave editor untouched. |

---

## 6. Verified External Facts

- **Jev MCP Decision Engine** (VERIFIED: 2026-10-14, `/Users/mohamad_mac/.gemini/antigravity/mcp/jev/instructions.md`):
  - Fast non-autoregressive decision model by TypeSafe AI with ~100ms typical latency.
  - `jev_evaluate_noul`: Evaluates binary probabilities (`0.0` to `1.0`) for safety checks and gatekeeping (used to decide context necessity).
  - `jev_classify_choice`: Fast categorical classifier for intent routing (used to select template from the 15 available options).
  - `jev_score`: Continuous scalar scoring for ranking diffs/logs.
