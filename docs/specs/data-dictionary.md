# Specification: Data Dictionary

## 1. Budget Types
- TimeBudget: max_duration_seconds, heartbeat_interval_seconds, on_timeout (Terminate, Report, Escalate), termination_grace_seconds
- IterationBudget: max_iterations, max_tool_calls, pressure_threshold (default 0.8), on_exhausted (StopAndSummarize, Terminate, Escalate)
- RunBudget: max_wall_clock_seconds, max_total_agent_calls (default 10,000), enable_pressure_warnings

## 2. Workflow Types
- WorkflowItem: id, brief, prompt, output_schema, time_budget, iteration_budget, role (Leaf, Orchestrator), input_files, sandbox (None, Docker, Remote), effort_level
- ReducerSpec: id, brief, prompt, input_item_ids, output_schema, time_budget, iteration_budget, sandbox, effort_level
- RunRequest: items, reducer, run_budget, max_concurrency, failure_policy (Collect, FailFast)

## 3. Harness Types
- PatchId, HarnessVersionId, TrajectoryId, SessionId, KnowledgeEntryId, CronId (Uuid wrappers)
- HarnessPatch: id, parent (SemVer), component (Prompt, Knowledge, Runtime, Config), pathology (ThinkingRunaway, PrematureFinalization, SilentFailure, LoopExhaustion, ContextDrift, CostOverrun, ToolProtocolError), description, diff (HarnessDiff), proposed_by, proposed_at, evidence, change_type
- HarnessVersion: version, created_at, parent, content_hash (canonical JSON SHA-256), components (prompt, knowledge, runtime, config)
- SemVer: major, minor, patch
- HarnessGeneBank: cells HashMap<CellKey, PreservedHarness> where CellKey = component x pathology

## 4. Trajectory Types
- Trajectory: id, session_id, task, steps, outcome, harness_version, started_at, ended_at
- TrajectoryStep: index, step_type (LlmCall, ToolCall, Decision, Recovery, Observation), input, output, duration_ms, error, timestamp
- TrajectoryOutcome: Success { user_confirmed, success_criteria_met, artifacts }, Failure { pathology, error }, Timeout, Cancelled

## 5. Skill Types
- SkillDefinition: name, description, creator ("flowz"), created_at, source_trajectory, harness_version, body, frontmatter
- SkillFrontmatter: trigger (intent_match, complexity_threshold), verification (system_pass, user_confirmed, activation_verified, sample_size >= 3), success_criteria, provenance

## 6. Evolution Types
- EvolutionBudget: max_candidates, max_eval_samples, max_duration_seconds, max_cost_usd
- EvolutionRequest: trajectories, current_harness, pathology_hypothesis, budget
- ModelTier: Tier1 (Haiku, GPT-4o-mini), Tier2 (Sonnet, GPT-5), Tier3 (Opus, GPT-5.6), Tier4 (Frontier)

## 7. Discovery Types
- DiscoveryResult: local (cpu, ram, gpu, disk), cli_agents, remote_providers, credentials, docker, evolver_tier, scanned_at
- CredentialSource: Env, Keychain, File, OAuth
- Preferences: cost_preference, latency_preference, region

## 8. Error Types
- FlowzError: Validation, Execution, Backend, Storage, Adapter, Evolution, Worker, Config
