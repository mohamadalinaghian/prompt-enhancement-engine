# API Contracts & Cross-Template Integration Guide

This document defines the XML prompt contract structure, wire format conventions, `TemplateContract` definitions, and cross-template compatibility rules for `prompt_enhancement_engine`.

---

## 1. Template Contract Architecture (`crates/core/src/templates/`)

The template contract system is powered by structured definitions (`definitions.rs`) and contracts (`contract.rs`) that map to the 15 session archetypes (`templates/*.md`).

### Contract Components (`contract.rs`)
- `TemplateContract`: Defines the template identity (`template_id: TemplateId`), collection profiles (`CollectionProfile`), field contracts (`Vec<FieldContract>`), and static sections (`StaticSections`).
- `FieldContract`: Defines a single dynamic field (`name`, `required: bool`, `evidence_policy: EvidencePolicy`, `description`).
- `StaticSections`: Hardcoded sections (`rules`, `out_of_scope`, `verify`, `report`) that must not be altered by user input without `escape_xml_text`.
- `CollectionProfile`: Defines evidence gathering behavior (`TargetedFile`, `BugContext`, `None`, etc.).

---

## 2. XML Schema Export & Cross-Template Sync

Unlike a traditional OpenAPI JSON contract, this engine uses XML wire contracts embedded in `templates/*.md` and validated by `XmlRenderer`. To ensure type safety across templates:

### Contract Verification Commands
```bash
# Verify all TemplateId values match file names in templates/
python3 -c 'import os; print(len([f for f in os.listdir("templates/") if f.endswith(".md")]))'
# Should output 15

# Verify definitions match template files (manual review)
grep -o 'TemplateId::[A-Za-z]*' crates/core/src/types.rs | sort
```

### Consumption in Downstream Agents
- Downstream AI coding agents (`prompt_writer`, `context_summarizer`) read the XML payload produced by `XmlRenderer` (stdout) and the status (`TerminalStatus`) from stderr.
- Whenever a `TemplateContract` changes (`new FieldContract`, `modified StaticSections`), the corresponding `templates/*.md` file must be updated manually.
- The manifest of templates is maintained in `templates/README.md`. Individual template files (`implementation_refactoring.md`, etc.) are tracked in git.

---

## 3. Wire Format Conventions

### Schema Naming & Tag Structure
- All XML tags are fixed (`<task>`, `<acceptance_criteria>`, `<context>`, `<scope>`, `<relevant_files>`, `<out_of_scope>`, `<rules>`, `<verify>`, `<report>`).
- `scope` contains `<relevant_files>` (optional, user-provided) and `<out_of_scope>` (hardcoded from `StaticSections`).
- Dynamic content (`task`, `acceptance_criteria`, `relevant_files`, `context`) must be validated (`required_fields()`) and escaped (`escape_xml_text`).
- Hardcoded content (`rules`, `out_of_scope`, `verify`, `report`) is inserted directly from `StaticSections` without escaping, but must not contain unverified user input.

### Standard Wire Shape (`XmlRenderer` output)
```xml
<task session="implementation_refactoring">
Specific feature or refactoring objective...
</task>

<acceptance_criteria>
- Observable, testable criteria...
</acceptance_criteria>

<scope>
<relevant_files>path/to/file.rs</relevant_files>
<out_of_scope>Unrelated modules, breaking public API changes without prior approval, rewriting passing tests</out_of_scope>
</scope>

<rules>
- Be EXTREMELY concise. ...
</rules>

<verify>
- Run cargo check...
</verify>

<report>
KEEP YOUR RESPONSE EXTREMELY SHORT.
1. 🎯 **Key Changes**: ...
2. ⚠️ **Important**: ...
</report>
```

### Dynamic vs Hardcoded Sections
- **Dynamic** (`{{task}}`, `{{acceptance_criteria}}`, `{{relevant_files}}`, `{{context}}`, `{{target_test_command}}`): Derived from user input (`stdin`) or verified evidence (`context/scanner.rs`). Must pass `EvidencePolicy` checks.
- **Hardcoded** (`rules`, `out_of_scope`, `verify`, `report`): Defined in `definitions.rs` (`StaticSections`). Must not contain user-controlled data unless explicitly allowed (`EvidencePolicy::EvidenceAllowed` for `relevant_files` only).

---

## 4. Cross-Repo Networking & Environment Contracts

This repository is designed for offline-capable operation (`--no-llm`). There is no external networking contract (no `CORS`, no API base URL, no WebSocket endpoints). However, integration contracts exist for external tools:

- **Karabiner-Elements (`scripts/karabiner/pe.json`)**: Defines keyboard shortcut bindings for triggering `scripts/pe_run.sh`. The JSON contract is validated by `python3 -m json.tool`.
- **Clipboard Integration (`scripts/pe_run.sh`)**: Defines stdin/stdout contract between the OS clipboard and the `pe` binary. Input (`COPIED`) is passed via stdin; output (`STDOUT_PAYLOAD`) is placed back on clipboard (`pbcopy`).
- **LLM Integration (`adapters/src/jev.rs`, `adapters/src/writer.rs`, `adapters/src/summarizer.rs`)**: Defines interfaces (`PromptWriter`, `ContextSummarizer`, `DecisionProvider`) for external LLM clients. These contracts are abstract (`traits.rs`) and must be implemented by concrete adapters, not by `core` modules.

---

## 5. Template Contract Integrity Rules

Before committing any contract change:

1. Verify `TemplateId` exists in `types.rs` and `definitions.rs`.
2. Verify `FieldContract` names match dynamic placeholders in `templates/*.md`.
3. Verify `CollectionProfile` values match evidence gathering behavior (`TargetedFile` for file-based, `BugContext` for debug, `None` for no scanning).
4. Verify `StaticSections` (`rules`, `out_of_scope`, `verify`, `report`) are hardcoded correctly and contain no unverified content.
5. Verify `templates/*.md` file count (`ls templates/*.md | wc -l`) equals 15 (`TemplateId::ALL.len()`).
6. Verify XML output produces valid tags (manual review or `python3 -c 'import xml.etree.ElementTree as ET; ET.parse("test_output.xml")'`).
