# ADR-0031: System One Decision Backend

**Status:** Accepted
**Date:** 2026-10-01

## Context

Flowz needs a bounded, typed interface for probability-bearing decisions that can be served by Jev, local Laya, or tev1-compatible endpoints. These providers share the `/v1/systemone` protocol, but provider/model capabilities and runtime behavior must be verified independently. A model response must not become control flow before it has been validated by application code.

## Decision

1. Add a standalone `crates/decider` workspace member with typed `Choice`, `Noul`, and `Score` questions, typed request/response envelopes, and a reusable HTTP client for `/v1/systemone`.
2. Keep this crate transport-focused. It returns validated JSON types and transport errors; it does not route agents, change budgets, make Guardian admission decisions, or alter Supervisor behavior.
3. Validate non-empty model/state/questions, question names and instructions, and minimum choice/score criteria before sending requests. Preserve provider response fields that may vary by implementation.
4. Use optional bearer authentication, configurable request timeout, and no credential-bearing `Debug` or log output.
5. Exercise the wire contract with an in-process mock HTTP server. Run real Jev/Laya/tev1 capability probes separately with the same synthetic Choice/Noul/Score query; do not infer live compatibility solely from documentation or mocks.
6. Keep downstream integrations advisory until separately reviewed. Guardian integrations must never weaken an existing `Block` action; remote calls must use sanitized state.

## Consequences

- Protocol handling and request validation can be tested without provider credentials or model downloads.
- The crate can be reused by each backend by changing endpoint, model, and optional key.
- Live capability, calibration, language support, and latency remain deployment-specific and require real backend probes before integration.
- Supervisor event infrastructure and the Guardian admission API remain separate implementation work.

## Security

- Callers are responsible for removing credentials and user runtime data from `state` before sending it to a remote backend.
- API keys are supplied at runtime and must not be written to repository files, logs, error messages, or test fixtures other than synthetic mock values.
- The client reports HTTP status without echoing response bodies, which may contain sensitive request data.
