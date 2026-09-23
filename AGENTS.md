# AGENTS.md — Flowz Repository Instructions for AI Coding Agents

## 1. Project Overview & Identity
- **Repository:** `billlzzz18/flowz`
- **Goal:** Rust-based MCP server & orchestrator for multi-agent workflows (time/iteration budgets, context compression, cron scheduling, subagent live control, self-evolving harness bank).
- **Core Standard:** Zero-defect implementation, YAGNI, TDD (Red-Green-Refactor), strict adherence to ADRs (0001–0037).

---

## 2. Architecture & Design Rules (Enforced by ADRs)
1. **Namespace & Tools (ADR-0006):**
   - Every MCP tool MUST use the `flowz_` prefix (e.g., `flowz_workflow_run`, `flowz_cron_create`).
   - Tools are self-registering via `McpTool` trait in their respective files (ADR-0021).
2. **Budget Triple & Separation (ADR-0001, 0002, 0020):**
   - `TimeBudget` (wall-clock timeout per item) is strictly separated from cron scheduling.
   - Every workflow item requires `TimeBudget` and `IterationBudget`.
   - `RunBudget` governs total workflow wall-clock limit and maximum total agent calls across reducers.
3. **Cron Subsystem (ADR-0004, 0005, 0036):**
   - Subsystem in `src/cron/` isolated from workflow runner. User-initiated only (regular agents cannot schedule cron).
   - Cron misfire policy: `run_once` on startup (never thundering herd catch-up).
4. **Service Layer (ADR-0007, 0008):**
   - All business logic lives in `src/service/`. MCP and CLI are thin adapters passing `InvocationContext` (`Mcp`, `Cli`, `Cron`, `Evolution`).
5. **Supervisor & Events (ADR-0009, 0010, 0011):**
   - Typed events (`ObservedEvent` struct) only, never string log scraping.
   - Rule-based first; LLM supervision is strictly optional with circuit breakers.
   - Scope is restricted to flowz traffic only.
6. **Self-Evolution & HarnessBank (ADR-0030, 0031, 0032, 0033, 0034, 0035, 0037):**
   - 4 Gates for candidate patches: Validity -> Activation -> Significance (z >= 1.96, n >= 26) -> Gain.
   - Evolver Tier MUST exceed Task Tier + 1 (prevent self-eval bias).
   - Admitted patches versioned with SemVer and canonical JSON SHA-256 content hashes.

---

## 3. Tooling & Development Workflow
- **Build & Verification:**
  - Format: `cargo fmt --all -- --check`
  - Lint: `cargo clippy --all-targets --all-features -- -D warnings`
  - Test: `cargo test --workspace --all-features`
- **ADR Maintenance:**
  - DO NOT edit `docs/decisions.csv` manually.
  - Run: `python scripts/register_adr.py --id <ID> --topic "<Topic>" --decision "<Decision>" --rationale "<Rationale>"` or `--scan`.
- **Plans & Tasks:**
  - Always check `docs/plans.csv` and `docs/specs.csv` before implementing.
  - Update status in `docs/plans.csv` upon completing tasks (⬜ -> 🔄 -> ✅).

---

## 4. Coding Standards & Idioms
- Native Rust stdlib first, minimal external dependencies.
- Clear error handling with `thiserror` (`FlowzError` variants) and `anyhow` for internal pipelines.
- Comments explaining *intent* and *why* in Thai or concise English.
- No dummy/mock replacements in production paths.
