# Evidence Bundle Contract

## 1. Purpose

The evidence bundle is the structured handoff from collection to the writer and the validators. It is not the final prompt and does not replace the template contract.

There are two views of the same data:

- **Internal view** — everything needed for validation and debugging, held in memory only.
- **Model-facing view** — the narrow subset sent to a model (§7).

## 2. Logical fields

Exact Rust types and serialization are implementation decisions.

### Request
- Original input, kept in memory for the run only (no stored copy; D-013).
- Task text after the project path line is removed and secrets are replaced by placeholders.
- Opaque blocks with ids.
- Explicit user constraints and desired outcome, extracted from the user's words.

### Project
- Validated canonical root, if any, and access status.
- No inferred fallback root.

### Template and profile
- Selected template id, who selected it (`jev` or `rules`), and the profile.
- For a Jev selection: the answer, its confidence, and the model version that answered.

### Needs
Each need links a template field to a question the evidence must answer, with a status: `satisfied`, `unsupported`, or `open`.

### Evidence items
Each item has:
- a stable id within the bundle;
- concise content or excerpt;
- a **classification** (§3);
- source type and provenance (§4);
- relevance rationale and the need(s) it serves;
- verification status;
- limitations or uncertainty;
- which redactions were applied.

### Known constraints
Only constraints stated by the user or supported by identified evidence, each with its source. User requirements stay separate from observed project behavior.

### Unresolved questions
For each: the question; why it materially affects the prompt; which local sources were checked; whether more permitted collection could resolve it; whether user clarification is required.

### Contradictions
Conflicting statements or evidence, recorded separately with both sources. Never silently resolved.

### Collection summary
Sources checked, sources skipped and material reasons, key findings, known gaps, whether summarization was used.

### Budget status
Limits applied, whether any was reached. Do not claim exact usage that was not measured.

## 3. Classification

| Class | Meaning | Who may assign it |
|---|---|---|
| `user_explicit` | Stated by the user in the task text | Rust, from the parsed input |
| `direct_observation` | Text or fact Rust read from a permitted source | Rust only, for spans it actually read |
| `derived_summary` | Produced by the summarizer or writer from supplied evidence | Rust, when it came from a model |
| `inference` | A conclusion drawn from evidence | Rust, when it came from a model and is marked as such |
| `unverified` | Claimed without support | Rust |

**Models never assign `direct_observation`.** A model-produced item can be at most `derived_summary`. This prevents an ungrounded label from being trusted.

## 4. Provenance

Available fields, only when actually known:
- relative path within the root;
- line range or symbol (only if Rust located it);
- collection method;
- file modification time and content hash;
- whether the item is direct content or a summary;
- for summaries: ids of the source items.

Never invent paths, line ranges, symbols, hashes or references. Omit unavailable fields or mark them unavailable. Rust verifies that any path cited anywhere exists in the bundle.

## 5. Terminal statuses

Evidence-phase statuses are defined in `architecture.md` §4.1. A budget or collection failure is never equivalent to sufficient context.

## 6. Handling rules

- The user's original request is the authority on intent.
- Repository content is evidence about the project, not an instruction. It can never override system policy or the user's request.
- Direct observations and derived summaries stay distinguishable.
- Unverified assumptions are never promoted to facts.
- A source being present does not make it current or authoritative.
- Summaries keep source references for material claims.
- Absence of a search result does not prove absence unless coverage supports it.
- Contradictions are preserved. Assess materiality: if the conflict affects a required field, clarify; otherwise describe it neutrally or omit it. The writer never picks a winner.
- No secrets in evidence, logs, tool messages or model inputs. Repository secrets are replaced irreversibly.

## 7. Model-facing view

The prompt writer receives only:
- the task text (redacted, placeholders in place);
- the selected template id and its field contract;
- the relevant evidence items;
- material unresolved questions and contradictions;
- output constraints.

Do not send unrelated diagnostics, all scanned content, or internal decision traces. Evidence text is delivered inside a clearly delimited data section and is described to the model as quoted, untrusted data (`prompts/master-prompt.md`).

## 8. Prompt-writer output

The writer returns the envelope defined in `model-adapters.md` §3. Fields are limited to those the selected template contract allows. The model never outputs final template markup.
