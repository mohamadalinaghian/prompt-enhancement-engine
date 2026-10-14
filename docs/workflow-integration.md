# Workflow and Editor Integration

This document defines the contract between Karabiner, the shell script, and the Rust application. Exact Karabiner configuration and keystroke-automation mechanics are not specified here; the implementing agent must verify them against current primary documentation (see §9).

## 1. Principle

The shell script is thin and has no prompt logic. The Rust application is a pure function from text to (text, status) plus notifications. This keeps Rust fully testable without an editor.

## 2. Shell script responsibilities (DEFAULT)

In order:

1. Record the frontmost application identity.
2. Save the current clipboard contents.
3. Select all in the focused text box and copy it.
4. Run the Rust binary with the copied text on stdin and read stdout, stderr and the exit code.
5. If the exit code means "paste" (see §4): confirm the frontmost application is unchanged, then paste the stdout text as a single paste action.
6. If the frontmost application changed during the run: do **not** paste. Put the result on the clipboard instead, and notify the user that the result was not pasted.
7. Restore the previous clipboard (after a short, configurable, measured delay that lets the paste finish).
8. Show a notification built from stderr.

The shell must never modify the editor on any non-paste exit code.

## 3. Recovery model (LOCKED: D-013)

No copy of the original text is stored. Recovery relies on the editor's undo.

Consequences the implementation must respect:

- Rust never writes to the editor. Only the shell does, and only on success or clarification.
- The replacement is a single paste action, so one undo restores the original.
- Some inputs (for example terminal-style text boxes) may not support undo. This is an accepted limitation, to be recorded in user-facing documentation.
- If the target application changed during the run, nothing is pasted (§2 step 6).

## 4. Exit-code contract (DEFAULT)

Exact numbers are an implementation detail; the **classes** below are required and the shell must treat them as follows.

| Class | Rust stdout | Shell action |
|---|---|---|
| Success (`RENDERED`) | Final prompt text | Paste, notify |
| Clarification (`CLARIFICATION_EMITTED`) | Clarification block + original text | Paste, notify |
| Busy | empty | Notify only |
| Failure (any other terminal status) | empty | Leave text unchanged, notify with the status and a short reason from stderr |

stderr carries one short human-readable line: the status, and the `JEV_DEGRADED` flag when present. It never contains prompt text, file contents or secrets.

## 5. Single-flight (LOCKED: D-014)

- Rust takes an exclusive lock in a user-private runtime location at start.
- If the lock is held by a live process, exit `BUSY` immediately. Nothing is queued and the first run is not disturbed.
- Stale locks (dead process) must be detected and cleared.
- The lock is released on every exit path, including panics and deadline expiry.

## 6. Deadline (LOCKED: D-015 implies the need)

There is no cancel shortcut in v1, so a stuck run can only end by deadline.

- A single overall deadline covers the whole run. Per-call timeouts are derived from it.
- **OPEN:** the value. The implementing agent proposes a provisional value, records it as PROVISIONAL, and measures before fixing it.
- On expiry: stop all work, release the lock, exit `DEADLINE_EXCEEDED`, leave the text unchanged.

## 7. Input parsing

Order of checks on the text read from stdin:

1. **Empty or whitespace only** → `INPUT_EMPTY`.
2. **Leading clarification block** (§8): the text begins with the clarification BEGIN sentinel → `STALE_CLARIFICATION_MARKER`. No model calls, no filesystem access.
3. **Project path line** (LOCKED: D-004): the first line is a project path only if it begins with `~/` (DEFAULT: other forms, including absolute `/…` paths, are not accepted in v1 and the line is treated as ordinary task text with no project context; **OPEN** whether to accept absolute paths).
   - Expand `~` using the current user's home directory.
   - Canonicalize. It must be an existing, readable directory. Otherwise → `PROJECT_UNAVAILABLE`. Never fall back to another directory.
   - On success, record the root and remove that line from the task text before any model sees it.
   - The path line is **not** re-emitted in the final output (LOCKED: D-004).
   - The path must never become a task requirement.
4. **Opaque blocks:** fenced code or log blocks in the remaining text are given ids. They are shown to models with ids and must reappear byte-identical in the output (DEFAULT; see `template-contracts.md` §6).

Unicode, CRLF and large inputs must be handled without altering the user's text semantically. Over-limit inputs follow a recorded size policy (**OPEN:** limit).

## 8. Clarification block (LOCKED: D-007; syntax DEFAULT)

When the writer returns `NEEDS_CLARIFICATION`, or collection ends in `NEEDS_CLARIFICATION`, the tool pastes a block above the **complete original input** (including the project path line, since no output was produced). The block must be visually distinct and machine-recognizable.

Proposed syntax (DEFAULT; the owner may change it, but it must be stable and versioned):

```
⚠ PROMPT ENHANCER — CLARIFICATION NEEDED
Delete this whole block, add your answer to your prompt, and run the shortcut again.
[[PE-CLARIFY-v1:BEGIN]]
Question: <one concrete question>
Why it blocks: <one or two sentences>
[[PE-CLARIFY-v1:END]]
<original input, unchanged>
```

Rules:

- Detection is line-anchored: the text must **begin** with the visible header line followed by the BEGIN sentinel line, or begin directly with the BEGIN sentinel. Sentinels appearing elsewhere are ordinary text.
- If a block is present at the start of the input on a later run, the run stops with `STALE_CLARIFICATION_MARKER` and the notification asks the user to delete the block. The tool never strips it itself and never treats its content as task text (LOCKED: manual deletion).
- The block contains no chain-of-thought, no internal decision trace, no file contents and no secrets.
- One question per block.
- The original input after the block is byte-identical to what the tool received.
- Secrets in the original input are **not** replaced by placeholders in this output; the original text is returned as received.

## 9. Items the implementing agent must verify from primary sources

- How Karabiner-Elements launches a shell command from a shortcut, and the environment that command receives.
- How to select all, copy, paste and read/write the clipboard from a shell script on the current macOS version, and which system permissions (for example Accessibility) that requires.
- How to detect the frontmost application from a script.
- How to preserve the previous clipboard, and which content types can be preserved (**OPEN:** plain text only vs richer types).
- How to show a user notification from a script.

Record each finding and its source in the decision log. Do not assume.

## 10. Failure behavior summary

| Situation | Behavior |
|---|---|
| Rust crashes or times out | Shell sees a failure code, leaves the editor unchanged, restores the clipboard, notifies. |
| Shell cannot read Rust output | Same. |
| Target application changed mid-run | No paste; result placed on clipboard; notification. |
| Paste issued | Reported only as "paste issued". The shell cannot confirm the editor applied it. |
