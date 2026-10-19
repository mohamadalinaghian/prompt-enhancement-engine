# Agent Output Formats & Reporting Standards

This document defines standardized, token-efficient response archetypes for AI agents working in `prompt_enhancement_engine`. All formats are derived from the 15 `TemplateId` session archetypes (`templates/*.md`) and their hardcoded `<report>` sections (`crates/core/src/templates/definitions.rs`).

---

## ⚡ Universal Output Rules

1. **Keep Responses Extremely Short**: Never repeat full file contents, diffs already viewable in tool calls, or conversational filler ("Sure, I have updated the model..."). Start immediately with the structured report.
2. **Select the Right Archetype**: Choose the template matching the primary session kind. If a task spans multiple types, use the most specific one (e.g., `debug_bugfix` over `general_task`).
3. **Strict Numbered Sections**: Retain the exact numbered section titles and emojis specified below for each archetype.
4. **1-Line Bullet Points**: Keep each point under a section to a single, factual line. No multi-line explanations unless explicitly requested.

---

## 📦 Task Archetypes Catalog (Mapped to `templates/`)

### 1. 🎯 Implementation & Refactoring (`implementation_refactoring`)
*Use when creating new crates, modifying `XmlRenderer`, updating `TemplateContract`, or refactoring `.rs` files.*

```markdown
KEEP YOUR RESPONSE EXTREMELY SHORT.
1. 🎯 **Key Changes**: (bullet points, max 1 line each)
2. ⚠️ **Important**: (assumptions, critical decisions)
3. 🚫 **Blockers**: (items actively blocking progress)
4. 📋 **Remaining Tasks**: (what is left to do, actionable items)
```

---

### 2. 🔍 Debug & Bugfix (`debug_bugfix`)
*Use when investigating compile errors (`cargo check` failures), runtime panics in `execute_pipeline`, XML rendering errors (`escape_xml_text`), or failing golden prompt tests.*

```markdown
KEEP YOUR RESPONSE EXTREMELY SHORT.
1. 🔍 **Root Cause**: (exact file:line and failure mechanism)
2. 🎯 **Fix Applied**: (specific files and logic changed)
3. 🧪 **Verification Result**: (test command and pass confirmation)
```

---

### 3. 🧪 Testing & Verification (`testing_verification`)
*Use when adding `tests/` cases, updating golden prompt assertions, or verifying budget/scanner behavior.*

```markdown
KEEP YOUR RESPONSE EXTREMELY SHORT.
1. 🧪 **Tests Added / Updated**: (test functions and scenarios)
2. ✅ **Test Execution Results**: (pass count, timing, command used)
3. 🛡️ **Edge Cases Covered**: (boundary and failure cases tested)
```

---

### 4. 🪜 Step Progression / Plan Execution (`session_progression`)
*Use when executing an approved multi-step plan (`.plans/`) step-by-step.*

```markdown
KEEP YOUR RESPONSE EXTREMELY SHORT.
1. 🪜 **Step Completed**: (step number and title)
2. 🎯 **Key Changes Made**: (files and logic changes)
3. ✅ **Exit Check Status**: (pass/fail against plan criteria)
4. ⏩ **Ready for Next Step**: (next actionable step awaiting user cue)
```

---

### 5. 🏛️ Plan & Architecture (`plan_architecture`)
*Use when proposing new `TemplateId` definitions, restructuring `crates/`, or designing pipeline changes.*

```markdown
KEEP YOUR RESPONSE EXTREMELY SHORT.
1. 🏛️ **Architecture / Plan Overview**: (1-2 sentences)
2. 🪜 **Step-by-Step Breakdown**: (atomic phases with verification commands)
3. ⚠️ **Risks & Open Questions**: (decisions needing user confirmation)
```

---

### 6. 🛡️ Codebase Audit & Security (`codebase_audit`)
*Use when reviewing `secret.rs`, `escape_xml_text`, XML injection vulnerabilities, or template contract integrity.*

```markdown
KEEP YOUR RESPONSE EXTREMELY SHORT.
1. 🛡️ **Audit Findings**: (Severity, File:Line, Description)
2. 💥 **Impact & Exploitability**: (realistic risk assessment)
3. 🔧 **Remediation Steps**: (actionable recommendations for each finding)
```

---

