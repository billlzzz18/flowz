# Guardian command hook — fish
# Logs every command before execution.
# Source: source /path/to/guardian.fish

set -q GUARDIAN_LOG; or set -gx GUARDIAN_LOG $HOME/.guardian/command.log
set -q GUARDIAN_ENABLED; or set -gx GUARDIAN_ENABLED 1

mkdir -p (dirname $GUARDIAN_LOG)

function _guardian_preexec --on-event fish_preexec
    test "$GUARDIAN_ENABLED" != "1" and return
    test -z "$argv[1]" and return
    printf '%s\t%s\t%s\n' \
        (date -u +%Y-%m-%dT%H:%M:%SZ) \
        $PWD \
        "$argv[1]" \
        >> $GUARDIAN_LOG
end
