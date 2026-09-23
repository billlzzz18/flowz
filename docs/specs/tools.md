# Specification: Tool Schemas

## Registered MCP Tools (flowz_ prefix)
1. flowz_workflow_run: items, reducer, run_budget, max_concurrency, failure_policy
2. flowz_workflow_job: job_id -> JobStatus
3. flowz_workflow_cancel: job_id -> { cancelled: bool }
4. flowz_cron_create: name, expression, timezone, command (Shell/Tool/Skill), execution_mode, overlap_policy, misfire_policy, max_runs
5. flowz_cron_list: include_disabled -> Vec<CronDefinition>
6. flowz_cron_cancel: cron_id -> { cancelled: bool }
7. flowz_subagent_delegate: action (spawn, list, steer, stop)
8. flowz_harness_list: list versions & current
9. flowz_harness_show: version -> HarnessVersion
10. flowz_harness_rollback: version -> rollback current
11. flowz_skill_list: list generated and registered skills
12. flowz_evolution_run: trigger evolution round with budget
13. flowz_evolution_status: check current evolution round and tiers
