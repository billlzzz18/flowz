# Guardian command hook — bash
# Logs every command before execution.
# Source: source /path/to/guardian.bash
# ponytail: trap DEBUG is the only stdlib way to intercept in bash

GUARDIAN_LOG="${GUARDIAN_LOG:-$HOME/.guardian/command.log}"
GUARDIAN_ENABLED="${GUARDIAN_ENABLED:-1}"

mkdir -p "$(dirname "$GUARDIAN_LOG")"

_guardian_preexec() {
    [ "$GUARDIAN_ENABLED" != "1" ] && return
    # Skip the trap itself and empty lines
    [[ "$BASH_COMMAND" == "_guardian_preexec" ]] && return
    [[ -z "$BASH_COMMAND" ]] && return
    printf '%s\t%s\t%s\n' \
        "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        "$PWD" \
        "$BASH_COMMAND" \
        >> "$GUARDIAN_LOG"
}

trap '_guardian_preexec' DEBUG
