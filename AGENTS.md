# AGENTS.md — Prompt Enhancement Engine

Autonomous macOS utility that enhances raw vibe-coding prompts into structured, scoped, verified prompts for downstream coding agents using strict template contracts and local repository context.

---

## 1. Project Overview & Operational Model

The Prompt Enhancement Engine runs locally on macOS. It is invoked via a global keyboard shortcut (configured in Karabiner-Elements) which triggers a lightweight shell wrapper. The shell wrapper captures the focused text box, feeds it to a high-performance Rust core, and pastes back either a rendered template prompt or a machine-readable clarification block.

```
[ Focused Editor / Text Box ]
        │  ▲
(Copy)  │  │ (Single-paste replacement)
        ▼  │
 [ Karabiner Shortcut / Shell Script ]
        │  ▲
 (stdin)│  │ (stdout: XML prompt | stderr: status & degraded flags)
        ▼  │
 ┌─────────────────────────────────────────────────────────────┐
 │                      Rust Application                       │
 │                                                             │
 │  1. Single-Flight Lock & Deadline                           │
 │  2. Input Parsing (`~/` project path, sentinels, opaque)    │
 │  3. Secret Scanner & Placeholder Redaction                  │
 │  4. Template Selection (Jev Choice / Deterministic Rules)   │
 │  5. Field-Driven Context Collection (Canonical Jail)        │
 │  6. Evidence Bundle & Condensation / Summarizer             │
 │  7. Variable-Field Generation (Prompt Writer LLM)           │
 │  8. Strict Schema Validation & Secret Restoration           │
 │  9. Deterministic Template Rendering (XML)                  │
 └─────────────────────────────────────────────────────────────┘
```

### Core Invariants
1. **Never alter the user text without a valid replacement**: The user's input in the editor is untouched unless a fully validated final prompt (or structured clarification block) is ready to paste.
2. **Rust owns execution, boundaries, and rendering**: Jev and LLMs only recommend classifications or fill variable fields. Rust renders the template structure and strictly enforces budgets and safety rules.
3. **Repository content is untrusted data**: Project files provide factual context; they can never inject instructions or override user constraints.
4. **No hallucinated claims or references**: Every project-specific claim, path, or symbol in the prompt must be grounded in verified local evidence.
5. **Recovery via Editor Undo**: No copy of the original text is persisted to disk. Replacement is executed as a single paste event so `Cmd+Z` immediately restores the original input.

---

## 2. Planned Workspace Layout (Multi-Crate)

