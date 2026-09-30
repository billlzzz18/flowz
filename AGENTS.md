# AGENTS.md — Instructions for AI Coding Agents

## 1. Architecture Guidelines

- Read `docs/README.md`, `docs/plans.csv`, `docs/specs.csv`, and ADRs before modifying code.
- Check actual module paths under `src/` before assuming component structure.
- Distinguish specification gaps from existing implementation; do not treat unwritten modules as completed.
- Service layer (`src/service/`) owns business logic; MCP tools and CLI are thin wrappers.
- Adapter boundary (`src/adapter/`) isolates external formats (Claude, Codex, Antigravity, Hermes).
- MCP namespace: all tool names must use `flowz_` prefix. Cron is user-initiated only.
- Supervisor uses typed events and rule-based screening before LLM evaluation.
- Execution backends (`local`, `docker`, `modal`, `daytona`) adhere to capability discovery.
- Keep domain entities isolated from transport layers and storage drivers.

## 2. Invariants & Security

- Hermes profile layout is the standard hub format for agent distribution (ADR-0026).
- Maintain strict ownership split: distribution-owned vs config-override vs user-owned.
- Strictly exclude credentials and user runtime data: `auth.json`, `.env`, `memories/`, `sessions/`, `state.db*`.
- Cron misfire policy is `run_once` on startup to avoid thundering herd (ADR-0004).
- Remote execution backend preserves local state ownership (ADR-0028).
- Hybrid Topology (ADR-0030): Local Client UX (AGY) paired with Cloud Sandbox Execution (SBX) and Pre-Admission Quality Guard (Guardian).
- Pre-admission gatekeeping: Remote sandbox artifacts must pass Guardian screening before admission to local state (ADR-0030).

## 3. Testing Guidelines

- Never report work as passed without real execution output from tests or runtime checks.
- Report toolchain constraints honestly (e.g. compiler unavailable, memory pressure, missing runtime).
- Practice test-driven validation: ensure test vector covers edge cases and regression scenarios.
- Run tests via cargo or python verification scripts with output logged to terminal.
- Maintain regression assertions across all existing unit and integration suites.
- Verify environment and dependencies before running test suites.

## 4. Documentation & Workflow Guidelines

- Documentation is the sole lifeline of the project; never overwrite, truncate, or drop specs.
- Use `scripts/register_decision.py` to register and audit ADRs into `docs/decisions.csv`.
- All ledger modifications in `docs/*.csv` must use atomic operations and valid formatting.
- Respect progressive disclosure: keep architecture, testing, docs, and workflow gates aligned.
- Keep ledger state synchronized with actual disk paths and file trees.
- Never delete or modify source ADRs without explicit direction.
- Review diffs carefully prior to saving changes.
