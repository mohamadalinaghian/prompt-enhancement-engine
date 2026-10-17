#!/usr/bin/env bash
# ==============================================================================
# Prompt Enhancement Engine — macOS Runner
# Integrates with Karabiner-Elements shortcut, captures active text box,
# executes the Rust `pe` binary, and safely replaces text or notifies.
# ==============================================================================

set -uo pipefail

export LC_ALL=en_US.UTF-8
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:$PATH"

# Resolve environment keys if launched in a minimal environment (e.g. Karabiner env -i)
if [ -z "${OPENROUTER_API_KEY:-}" ] || [ -z "${GROQ_API_KEY:-}" ] || [ -z "${TYPESAFE_API_KEY:-}" ]; then
    for shell_bin in "/bin/zsh" "/bin/bash" "/opt/homebrew/bin/fish"; do
        if [ -x "$shell_bin" ]; then
            if [ "$shell_bin" = "/opt/homebrew/bin/fish" ]; then
                [ -z "${OPENROUTER_API_KEY:-}" ] && OPENROUTER_API_KEY=$("$shell_bin" -c "echo -n \$OPENROUTER_API_KEY" 2>/dev/null || true)
                [ -z "${GROQ_API_KEY:-}" ] && GROQ_API_KEY=$("$shell_bin" -c "echo -n \$GROQ_API_KEY" 2>/dev/null || true)
                [ -z "${TYPESAFE_API_KEY:-}" ] && TYPESAFE_API_KEY=$("$shell_bin" -c "echo -n \$TYPESAFE_API_KEY" 2>/dev/null || true)
                [ -z "${OPENAI_API_KEY:-}" ] && OPENAI_API_KEY=$("$shell_bin" -c "echo -n \$OPENAI_API_KEY" 2>/dev/null || true)
            else
                [ -z "${OPENROUTER_API_KEY:-}" ] && OPENROUTER_API_KEY=$("$shell_bin" -l -c "echo -n \$OPENROUTER_API_KEY" 2>/dev/null || true)
                [ -z "${GROQ_API_KEY:-}" ] && GROQ_API_KEY=$("$shell_bin" -l -c "echo -n \$GROQ_API_KEY" 2>/dev/null || true)
                [ -z "${TYPESAFE_API_KEY:-}" ] && TYPESAFE_API_KEY=$("$shell_bin" -l -c "echo -n \$TYPESAFE_API_KEY" 2>/dev/null || true)
                [ -z "${OPENAI_API_KEY:-}" ] && OPENAI_API_KEY=$("$shell_bin" -l -c "echo -n \$OPENAI_API_KEY" 2>/dev/null || true)
            fi
            [ -n "${GROQ_API_KEY:-}" ] && [ -n "${OPENROUTER_API_KEY:-}" ] && break
        fi
    done
    export OPENROUTER_API_KEY GROQ_API_KEY TYPESAFE_API_KEY OPENAI_API_KEY
fi

# 1. Debounce to ensure physical modifier keys from shortcut are released
sleep 0.35

# 2. Record frontmost application identity (workflow-integration.md §2)
INITIAL_APP=$(osascript -e 'tell application "System Events" to get name of first application process whose frontmost is true' 2>/dev/null || echo "Unknown")
INITIAL_APP=$(echo "$INITIAL_APP" | xargs)

# 3. Locate `pe` binary
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

if [ -n "${PE_BIN:-}" ] && [ -x "$PE_BIN" ]; then
    # Custom binary override
    :
elif command -v pe >/dev/null 2>&1; then
    PE_BIN=$(command -v pe)
elif [ -x "$REPO_DIR/target/release/pe" ]; then
    PE_BIN="$REPO_DIR/target/release/pe"
elif [ -x "$REPO_DIR/target/debug/pe" ]; then
    PE_BIN="$REPO_DIR/target/debug/pe"
elif [ -x "$HOME/.local/bin/pe" ]; then
    PE_BIN="$HOME/.local/bin/pe"
else
    osascript -e 'display notification "Prompt Enhancement binary `pe` not found in PATH" with title "Prompt Enhancer" sound name "Basso"'
    exit 1
fi

# 4. Save existing clipboard contents
PREV_CLIPBOARD="$(pbpaste 2>/dev/null || echo "")"