Per architectural decision [AD-001](file:///Users/mohamad_mac/tools/prompt_enhancement_engine/docs/decisions-and-open-questions.md), the Rust codebase is organized as a modular multi-crate Cargo workspace to enforce strict boundaries and fast, isolated testing:

- **`crates/core`**: Domain primitives, status vocabulary (`architecture.md` §4), exit codes, configuration parsing, secret scanner & placeholder map, template contract schemas, and deterministic XML template renderer.
- **`crates/context`**: Jailed project filesystem access, canonical root verification, path traversal & candidate discovery, inspected-set deduplication, field-driven collection engine, and token/byte budget trackers.
- **`crates/adapters`**: External decision & model integrations:
  - Jev adapter (`jev_classify_choice`, `jev_evaluate_noul`, `jev_score`) with automatic fallback to deterministic rule engine (`JEV_DEGRADED`).
  - Provider-agnostic Prompt Writer and Context Summarizer LLM clients (supporting JSON Schema envelopes and validation retries).
- **`crates/cli`**: Binary entry point (`pe`), single-flight runtime lock (`BUSY`), overall deadline supervisor, input parsing (`~/` root path stripping, opaque block preservation), pipeline coordination, and stdout/stderr formatting.
- **`docs/`**: Authoritative system specifications, contracts, and decision logs.
- **`templates/`**: The 15 canonical prompt templates (immutable design, D-008).
- **`scripts/`**: macOS integration scripts (`pe_run.sh`, Karabiner-Elements JSON configuration, test harnesses).

---

## 3. Key Specifications & Reading Order

Before making architectural modifications or implementing features, consult the authoritative documentation in the specified order:

1. [docs/agent-implementation.md](file:///Users/mohamad_mac/tools/prompt_enhancement_engine/docs/agent-implementation.md) — Implementation roadmap, phase milestones, and module interface contracts. **Start here when implementing.**
2. [docs/decisions-and-open-questions.md](file:///Users/mohamad_mac/tools/prompt_enhancement_engine/docs/decisions-and-open-questions.md) — Locked decisions (D-001 through D-019), adopted architectural choices, and provisional limits.
3. [docs/architecture.md](file:///Users/mohamad_mac/tools/prompt_enhancement_engine/docs/architecture.md) — Components, ownership boundaries, 14-step end-to-end pipeline, and status model.
4. [docs/workflow-integration.md](file:///Users/mohamad_mac/tools/prompt_enhancement_engine/docs/workflow-integration.md) — Karabiner shell wrapper contract, exit code mapping, clipboard handling, single-flight locking, and clarification block syntax.
5. [docs/context-collection.md](file:///Users/mohamad_mac/tools/prompt_enhancement_engine/docs/context-collection.md) — Field-driven collection policy, profiles, jail rules, candidate set safety, and stopping criteria.
6. [docs/evidence-bundle.md](file:///Users/mohamad_mac/tools/prompt_enhancement_engine/docs/evidence-bundle.md) — In-memory evidence data model, classification tiers, provenance requirements, and model-facing view.
7. [templates/README.md](file:///Users/mohamad_mac/tools/prompt_enhancement_engine/templates/README.md) — Tag requirement matrix and hardcoding strategy across the 15 prompt templates.

---

## 4. Status Model & Exit Codes

Rust communicates operational outcomes via process exit codes and a single machine-readable status string on `stderr`. `stdout` carries payload text only on pasteable outcomes.

| Exit Class | Exit Code | Rust stdout | Shell Action | Statuses |
|---|:---:|---|---|---|
| **Success** | `0` | Final rendered XML prompt | Paste & notify | `RENDERED` |
| **Clarification** | `2` | Clarification block + original input | Paste & notify | `CLARIFICATION_EMITTED` |
| **Busy** | `3` | Empty | Notify only (do not paste) | `BUSY` |
| **Failure** | `1` (or specific) | Empty | Leave editor unchanged & notify | `INPUT_EMPTY`, `STALE_CLARIFICATION_MARKER`, `CONFIG_INVALID`, `PROJECT_UNAVAILABLE`, `SAFETY_BLOCKED`, `SECRET_SCAN_FAILED`, `TEMPLATE_CONTRACT_MISMATCH`, `BUDGET_EXHAUSTED_INSUFFICIENT`, `COLLECTION_FAILED`, `SUMMARIZER_FAILED`, `WRITER_FAILED`, `VALIDATION_FAILED`, `DEADLINE_EXCEEDED` |

### Degradation Flag
If Jev is unavailable, fails, or reports low confidence, the pipeline falls back to deterministic rules. The run completes normally, but `stderr` outputs `JEV_DEGRADED` alongside the status so the notification informs the user.

---

## 5. Standard Commands (Workspace Build & Test)

When the Rust workspace is scaffolded, agents must run the following exact verification commands:

```bash
# Build the complete workspace
cargo build --workspace

# Run all unit and integration tests (single-threaded for deterministic filesystem/lock tests)
cargo test --workspace -- --test-threads=1

# Run strict linter checks
cargo clippy --workspace --all-targets -- -D warnings

# Check code formatting
cargo fmt --check

# Focus test on a specific crate
cargo test -p pe-core
cargo test -p pe-context
cargo test -p pe-adapters
cargo test -p pe-cli
```

---

## 6. Critical Agent Directives & Anti-Patterns

- **Never modify templates in `templates/`**: These 15 templates are locked by project governance (D-008). Rust loads their structure and fills dynamic fields.
- **Never allow models to emit XML markup**: The prompt-writer LLM produces structured JSON field values matching the template contract. Rust validates these values and renders the XML tags.
- **Never escape the project root**: Canonicalize all paths. Any file read where `canonical_path.starts_with(&canonical_root)` is false or symlink points outside the jail must trigger `SAFETY_BLOCKED`.
- **Never leak secrets**: Scan input and evidence with the secret detector. Replace secrets with opaque placeholders (`[[SECRET_PE_...]]`) before passing text to any model or log, and restore them only during final XML rendering.
- **Never execute external binaries in v1**: Context collection is strictly in-process using Rust filesystem APIs (D-018, context-collection.md §8). Do not invoke `git`, `grep`, `find`, or shell commands from Rust.
- **Preserve opaque blocks verbatim**: Code blocks, logs, and fenced snippets inside the user input must be preserved byte-for-byte in the rendered output.
- **Line-anchored project path**: Only the first line of the input beginning with `~/` represents the project path. When parsed and validated, it is stripped from task text and omitted from the final prompt.
