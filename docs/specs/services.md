# Specification: Service Layer Contracts

## Traits
- WorkflowService: run(), job(), cancel(), watch()
- CronService: create(), list(), cancel(), run_now(), history()
- SupervisorService: subscribe(), emit(), findings(), acknowledge()
- SubagentService: delegate(), list(), steer(), stop()
- HarnessService: list_versions(), show_version(), current_version(), rollback_to(), list_bank_cells(), show_bank_cell(), diff()
- SkillService: list(), show(), register(), unregister(), find_by_intent()
- EvolutionService: run_round(), synthesize_skills(), status()
- TrajectoryService: record_step(), finalize(), query_recent(), query_successful(), get()