### 7. ⚙️ Configuration & Tooling (`config_tooling`)
*Use when updating `Cargo.toml`, `pyproject.toml` (if present), `Config` defaults, `clippy` settings, or `.gitignore`.*

```markdown
KEEP YOUR RESPONSE EXTREMELY SHORT.
1. ⚙️ **Configured Tools**: (tool name, version, status)
2. 📂 **Files Modified**: (configuration file paths)
3. 🧪 **Verification Output**: (output from CLI test/lint command)
```

---

### 8. 🚀 DevOps & Deployment (`devops_deployment`)
*Use when modifying `scripts/pe_run.sh`, `scripts/karabiner/pe.json`, build scripts, or binary distribution.*

```markdown
KEEP YOUR RESPONSE EXTREMELY SHORT.
1. 🚀 **Infrastructure Changes**: (scripts, configs, binary updates)
2. 🔒 **Security & Secrets Check**: (confirmation no secrets leaked)
3. 🔄 **Verification & Rollback**: (build/test output and rollback procedure)
```

---

### 9. 📦 Git & VCS Operations (`git_vcs`)
*Use when managing branches, commits, rebases, or template contract versioning.*

```markdown
KEEP YOUR RESPONSE EXTREMELY SHORT.
1. 📦 **Git Operation**: (branch name, commit hash, or PR status)
2. 📝 **Commit Message / Summary**: (title and bullet points)
3. 🔍 **Cleanliness Check**: (verification that no dirty/untracked files remain)
```

---

### 10. 📚 Documentation (`documentation`)
*Use when updating `AGENTS.md`, `templates/*.md`, `docs/ai/*`, or `README.md` files.*

```markdown
KEEP YOUR RESPONSE EXTREMELY SHORT.
1. 📚 **Documentation Updated**: (files and sections modified)
2. 🔗 **Verified References**: (code paths and commands verified)
3. 💡 **Maintenance Notes**: (guidelines for future doc sync)
```

---

### 11. 💡 Question & Consultation (`question_consultation`)
*Use when answering questions about `TemplateContract`, `escape_xml_text`, budget tracking, or architecture.*

```markdown
KEEP YOUR RESPONSE EXTREMELY SHORT.
1. 💡 **Direct Answer**: (core conclusion or answer)
2. 📌 **Code References**: (file paths and line numbers)
3. ⚖️ **Key Considerations / Trade-offs**: (if applicable)
```

---

### 12. ⚖️ Research & Feasibility (`research_feasibility`)
*Use when evaluating new Rust crates, LLM libraries (`LlmWriter`, `LlmSummarizer`), or XML parsing alternatives.*

```markdown
KEEP YOUR RESPONSE EXTREMELY SHORT.
1. ⚖️ **Evaluation Summary**: (options evaluated, pros & cons)
2. 🎯 **Recommendation**: (chosen option with technical justification)
3. ⚠️ **Compatibility & Risks**: (potential blockers, license traps)
```

---

### 13. 📊 Status Report & Summary (`report_summary`)
*Use when summarizing milestone progress or preparing an end-of-session briefing.*

```markdown
KEEP YOUR RESPONSE EXTREMELY SHORT.
1. 📊 **Progress & Current Status**: (factual completed deliverables)
2. ⚠️ **Blockers & High-Priority Risks**: (active impediments)
3. 📋 **Next Action Items**: (prioritized next steps)
```

---

### 14. 📝 Prompt Engineering (`prompt_engineering`)
*Use when designing or revising `prompts/*.md`, `templates/*.md`, or XML tag structures.*

```markdown
KEEP YOUR RESPONSE EXTREMELY SHORT.
1. 📐 **Prompt Architecture**: (tags included and rationale)
2. 🎯 **Token Budget & Model Target**: (estimated token count or budget impact)
3. 🛡️ **Guardrails Enforced**: (anti-hallucination and security boundaries)
```

---

### 15. 🏗️ General Instruction (`general_task`)
*Use for custom workflows outside the standard 14 categories.*

```markdown
KEEP YOUR RESPONSE EXTREMELY SHORT.
1. 🎯 **Key Changes**: (bullet points, max 1 line each)
2. ⚠️ **Important**: (assumptions, critical decisions)
3. 🚫 **Blockers**: (items actively blocking progress)
4. 📋 **Remaining Tasks**: (what is left to do, actionable items)
```
