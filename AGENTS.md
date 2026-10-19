# Prompt Enhancement Engine (PE)

## Purpose
`prompt_enhancement_engine` is the core CLI and template engine for structured prompt enhancement. It transforms raw developer prompts into standardized XML prompt templates using 15 session archetypes (`templates/`). It serves downstream AI coding agents by enforcing anti-hallucination rules, token budgets, evidence policies, and standardized output formats. The engine is a Rust workspace (`core`, `context`, `adapters`, `cli`) that operates via stdin/stdout with macOS Karabiner-Elements integration (`scripts/`), supports deterministic offline mode (`--no-llm`), and uses a single-flight runtime lock (`SingleFlightLock`) to prevent concurrent executions.

---

## Core Tech Stack

- **Runtime & Language**: Rust 2021 (`edition = "2021"`), Tokio async runtime (`tokio::main`)
- **CLI & Args**: `clap` (`derive` macro) for structured argument parsing
- **Serialization**: `serde` (`Serialize`/`Deserialize`) for config, templates, and output payloads
- **Workspace Crates**: `core` (types, templates, render, parser, secrets), `context` (scanner, collector, evidence, budget, jail), `adapters` (jev, rules, writer, summarizer, envelope), `cli` (pipeline, lock, clarification, main)
- **Template System**: XML-based `TemplateContract` with `FieldContract`, `StaticSections`, and `CollectionProfile` definitions
- **Security**: `SecretScanner` / `SecretMap`, `escape_xml_text` for prompt injection defense, `SingleFlightLock` for concurrency control
- **Testing**: `cargo test --workspace`, golden prompt tests (`tests/golden_prompts_test.rs`), concurrency tests, security tests, shell integration (`tests/test_pe_run.sh`)
- **Code Quality**: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`, `cargo check`
- **Observability**: Structured stderr status (`TerminalStatus`) and stdout XML payload; no external logging framework

### Target Environments & Infrastructure
- **Development Environment**:
  - Host: macOS (current local system)
  - Stack: Native `cargo` build (`target/debug/pe`, `target/release/pe`) and `uv`-less Rust toolchain
  - Integration: `scripts/pe_run.sh` for clipboard capture/paste workflow; `scripts/karabiner/pe.json` for shortcut binding
- **Deployment / Usage**:
  - Binary delivered via `cargo build --release`
  - Config loaded from `pe_core::Config::load_or_default()` or `--config` override
  - Lockfile managed by `SingleFlightLock` (default or `--lock-path` override)

---

## Architecture & Modular Crates

The codebase uses direct root imports (`use pe_core::...`) without `src/` prefixes. `crates/core/src/lib.rs` defines the public API surface; `crates/cli/src/main.rs` is the entry point.

- **`core`**: Domain primitives (`types.rs`), `TemplateContract` (`templates/definitions.rs`, `templates/contract.rs`), `XmlRenderer` (`render.rs`), input parsing (`parser.rs`), config (`config.rs`), secrets (`secrets.rs`), errors (`error.rs`), status codes (`status.rs`).
- **`context`**: Safe repository inspection (`scanner.rs`, `collector.rs`), evidence bundling (`evidence.rs`), token budget tracking (`budget.rs`), sandboxed file access (`jail.rs`).
- **`adapters`**: External integrations — `jev` (behavioral discovery), `rules` (decision engine), `writer` (LLM prompt writer), `summarizer` (LLM summarizer), `envelope` (envelope structures), `traits` (abstraction interfaces).
- **`cli`**: Pipeline execution (`pipeline.rs`), single-flight lock (`lock.rs`), clarification handling (`clarification.rs`), main binary (`main.rs`).

---

## Sub-Agent Documentation Index (`./docs/ai/`)

For specialized deep-dives, conventions, operational workflows, and rules, agents MUST consult the categorized guides in `docs/ai/`:

| Document | Focus & Domain | When to Consult |
| :--- | :--- | :--- |
| **[OUTPUT_FORMATS.md](docs/ai/OUTPUT_FORMATS.md)** | Standardized concise response archetypes for all 15 session kinds (`<report>` tags) | When formulating final responses for any task (code, debug, test, audit, documentation, prompt engineering). |
| **[COMMANDS.md](docs/ai/COMMANDS.md)** | Quick CLI cheat sheet: `cargo build/test/clippy/fmt`, binary execution (`pe --no-llm`), script verification, Karabiner JSON validation | When needing exact commands to build, test, lint, or operate the engine. |
| **[PLAN_STRUCTURE.md](docs/ai/PLAN_STRUCTURE.md)** | Implementation planning rules, required sections, anti-invention rules, `.plans/` convention | Before writing or modifying any production/template/test code. |
| **[CODING_CONVENTION.md](docs/ai/CODING_CONVENTION.md)** | Rust 2021 conventions, XML template contracts, `escape_xml_text`, anti-hallucination, DRY, token budget economy, strict typing | When creating or modifying `.rs`, `.md` templates, `templates/`, or `prompts/`. |
| **[SECURITY_CHECK.md](docs/ai/SECURITY_CHECK.md)** | Secret scanning (`SecretScanner`), anti-injection (`escape_xml_text`), no embedded instructions, lockfile safety, XML escaping verification | When modifying authentication, secrets, prompt injection defenses, or XML rendering logic. |
| **[TESTS_WORKFLOW.md](docs/ai/TESTS_WORKFLOW.md)** | `cargo test` setup, golden prompt verification, concurrency tests, security tests, budget checks (`tests/test_pe_run.sh`) | When authoring, running, or debugging tests; before opening any change. |
| **[PERFORMANCE_GUIDELINES.md](docs/ai/PERFORMANCE_GUIDELINES.md)** | Token budgets (`BudgetTracker`), context scanner limits, async pipeline deadlines (`Duration::from_millis`), prompt size optimization | When modifying scanner limits, budget tracking, pipeline timeouts, or XML rendering performance. |
| **[API_CONTRACTS.md](docs/ai/API_CONTRACTS.md)** | `TemplateContract` definitions, XML wire format (`<task>`, `<scope>`, `<rules>`), dynamic vs hardcoded sections, cross-template compatibility | When changing `TemplateId`, `FieldContract`, schema shapes, or `templates/*.md`. |
| **[DEPLOYMENT_OPS.md](docs/ai/DEPLOYMENT_OPS.md)** | macOS binary build (`cargo build --release`), `scripts/pe_run.sh`, Karabiner integration (`scripts/karabiner/pe.json`), `SingleFlightLock`, config override (`--config`) | When modifying scripts, build processes, deployment workflows, or binary packaging. |
| **[DATABASE_PERSISTENCE.md](docs/ai/DATABASE_PERSISTENCE.md)** | File-based state persistence (`templates/`, `prompts/`, `.plans/`, config files), lockfile persistence, migration/integrity rules for XML templates | When designing persistence of templates, prompts, lockfiles, or template contracts (no relational DB). |
| **[REALTIME_WEBSOCKETS.md](docs/ai/REALTIME_WEBSOCKETS.md)** | Async pipeline concurrency (`execute_pipeline`), `SingleFlightLock`, deadline supervisor (`tokio::time::timeout`), concurrent execution rules | When working with async pipeline, concurrent execution, timeout/deadline behavior, or lock management. |

---

## 📋 Agent Response & Output Standardization

> [!IMPORTANT]
> **Agents MUST keep final responses extremely short and structured.** Avoid conversational filler ("Sure, I updated the model..."), repeating prompts, or reprinting entire files/diffs already visible in tool execution.
> Every response MUST conclude with a standardized archetype from [OUTPUT_FORMATS.md](docs/ai/OUTPUT_FORMATS.md).

### Default Output Archetype (General Tasks & Implementation)
For general coding, refactoring, or feature tasks in this repository, structure your response as:
```markdown
KEEP YOUR RESPONSE EXTREMELY SHORT.
1. 🎯 **Key Changes**: (bullet points, max 1 line each)
2. ⚠️ **Important**: (assumptions, critical decisions)
3. 🚫 **Blockers**: (items actively blocking progress)
4. 📋 **Remaining Tasks**: (what is left to do, actionable items)
```
*For specialized tasks (Debug, Testing, Step Progression, Architecture Plans, Audits, Documentation, Prompt Engineering, DevOps, Git, Config/Tooling, Research, Report Summary), see [OUTPUT_FORMATS.md](docs/ai/OUTPUT_FORMATS.md).*

---

## Cross-Repo Contracts

- **Template Contracts**: XML prompt contracts (`templates/*.md`) define the 15 session archetypes with required (`<task>`, `<acceptance_criteria>`, `<scope>`, `<rules>`, `<report>`) and optional (`<context>`, `<relevant_files>`, `<verify>`) tags. Dynamic fields (`{{task}}`) are validated against `TemplateContract` in `core/src/templates/`.
- **Evidence Policy**: `EvidencePolicy` (`UserTextOnly` vs `EvidenceAllowed`) controls whether fields can incorporate verified repository evidence or must be derived solely from user input (`prompts/master-prompt.md`).
- **Prompt Integration**: `scripts/pe_run.sh` integrates with macOS clipboard and Karabiner; the binary outputs XML payload to stdout and machine-readable status to stderr. No cross-repo networking contracts exist (offline-capable by design).

---

## Code Navigation & Behavioral Discovery

This repository uses `jg` (Jevgrep) for behavioral code discovery:
- **When to use**: For "how/why/where does X work?" questions across unfamiliar `.rs` files or templates, start with `jg "<question>" crates/` before broad `grep` or Git history digging.
- **How to verify**: `jg files crates/` reports eligible file/byte counts. Requires `jg auth`.

This repository also maintains structured template definitions:
- **Where to look**: `templates/README.md`, `crates/core/src/templates/definitions.rs`, `crates/core/src/templates/contract.rs`.
- **How to update**: Modify `definitions.rs` for new `TemplateId` or `FieldContract` entries; sync `templates/*.md` manually; verify with `cargo test --workspace`.
- **Usage**: Consult template contracts first when changing prompt behavior, then verify against `.rs` source.
