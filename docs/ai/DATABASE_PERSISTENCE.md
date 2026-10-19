# File-Based State & Persistence Architecture Guide

This repository does not use a relational database (`PostgreSQL`, `MySQL`, etc.). Instead, persistence is file-based: XML templates (`templates/*.md`), prompt contracts (`prompts/*.md`), source code (`crates/*.rs`), lockfiles (`/tmp/pe_default.lock`), and binary artifacts (`target/release/pe`). This document defines persistence rules, migration/integrity conventions, and state safety for file-based systems.

---

## 1. File-Based Persistence & Template Integrity

- **Template Files (`templates/*.md`)**: These are the primary persistent contracts. They must match `definitions.rs` (`TemplateId`, `FieldContract`, `StaticSections`) exactly.
- **Source Code (`crates/*.rs`)**: Source files are persistent through `git`. Changes must pass `cargo check`, `cargo test`, and `cargo fmt -- --check` before committing.
- **Binary Artifacts (`target/release/pe`)**: Not version-controlled (`.gitignore` excludes `target/`). Rebuild after any `.rs` or `Cargo.toml` change.
- **Config & Lockfiles (`.plans/`, `.local/*.lock`)**: Persistent but must be cleaned up after use (`trap 'rm -rf ...' EXIT`). Lockfiles (`SingleFlightLock`) must not accumulate stale files.

---

## 2. Template Contract Migration & Integrity Rules

Because there is no relational database, "migrations" are manual updates to `templates/*.md` synchronized with `definitions.rs`.

### Migration Safety Rules (File-Based Equivalent)
1. **Zero-Downtime Template Updates**: When adding a new `TemplateId` or `FieldContract`, create the `.md` file and `.rs` definition simultaneously. Never delete an existing `.md` file without removing its `TemplateId` from `types.rs` and `definitions.rs`.
2. **Contract Verification Before Commit**:
   ```bash
   # Verify file count matches TemplateId count
   ls templates/*.md | wc -l  # Must equal 15 (TemplateId::ALL.len())

   # Verify definitions match templates
   grep -o 'name: "[^"]*"' crates/core/src/templates/definitions.rs | sort
   ```
3. **File Integrity Checks**: Before opening any PR, verify:
   - `templates/*.md` files contain no broken XML tags (`python3 -c 'import xml.etree.ElementTree as ET; ET.parse(open("test_output.xml"))'`).
   - `definitions.rs` contains no missing `FieldContract` references.
   - All `tests/` pass (`cargo test --workspace -- --test-threads=1`).

---

## 3. Lockfile Persistence (`cli/src/lock.rs`)

- `SingleFlightLock` creates a persistent file (`/tmp/pe_default.lock` or custom `--lock-path`).
- The lock must be released (`drop`) promptly when the binary exits (successful or failure path).
- Stale lockfiles (`file exists but process is dead`) must be handled safely. Verify lock acquisition logic (`SingleFlightLock::acquire`) checks process existence or uses file locking (`flock`) rather than simple file existence.
- Lockfile cleanup: After any crash or unexpected termination, the lockfile may remain. Verify that the next invocation can still acquire the lock (either by cleaning stale locks or using `O_CREATE` with safe overwrite logic).

---

## 4. Template & Prompt Persistence (`prompts/*.md`)

- `prompts/master-prompt.md` and `prompts/context-summarizer-prompt.md` define output schemas (`JSON`, strict fields). These are persistent contracts.
- Changes to prompt contracts (`master-prompt.md`) must be verified against actual `XmlRenderer` output and `adapters/src/summarizer.rs` behavior.
- Never commit unverified prompt templates (`fake_template.md`). Verify file paths exist (`ls templates/*.md`) before referencing them in documentation or contracts.

---

## 5. Verification Before Opening a PR

Before submitting any persistence-related change:

```bash
# Verify all persistent files exist and are valid
find templates/ -name '*.md' | sort
find crates/core/src/ -name '*.rs' | sort

# Verify no stale lockfiles remain in default path
ls /tmp/pe_default.lock 2>/dev/null || echo "No stale default lock"

# Verify binary builds cleanly from persistent source
cargo check --workspace
cargo build --release
```
