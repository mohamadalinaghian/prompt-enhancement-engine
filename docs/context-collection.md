# Context Collection Policy

## 1. Objective

Collect the minimum verified local context that materially improves the prompt. Do not try to understand the repository. The product is a better instruction for a downstream coding agent, not completion of the coding task.

Gathering more context does not always improve the result. Irrelevant context lowers quality, raises cost and increases leakage risk. Collection is therefore **driven by what the selected template's fields need**, not by what is available.

## 2. Project root (LOCKED: D-004, D-018)

- A project root exists only when the first input line is a `~/`-prefixed path that resolves to an accessible directory (see `workflow-integration.md` §7).
- No project root means no project filesystem context (`NO_COLLECTION`).
- Never select another root. Never read outside the root.
- All paths are canonicalized. A read is allowed only if the canonical path is inside the canonical root. A symlink whose target escapes the root is blocked, not followed.
- Containment is re-checked at read time to narrow races between discovery and read.
- v1 does not detect nested projects or monorepo sub-roots. Paths named by the user are resolved relative to the root. Recorded as a known limitation.
- Session/agent history is **not** a v1 source (LOCKED: D-005). The collector is built behind a source interface so a history source can be added later by a recorded decision.

## 3. Field-driven collection

Each template contract (see `template-contracts.md`) lists its variable fields and, per field, an `evidence_policy`:

- `user_text_only` — the field may be derived only from the user's words.
- `evidence_allowed` — the field may use verified project evidence.

Rules:

1. If the selected template has no `evidence_allowed` field, skip collection (`NO_COLLECTION`).
2. Otherwise, each `evidence_allowed` field that is required, or optional and plausibly improvable, yields a **need** (for example "which files are in scope", "which project convention applies").
3. Collection stops when every need is satisfied, explicitly marked unsupported, or a limit applies.

This replaces a vague "is the evidence sufficient?" judgment with a concrete, checkable stopping rule.

## 4. Phases

### A. Parse the request (Rust)

Extract only what is explicitly present: requested outcome, named files, directories, symbols, errors, APIs, tools, explicit constraints, expected behavior. Do not convert an assumption into a fact.

### B. Decide whether context is useful

Only when a root exists and the template has evidence-eligible fields: Jev Noul (`jev-integration.md`), rule fallback (D-010).

### C. Initial targeted collection

Default priority, changed only when the task gives a clear reason:

1. Paths or symbols the user named.
2. Directly related local project instructions that apply to the target area, if they exist and are safe to read.
3. The target file or the smallest relevant file set.
4. Directly related tests, configuration, interfaces or neighbouring examples that answer a specific unresolved need.
5. Further references only when a named need requires them.

Check that a file exists before reading it. Do not assume conventional files exist. Do not read a whole directory because one file in it is relevant.

### D. Evaluate and choose the next step

Rust builds the candidate set (existing, permitted, not yet inspected). Jev may pick one candidate id or "none useful" (`jev-integration.md`). Rust validates the choice against the candidate set, the exclusions and the budgets before any read.

### E. Expand selectively

Expand only when a named need can plausibly be answered by a discovered, permitted candidate. Prefer a specific file over a broad search.

### F. Summarize when necessary

Rust decides, not Jev. If the evidence bundle exceeds the writer's input budget, run the summarizer (`model-adapters.md`). Before summarizing, first try deterministic reduction: select line ranges around the named symbol or the relevant section, and drop low-value items.

### G. Stop

Stop on any of: all needs resolved; the next candidate has low expected value; a hard budget reached; project unavailable; a security rule blocks the operation; a material ambiguity needs the user.

## 5. Candidates and loop safety

- Candidates are created by local code and have stable ids for this run.
- Jev can only select an id from the current candidate set.
- Rust keeps the set of inspected ids and never re-inspects one without a documented reason.
- Every iteration must add evidence, change the status, or terminate. Repeated selection of an already-inspected or unavailable candidate terminates expansion and records the decision failure.
- A hard iteration limit in Rust always applies.

## 6. Budgets

Rust enforces configurable upper bounds for: files inspected, bytes read, estimated model-input tokens, iterations, elapsed time (via the overall deadline), and model calls or cost where measurable.

**OPEN:** values. The implementing agent proposes provisional values, labels them PROVISIONAL, records the rationale and measures on representative repositories before fixing them. Limits are checked before reading, where feasible.

If a limit is reached, report `BUDGET_EXHAUSTED`. Proceed to writing only if the remaining evidence still backs every required field without invented details; otherwise the run ends as `BUDGET_EXHAUSTED_INSUFFICIENT` (or a clarification, if a specific question would resolve it).

## 7. Collection profiles

Profiles describe behavior. Each template lists its allowed profiles in its contract. A profile selection is a constrained enum validated by Rust. Add or change profiles only by recorded decision.

| Profile | Behavior |
|---|---|
| `none` | No project context. |
| `targeted_file` | A named file, symbol or feature is central. |
| `bug_context` | Relevant implementation, any observed error, directly related tests if present. |
| `architecture_context` | Only module boundaries and interfaces needed to frame a design request. |
| `test_context` | Relevant tests and the actual test-tool configuration, if discoverable. |
| `configuration_context` | Relevant configuration, build, CI or deployment files. |
| `repository_state` | **Disabled in v1.** It needs version-control commands, and the safe invocation rules (`security-and-privacy.md` §6) must be decided first. |

## 8. Local tools

**DEFAULT:** v1 uses in-process Rust file APIs for directory listing, file reading and text search. No external executable is run. Adding one requires a recorded decision with: exact invocation as an argument vector (never a shell string), scrubbed environment, repository-configured execution disabled, and a test.

## 9. Exclusions and data minimization

- Default exclusions (configurable, documented, not assumed universal): environment and credential files, private keys, version-control internals, dependency and vendored directories, build output, generated files, binaries, and files over the byte budget.
- A filename exclusion is not sufficient: all collected text is also scanned for secrets (`security-and-privacy.md` §3).
- Prefer a short excerpt or structured fact over full file contents.
- Do not send unrelated repository content to any model.
- Apply size limits before reading or sending where feasible.

## 10. Negative findings

A failure to find something in a bounded search is not proof it does not exist. Record the search scope with every negative finding. Only state absence when the coverage justifies it.

## 11. Staleness and uncommitted state

Record each file's modification time and content hash as provenance. A file being present does not make it current or authoritative. When two sources conflict, keep both (`evidence-bundle.md` §6). v1 does not read version-control state.

## 12. Clarification policy

Ask only when the ambiguity is material to scope, behavior, safety or a key design choice, **and** permitted local evidence cannot resolve it. Do not fabricate a completed prompt that encodes one interpretation as fact. Use the clarification block (`workflow-integration.md` §8).

When the writer and collection both raise clarifications, the writer's single most blocking question wins; only one question is shown.
