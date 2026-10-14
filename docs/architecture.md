# Architecture

## 1. Goal

Turn a user-written prompt into a clearer, scoped, checkable prompt for a downstream coding agent, using only the user's words and verified local evidence. Never destroy or silently reinterpret the user's text.

The tool's value over the downstream agent (which can read the repository itself) is: intent clarification, scope limits, acceptance criteria, project conventions, and verified references. It is not to copy source code into the prompt. **DEFAULT:** evidence is carried as facts and verified pointers; excerpts are short and used only when they carry a constraint.

## 2. Components and ownership

| Component | Owns | Must not |
|---|---|---|
| Karabiner + shell script | Triggering; clipboard save/restore; select-all/copy/paste keystrokes; notifications | Contain prompt logic |
| Rust application | Everything else: input parsing, locking, redaction, collection, budgets, validation, rendering, exit codes | Touch the editor directly |
| Jev adapter | Semantic recommendations as typed answers (see `jev-integration.md`) | Return anything Rust treats as authority |
| Context collector | Bounded, validated reads under the project root | Read outside root, follow escaping symlinks, run repository-configured code |
| Summarizer model | Condensing already-selected, redacted evidence | See secrets, invent provenance |
| Prompt-writer model | Values for variable template fields, or a clarification request | Emit template markup or fixed sections |
| Template renderer | Final structure from the existing template files | Accept structure from a model |

Separation of concerns: **Jev recommends, Rust validates and executes, writer/summarizer models interpret within a contract.**

## 3. End-to-end flow

1. Shell: save the current clipboard; select all in the focused text box; copy; invoke the Rust binary with the text on stdin. (`workflow-integration.md`)
2. Rust: load and validate configuration; take the single-flight lock (else exit `BUSY`); start the overall deadline.
3. Read input. Empty input → stop. A leading clarification block → stop (`STALE_CLARIFICATION_MARKER`).
4. Parse the first line as a project path (`~/` prefix). Invalid or inaccessible → `PROJECT_UNAVAILABLE`, stop. Valid → record the root and remove that line from the task text.
5. Extract opaque verbatim blocks (fenced code/log blocks). Scan the task text for secrets; replace them with placeholders (`security-and-privacy.md`). Scanner error → stop (`SECRET_SCAN_FAILED`).
6. Select the template: Jev Choice (two option orders), rules as fallback. (`jev-integration.md`)
7. Decide whether project context is needed. Without a valid project root, collection is skipped. If the selected template has no evidence-eligible field, collection is skipped. Otherwise Jev Noul, rules as fallback.
8. Collect evidence in bounded iterations, driven by the template's unfilled field needs. (`context-collection.md`)
9. Build the evidence bundle. Redact repository secrets irreversibly. If it exceeds the writer's input budget, summarize.
10. Call the prompt writer with the task, template contract and evidence. Receive an envelope: `OK` with fields, or `NEEDS_CLARIFICATION`. (`model-adapters.md`)
11. Validate: schema, preservation check, path/symbol allowlist, injection flags, placeholder integrity, template delimiter safety. Bounded retry on failure.
12. Render the existing template in Rust. Restore user secrets from placeholders. Final integrity checks.
13. Write the result to stdout; exit with the matching code. The shell pastes only on success or clarification.
14. Shell: restore the previous clipboard; show a notification.

Every step that fails leaves the editor text untouched and exits with a failure code.

## 4. Status model

Rust owns one status vocabulary.

### 4.1 Evidence-phase outcomes

| Status | Meaning |
|---|---|
| `NO_COLLECTION` | No project context was needed or possible. |
| `SUFFICIENT_CONTEXT` | Every needed field is backed, or explicitly optional-and-unsupported. |
| `NEEDS_CLARIFICATION` | A material ambiguity cannot be resolved locally. |
| `BUDGET_EXHAUSTED` | A configured limit stopped collection. Never equals sufficient. |
| `PROJECT_UNAVAILABLE` | Supplied root invalid or inaccessible. |
| `SAFETY_BLOCKED` | A security rule forbade a required operation. |
| `COLLECTION_FAILED` | A local operation failed for another reason. |

### 4.2 Run-level terminal statuses

| Status | Editor effect |
|---|---|
| `RENDERED` | Replaced with the final prompt. |
| `CLARIFICATION_EMITTED` | Replaced with clarification block + original text. |
| `BUSY` | Unchanged. Notification only. |
| `STALE_CLARIFICATION_MARKER` | Unchanged. Notification asks the user to remove the block. |
| `INPUT_EMPTY` | Unchanged. |
| `CONFIG_INVALID` | Unchanged. |
| `PROJECT_UNAVAILABLE` | Unchanged. |
| `SAFETY_BLOCKED` | Unchanged. |
| `SECRET_SCAN_FAILED` | Unchanged. |
| `TEMPLATE_CONTRACT_MISMATCH` | Unchanged. |
| `BUDGET_EXHAUSTED_INSUFFICIENT` | Unchanged, or clarification if the policy says so. |
| `COLLECTION_FAILED` | Unchanged. |
| `SUMMARIZER_FAILED` | Unchanged. |
| `WRITER_FAILED` | Unchanged. |
| `VALIDATION_FAILED` | Unchanged. |
| `DEADLINE_EXCEEDED` | Unchanged. |

### 4.3 Degradation flags (non-terminal)

`JEV_DEGRADED` is set when Jev was unavailable, invalid, or below its confidence threshold and rules were used (D-010). The run continues. A successful result carries this flag into the notification text so the user knows rules decided.

## 5. Rust module boundaries

The implementing agent keeps these as separate modules or equivalent interfaces. Names are not prescribed.

- entry point and exit-code mapping
- configuration loading and validation
- single-flight lock
- input parsing (project path, clarification block, opaque blocks)
- secret detection and placeholder map
- project-root validation and safe file reading
- candidate discovery and the inspected set
- evidence normalization and provenance
- decision provider trait (Jev adapter + rule fallback)
- model adapter traits (writer, summarizer)
- template contract loading, drift check, validation, rendering
- clarification block builder and detector
- observability (metadata-only logging)

## 6. Design principles

- Evidence before specificity.
- Minimal relevant context, driven by what the template's fields need.
- Resource limits enforced in Rust regardless of any model's recommendation.
- Traceability: material claims carry provenance.
- Safe failure: failure never alters the editor.
- Template integrity: the renderer owns structure.
- No invented integration details: unknowns stay explicit.
- Determinism where possible: anything Rust can decide exactly, Rust decides (counting, dates, budgets, path validity).
