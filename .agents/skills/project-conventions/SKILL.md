---
name: "flowz-conventions"
description: "Core conventions and architectural patterns for the Flowz codebase"
domain: "project-conventions"
confidence: "high"
source: "flowz-architecture"
---

# Flowz Project Conventions

## Context
Flowz is an autonomous MCP Agent Orchestrator written in Rust. It enforces strict separation between documentation ledgers, domain contracts, service boundaries, and external execution adapters.

## Core Patterns

### 1. Documentation-First & Specification Primacy
- **Never drop or truncate documentation:** All architectural intents, ADRs (0001–0037), and 7 specification books in `docs/specs/` are the absolute source of truth.
- **Specification Gap Rule:** Paths listed in `docs/plans.csv` that do not exist under `src/` are specification gaps, never finished code.

### 2. Error Handling
- **Domain Errors:** All errors are strictly typed using `thiserror` in `src/error.rs` (`FlowzError`).
- **No Untyped Bubbling in Services:** Services and MCP tools must map low-level library errors into typed variants (e.g. `StorageError`, `WorkerError`, `EvolutionError`).
- **Fail-Closed Semantics:** Security boundaries, capability handle checks, and quota validations fail closed.

### 3. Architecture & Separation of Concerns
- **Domain Layer (`src/domain/`):** Pure data models, enums, serialization schemas. Zero network/IO logic.
- **Service Layer (`src/service/`):** Async trait definitions (`#[async_trait]`) decoupling business operations from infrastructure.
- **Adapter Boundary (`src/adapter/`, `src/execution/`):** External runtimes (Modal, Daytona, Hermes, Claude, Codex) isolated behind adapter traits.
- **Tool Protocol:** All MCP tools adhere strictly to the `flowz_` prefix and FastMCP / Hermes JSON-RPC stdio transport.

### 4. Dual-Track Evolution & Trajectory
- **Dual Representation:** In-memory struct (`Trajectory`) for real-time telemetry; ShareGPT JSONL format (`conversations` array with `<think>`, `<tool_call>`, `<tool_response>`) for on-disk persistence and evaluation datasets.
- **Dual-Track Trigger:** Live Concurrent Observer (non-blocking real-time telemetry) + Background Autonomous Evolver daemon & 03:00 daily sweep.

### 5. Testing & Verification Gates
- **Red-Green-Refactor:** Tests and verification vectors in `tests/` validate contracts before implementation.
- **Statistical Rigor:** Evolution screening requires 4-gate verification with paired z-test ($z \ge 1.96, n \ge 26$).
- **No Fabricated Output:** Status reports must reflect actual execution output and compiler/test diagnostics.

## Anti-Patterns
- **Overwriting specs with speculative code:** Never substitute concise code stubs for detailed specification files.
- **Direct stdio pollution:** `stdout` is strictly reserved for JSON-RPC transport; all debug logging must go to `stderr` via `tracing`.
- **Touching user-owned paths:** Never delete or overwrite `memories/`, `sessions/`, `state.db*`, `.env`, `credentials/`, or `trajectory/`.