# 5. Select all and copy from focused text box
# For terminal emulators use Ctrl+A / Ctrl+C; for GUI apps use Cmd+A / Cmd+C
if echo "$INITIAL_APP" | grep -qiE 'Terminal|iTerm2|Ghostty|Warp|kitty|alacritty'; then
    osascript 2>/dev/null <<EOF
    tell application "System Events"
        keystroke "a" using control down
        delay 0.05
        keystroke "c" using control down
    end tell
EOF
else
    osascript 2>/dev/null <<EOF
    tell application "System Events"
        keystroke "a" using command down
        delay 0.05
        keystroke "c" using command down
    end tell
EOF
fi

# Wait up to 500ms for clipboard content
COPIED=""
for _ in $(seq 1 10); do
    COPIED="$(pbpaste 2>/dev/null || echo "")"
    if [ -n "$COPIED" ]; then
        break
    fi
    sleep 0.05
done

# If nothing was copied, notify user and restore previous clipboard
if [ -z "$COPIED" ]; then
    printf "%s" "$PREV_CLIPBOARD" | pbcopy
    osascript -e 'display notification "Empty text box: nothing to enhance." with title "Prompt Enhancer"'
    exit 0
fi

# 6. Run Rust binary with copied text on stdin
STDERR_FILE=$(mktemp /tmp/pe_stderr.XXXXXX)
STDOUT_PAYLOAD=$(printf "%s" "$COPIED" | "$PE_BIN" --app "$INITIAL_APP" 2> "$STDERR_FILE")
EXIT_CODE=$?
# Extract final machine-readable status (last line of stderr, per workflow-integration.md §4)
STATUS_MSG=$(tail -n 1 "$STDERR_FILE" | tr -d '\r' || echo "Completed")
rm -f "$STDERR_FILE"

ESCAPED_MSG=$(echo "$STATUS_MSG" | sed 's/"/\\"/g')

# 7. Evaluate exit code and manage pasting / notifications (workflow-integration.md §4)
case "$EXIT_CODE" in
    0|2)
        # Check if frontmost application changed during execution (workflow-integration.md §2 step 6)
        CURRENT_APP=$(osascript -e 'tell application "System Events" to get name of first application process whose frontmost is true' 2>/dev/null || echo "Unknown")
        CURRENT_APP=$(echo "$CURRENT_APP" | xargs)

        if [ "$CURRENT_APP" != "$INITIAL_APP" ]; then
            # Application changed: DO NOT paste into wrong window. Leave result on clipboard.
            printf "%s" "$STDOUT_PAYLOAD" | pbcopy
            osascript -e "display notification \"Focus changed to $CURRENT_APP. Result placed on clipboard.\" with title \"Prompt Enhancer\" subtitle \"$ESCAPED_MSG\" sound name \"Glass\""
        else
            # Application unchanged: execute single-paste replacement
            printf "%s" "$STDOUT_PAYLOAD" | pbcopy
            sleep 0.05

            if echo "$INITIAL_APP" | grep -qiE 'Terminal|iTerm2|Ghostty|Warp|kitty|alacritty'; then
                osascript 2>/dev/null <<EOF
                tell application "System Events"
                    keystroke "v" using command down
                end tell
EOF
            else
                osascript 2>/dev/null <<EOF
                tell application "System Events"
                    keystroke "v" using command down
                end tell
EOF
            fi

            # Short measured delay (100ms) to allow paste to finish, then restore clipboard
            sleep 0.1
            printf "%s" "$PREV_CLIPBOARD" | pbcopy

            if [ "$EXIT_CODE" -eq 0 ]; then
                osascript -e "display notification \"Prompt enhanced successfully for $INITIAL_APP\" with title \"Prompt Enhancer\" subtitle \"$ESCAPED_MSG\" sound name \"Glass\""
            else
                osascript -e "display notification \"Clarification block prepended to prompt\" with title \"Prompt Enhancer\" subtitle \"$ESCAPED_MSG\" sound name \"Basso\""
            fi
        fi
        ;;

    3)
        # Busy (another run already in flight)
        printf "%s" "$PREV_CLIPBOARD" | pbcopy
        osascript -e "display notification \"Another enhancement is already in progress\" with title \"Prompt Enhancer\" subtitle \"BUSY\" sound name \"Basso\""
        ;;

    *)
        # Failure: leave editor text untouched and notify
        printf "%s" "$PREV_CLIPBOARD" | pbcopy
        osascript -e "display notification \"Enhancement aborted: $ESCAPED_MSG (editor untouched)\" with title \"Prompt Enhancer\" subtitle \"FAILED\" sound name \"Basso\""
        ;;
esac
