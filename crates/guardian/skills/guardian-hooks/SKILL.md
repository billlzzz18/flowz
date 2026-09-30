---
name: guardian-hooks
description: >
  Active Guardian shell hook system. Source in your shell config to
  capture every command, import to metrics DB, and run audit on demand.
  Use when: "setup guardian hooks", "enable guardian", "guardian hook",
  "activate guardian", or any request to start monitoring shell commands.
---

# Guardian Hooks — Active Monitoring System

## Hook Files

Source ONE of these in your shell config (`.bashrc`, `.zshrc`, `config.fish`, `$PROFILE.ps1`):

- `~/.hermes/plugins/guardian/hooks/guardian.bash`
- `~/.hermes/plugins/guardian/hooks/guardian.zsh`
- `~/.hermes/plugins/guardian/hooks/guardian.fish`
- `~/.hermes/plugins/guardian/hooks/guardian.ps1`

## What They Do

Every command you run:
1. Captured via `trap DEBUG` (bash), `preexec` (zsh), `fish_preexec` (fish), prompt wrapper (PS)
2. Appended to `~/.guardian/command.log` as TSV: `timestamp<TAB>cwd<TAB>command`
3. Zero overhead when disabled (`GUARDIAN_ENABLED=0`)

## Active Commands (after sourcing)

| Command | Purpose |
|---------|---------|
| `gcheck [file]` | Run guardian analysis on file (default: src/lib.rs) |
| `ghook [repo]` | Install git hooks in repo |
| `gaudit` | Import log + run audit (alias for `guardian import-log && guardian-audit`) |

## Environment Variables

```bash
export GUARDIAN_BIN=guardian          # override binary path
export GUARDIAN_ENABLED=1             # set 0 to disable capture
export GUARDIAN_LOG=~/.guardian/command.log  # custom log path
```

## Full Flow

```
User runs commands
       ↓
Hook captures → ~/.guardian/command.log
       ↓
User runs: guardian import-log
       ↓
~/.guardian/metrics.db (commands table, schema v5)
       ↓
CommandLogDetector analyzes:
  - CommandLoop (same cmd N times)
  - Destructive (rm -rf, git push --force, etc)
  - CdOscillation (cd back-forth)
  - RepeatedFailure (same cmd fails twice)
       ↓
guardian-audit presents findings with tags
```

## Integration with Ponytail Audit

When you want AI slop review:

```bash
# After a coding session:
gaudit

# Output:
# GUARDIAN AUDIT — commands
# Scanned: 127 commands
#
# findings:
# 1. ...: slop: unwrap() without error handling
# 2. ...: yagni: Config struct has one consumer
# 3. ...: shrink: manual loop → .map().collect()
#
# net: 3 findings, ~15 deletable lines
```

## Session Correlation

If `GUARDIAN_SESSION_ID` is set in the environment when sourcing,
commands are tagged with that session ID in the DB, enabling
cross-referencing with prompt analysis via `external_session_id`.

```bash
export GUARDIAN_SESSION_ID="session-$(date +%s)"
source ~/.hermes/plugins/guardian/hooks/guardian.bash
```

## Quick Setup

```bash
# One-liner for bash/zsh:
echo 'source ~/.hermes/plugins/guardian/hooks/guardian.bash' >> ~/.bashrc
# or
echo 'source ~/.hermes/plugins/guardian/hooks/guardian.zsh' >> ~/.zshrc

# For fish:
echo 'source ~/.hermes/plugins/guardian/hooks/guardian.fish' >> ~/.config/fish/config.fish

# For PowerShell:
Add-Content $PROFILE '. ~/.hermes/plugins/guardian/hooks/guardian.ps1'
```

## Boundaries

- Hooks are **capture only** — no analysis in the hook itself
- Analysis runs on demand via `guardian import-log`
- Log file rotates manually or via cron (user responsibility)
- No network calls, no background daemons