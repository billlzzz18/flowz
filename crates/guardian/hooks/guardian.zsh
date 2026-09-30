# Guardian command hook — zsh
# Logs every command before execution.
# Source: source /path/to/guardian.zsh

GUARDIAN_LOG="${GUARDIAN_LOG:-$HOME/.guardian/command.log}"
GUARDIAN_ENABLED="${GUARDIAN_ENABLED:-1}"

mkdir -p "$(dirname "$GUARDIAN_LOG")"

_guardian_preexec() {
    [ "$GUARDIAN_ENABLED" != "1" ] && return
    [[ -z "$1" ]] && return
    printf '%s\t%s\t%s\n' \
        "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
        "$PWD" \
        "$1" \
        >> "$GUARDIAN_LOG"
}

autoload -Uz add-zsh-hook
add-zsh-hook preexec _guardian_preexec
