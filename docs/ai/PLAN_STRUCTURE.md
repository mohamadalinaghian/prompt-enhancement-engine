# Implementation Planning Rules & Structure

Before writing code, create an execution-ready plan grounded in the actual repository. Resolve decisions during planning; do not leave them for implementation. Store plans in the `.plans/` directory (gitignored by convention, or commit only approved plans explicitly).

---

## Required Plan Sections

1. **Objective and acceptance criteria**
   - Define the observable outcome and numbered, testable criteria.
   - Reference exact `TemplateId`, `FieldContract`, or `.rs` symbols if applicable.

2. **Scope and constraints**
   - State included work (specific crates, files, templates), non-goals (unrelated refactoring), compatibility requirements, and behavior that must remain unchanged.
   - Explicitly list out-of-scope files (e.g., do not modify `scripts/karabiner/` during template contract changes).

3. **Verified repository context**
   - Read relevant `.rs` files (`crates/core/src/templates/*.rs`, `templates/*.md`, `tests/*.rs`) first.
   - Record exact paths, existing symbols (`XmlRenderer::escape_xml_text`, `TemplateContract`), signatures, dependencies, and patterns to reuse.
   - Distinguish verified facts from proposed additions. Never present assumptions as facts.

4. **Behavior and contracts**
   - Specify inputs (`raw_input` string), outputs (`XmlRenderer` output, `TerminalStatus`), types (`TemplateContract` fields), interfaces, data changes, edge cases, and error behavior.
   - Include applicable security (`escape_xml_text`, `SecretScanner`), configuration (`Config` overrides), and template contract (`FieldContract`) requirements.
   - Resolve design choices explicitly (e.g., new `FieldContract` vs reuse of existing `CollectionProfile`).

5. **File-by-file changes**
   - List exact files to create or modify (`crates/core/src/templates/definitions.rs`, `templates/*.md`, `tests/*.rs`, `docs/ai/*.md`).
   - Identify affected symbols (`TemplateId`, `StaticSections`, `CollectionProfile`) and intended changes.
   - Identify new symbols explicitly; do not assume they already exist.

6. **Ordered implementation tasks**
   - Break work into small, dependency-ordered tasks.
   - For each task specify:
     - Goal and acceptance criteria covered.
     - Prerequisites and files to read.
     - Allowed files and exact changes.
     - Contracts and behavior to preserve (`escape_xml_text`, XML tag pairing).
     - Concrete tests with expected results (`cargo test -p pe-cli`).
     - Validation commands and success conditions (`cargo check`, `cargo test --workspace`, `bash -n scripts/`).
     - Conditions requiring the agent to stop (missing verification, broken contracts, new unverified dependencies).

7. **Verification and completion**
   - Specify task-level (`cargo test -p <crate>`), integration (`cargo test --workspace`), and regression checks (`tests/test_pe_run.sh`).
   - Provide verified commands, working directories, and prerequisites (`cargo build` before `tests/test_pe_run.sh`).
   - Require a final report of changed files, checks actually run, results, and unresolved blockers.
   - When proposing a plan, format output according to the **Plan & Architecture** archetype in [OUTPUT_FORMATS.md](docs/ai/OUTPUT_FORMATS.md#5-%F0%9F%8F%9B%EF%B8%8F-plan--architecture-plan_architecture).
   - When executing multi-step plans, report each step using the **Step Progression** archetype in [OUTPUT_FORMATS.md](docs/ai/OUTPUT_FORMATS.md#4--step-progression--plan-execution-session_progression).

---

## Anti-Invention Rules

- Do not invent repository facts (`TemplateContract` definitions, `.rs` file names, `crates/` structures), APIs (`XmlRenderer` methods), dependencies (`Cargo.toml` entries), configuration keys (`Config` fields), or business rules (`EvidencePolicy`).
- Use existing verified interfaces (`TemplateContract`, `XmlRenderer`, `SecretScanner`); introduce new ones only when explicitly specified.
- Do not add dependencies to `Cargo.toml`, alter public contracts (`templates/*.md` wire format), or perform unrelated refactoring without approval.
- Avoid vague instructions such as "handle errors appropriately"; specify exact behavior (`raise HttpError` equivalent: return `Err(CoreError::TemplateContractMismatch(...))`).
- If information is missing or the repository contradicts the plan, report BLOCKED with the discrepancy and required clarification.
- Never weaken tests, suppress failures (`assert!` removal), or bypass checks (`--no-llm` override without verification) to make validation pass.
- Report unexecuted checks as NOT RUN; never claim unverified success.
- Implement one task at a time, inspect its diff (`git diff`), and pass its checks (`cargo test`) before proceeding.
- Before implementation, ensure no unresolved product, architecture, interface, or dependency decisions remain.
