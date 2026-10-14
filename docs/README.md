# Prompt Enhancement Engine — Specification

## Purpose

A macOS utility that improves prompts written for AI coding agents ("vibe coding").

You write a prompt in any text box and press a shortcut. A Rust program classifies the task, optionally collects a small amount of verified context from the local project, asks a language model to fill the **variable fields** of one of the 15 existing prompt templates, validates that output, renders the final prompt itself, and pastes it back over your text.

These documents define behavior, contracts and acceptance criteria. They do not implement the application. A coding agent implements the system from them (see `docs/agent-implementation.md`).

## How decisions are tagged

Every decision in this specification carries one tag:

- **LOCKED** — chosen by the project owner. Do not change without a recorded decision.
- **DEFAULT** — a design default proposed by this specification so the agent does not have to guess. The owner may change it; record the change in `docs/decisions-and-open-questions.md`.
- **OPEN** — not decided. Do not guess. Ask the owner or keep it configurable.

Facts about external systems (Jev, macOS, Karabiner) are tagged **VERIFIED (date, source)** only if they were read from a primary source. Everything else about them is unverified and must be checked by the implementing agent against current primary documentation.

## Locked decisions (summary)

| ID | Decision |
|---|---|
| D-001 | Platform: macOS. |
| D-002 | Karabiner-Elements invokes a shell script from a keyboard shortcut. |
| D-003 | The main application is written in Rust. |
| D-004 | The first line of the input may be a tilde-prefixed project path, e.g. `~/path/to/project`. It is **dropped** from the final output. |
| D-005 | v1 context comes from the project filesystem only. Session/agent history is a later phase. |
| D-006 | A separate summarizer model condenses evidence that is too large for the prompt-writer. |
| D-007 | When a material ambiguity cannot be resolved locally, a marked clarification block is placed at the top of the text. The user deletes it manually before re-running. |
| D-008 | The 15 prompt templates already exist and must not be redesigned. |
| D-009 | The language model fills variable fields only. Rust renders the template. |
| D-010 | Jev makes the semantic decisions. It selects the template; deterministic rules are the fallback. If Jev is unavailable or invalid, rules are used and the run continues. |
| D-011 | This design work defines tests, not test code. |
| D-012 | Documents are in English. |
| D-013 | Recovery of the original text relies on the editor's undo. No copy is stored. |
| D-014 | A second shortcut press during a run is rejected with a notification. |
| D-015 | No bypass or cancel shortcut in v1. |
| D-016 | Secrets are replaced by placeholders before any model call and restored in the output. |
| D-017 | Model providers are configured generically. No default provider. |
| D-018 | Paths are canonicalized; reads must stay inside the project root; escaping symlinks are blocked. |
| D-019 | Each template's variable-field contract is a hand-written copy kept in this project. |

## Documents

| File | Contents |
|---|---|
| `docs/architecture.md` | Components, ownership, end-to-end flow, status model. |
| `docs/workflow-integration.md` | Shell script contract, input parsing, clipboard, replacement, concurrency, clarification block. |
| `docs/context-collection.md` | Field-driven collection, profiles, budgets, stopping. |
| `docs/evidence-bundle.md` | Evidence data model, classification, provenance. |
| `docs/jev-integration.md` | Verified Jev API facts, decision catalog, confidence handling, fallback. |
| `docs/template-contracts.md` | Contract file format, README facts about the 15 templates, validation and rendering. |
| `docs/model-adapters.md` | Provider-agnostic writer/summarizer adapters, response envelope, retries. |
| `docs/security-and-privacy.md` | Threats, secrets round trip, prompt injection, filesystem, failure behavior, logging. |
| `docs/testing-spec.md` | Required scenarios and measurable acceptance criteria. No test code. |
| `docs/decisions-and-open-questions.md` | Decision log and unresolved items. |
| `docs/agent-implementation.md` | Instructions for the implementing coding agent. **Start here when implementing.** |
| `prompts/master-prompt.md` | Instruction for the prompt-writing model. |
| `prompts/context-summarizer-prompt.md` | Instruction for the summarizer model. |

## Reading order for the implementing agent

1. `docs/agent-implementation.md`
2. `docs/decisions-and-open-questions.md`
3. `docs/architecture.md`, `docs/workflow-integration.md`, `docs/security-and-privacy.md`
4. The remaining documents as each module is built.

## Non-goals

- Redesigning the existing templates.
- Letting a language model produce final template markup.
- Making code changes in the target project.
- Scanning the whole repository by default.
- Treating generated summaries as verified facts.
- Silently guessing project roots, paths, APIs, versions or requirements.
- Session-history context in v1.
- A bypass, cancel, preview or confirmation UI in v1.
- Automatic detection of nested projects or monorepo sub-roots in v1.

## Invariants

1. The user's text is never altered unless a fully validated final prompt (or a clarification block) is ready to paste.
2. Rust owns validation, budgets, filesystem access, secrets handling and rendering.
3. Models recommend or write; they never trigger side effects.
4. Repository content is untrusted data. It can never instruct the system, and it can never become an instruction in the rendered prompt.
5. No project-specific claim appears in the output without supporting evidence or the user's own words.
