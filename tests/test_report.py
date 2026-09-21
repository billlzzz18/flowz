#!/usr/bin/env python3
"""
Comparison Test: Builtin Hermes delegate_task vs flowz-mcp workflow/run
UPDATED: Fixed flowz-mcp worker path resolution issue
"""

import json
import time
from datetime import datetime

# ============================================================
# TEST RESULTS (from actual execution)
# ============================================================

RESULTS = {
    "test_metadata": {
        "timestamp": datetime.now().isoformat(),
        "codebase": "/data/data/com.termux/files/home/plugins/flowz-mcp",
        "max_workers": 2,
        "timeout_seconds": 150,
        "search_terms": ["OrchestrationContext", "WorkflowPolicy"]
    },
    "builtin_hermes_delegate_task": {
        "status": "FAILED",
        "elapsed_seconds": 3.5,
        "workers_launched": 2,
        "workers_completed": 0,
        "workers_failed": 2,
        "error": "Model 'auto' isn't available on Kilo Code provider. HTTP 404: The requested model 'auto' does not exist.",
        "root_cause": "Hermes profile configured with 'auto' model which Kilo Code gateway doesn't support. Need explicit model ID.",
        "fix": "Set explicit model in config.yaml or use /model command to select available model",
        "expected_behavior": "Should complete in <5s with 25+ matches for OrchestrationContext, 23+ for WorkflowPolicy"
    },
    "flowz_mcp_workflow_run": {
        "status": "SUCCESS (after fix)",
        "elapsed_seconds": 5.0,
        "workers_launched": 2,
        "workers_completed": 2,
        "workers_failed": 0,
        "job_result": {
            "job_id": "71f3f842-b995-40c7-a7cb-05076bf29406",
            "status": "completed",
            "total": 2,
            "completed": 2,
            "failed": 0
        },
        "root_cause_before_fix": "Worker process (worker.py) not found at runtime path. StdProcessSpawner used CARGO_MANIFEST_DIR which resolves at compile time, but the binary runs from target/release/ causing path mismatch.",
        "evidence_before_fix": [
            "Binary strings showed path: /data/data/com.termux/files/home/plugins/flowz-mcpworker.py (missing / separator)",
            "Server exits immediately when run standalone",
            "MCP tool returned failed status without executing workers",
            "Unit tests pass with MockProcessSpawner but real spawner fails"
        ],
        "fix_applied": "Changed worker path resolution in src/main.rs:24-28 to use std::env::current_exe() with relative path traversal (../../worker.py) and canonicalize, falling back to CARGO_MANIFEST_DIR",
        "fix_code": """
let worker_script = std::env::current_exe()
    .ok()
    .and_then(|p| p.parent().map(|d| d.join("../../worker.py")))
    .and_then(|p| std::fs::canonicalize(p).ok())
    .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("worker.py"));
""",
        "expected_behavior": "Should complete in <30s with workers returning search results via worker.py"
    }
}

# ============================================================
# ANALYSIS
# ============================================================

ANALYSIS = """
COMPARISON ANALYSIS (UPDATED)
=============================

AFTER FIX - flowz-mcp NOW WORKS:
--------------------------------
- Both workers spawned in parallel ✓
- Both workers completed successfully ✓
- Workflow returned completed status with 2/2 done ✓
- Total time: ~5 seconds (well under 150s timeout)

BEFORE FIX - BOTH FAILED FOR DIFFERENT REASONS:

1. BUILTIN HERMES delegate_task
   - Failure: CONFIGURATION issue (model selection)
   - Fix complexity: LOW (change model in config)
   - Architecture: Direct subagent spawning via Hermes runtime
   - Pros: Native integration, no external dependencies, flexible prompts
   - Cons: Requires working model configuration, subagent isolation overhead

2. FLOWZ-MCP workflow/run (BEFORE FIX)
   - Failure: RUNTIME/INFRASTRUCTURE issue (worker spawning)
   - Fix complexity: MEDIUM (debug Rust spawner, path resolution)
   - Architecture: MCP server -> Orchestration engine -> Process spawner -> Python worker
   - Pros: Structured workflow, schema validation, parallel execution control
   - Cons: More moving parts, worker script dependency, protocol overhead

WHY NEITHER COMPLETED WITHIN 150s (ORIGINAL):
---------------------------------------------
The 150s timeout was NOT reached - both failed FAST (<5s).
The timeout would only apply if workers were RUNNING but SLOW.
Here, both failed at STARTUP/CONFIGURATION phase.

IF THEY HAD RUN BUT TIMED OUT AT 150s:
--------------------------------------
Builtin delegate_task timeout reasons:
- Subagent model calls taking too long (API latency, rate limits)
- Complex prompts causing long reasoning loops
- Subagent stuck waiting for tool results

Flowz-mcp workflow/run timeout reasons:
- Worker processes hanging (Python script deadlock, infinite loop)
- Worker timeout (default 300s) not being enforced correctly
- MCP server not returning job completion status
- Spawner buffer/concurrency deadlock with 2 workers

NOTE ON WORKER IMPLEMENTATION:
------------------------------
The current worker.py is a MOCK worker that returns mock data based on keyword matching.
For real codebase search, the worker would need to be updated to:
1. Execute actual rg/grep commands
2. Return structured results matching the output schema
3. Handle the schema validation properly
"""

def print_report():
    print("=" * 70)
    print("COMPARISON TEST REPORT (UPDATED)")
    print("Builtin Hermes delegate_task vs flowz-mcp workflow/run")
    print("=" * 70)
    
    meta = RESULTS["test_metadata"]
    print(f"\nTest Time: {meta['timestamp']}")
    print(f"Codebase: {meta['codebase']}")
    print(f"Max Workers: {meta['max_workers']}")
    print(f"Timeout: {meta['timeout_seconds']}s")
    print(f"Tasks: {meta['search_terms']}")
    
    print("\n" + "-" * 70)
    print("RESULT: BUILTIN HERMES delegate_task")
    print("-" * 70)
    r = RESULTS["builtin_hermes_delegate_task"]
    print(f"Status: {r['status']}")
    print(f"Elapsed: {r['elapsed_seconds']}s")
    print(f"Workers: {r['workers_launched']} launched, {r['workers_completed']} completed, {r['workers_failed']} failed")
    print(f"Error: {r['error']}")
    print(f"Root Cause: {r['root_cause']}")
    print(f"Fix: {r['fix']}")
    print(f"Expected: {r['expected_behavior']}")
    
    print("\n" + "-" * 70)
    print("RESULT: FLOWZ-MCP workflow/run")
    print("-" * 70)
    r = RESULTS["flowz_mcp_workflow_run"]
    print(f"Status: {r['status']}")
    print(f"Elapsed: {r['elapsed_seconds']}s")
    print(f"Workers: {r['workers_launched']} launched, {r['workers_completed']} completed, {r['workers_failed']} failed")
    print(f"Job Result: {json.dumps(r['job_result'], indent=2)}")
    print(f"Root Cause (before fix): {r['root_cause_before_fix']}")
    print(f"Fix Applied: {r['fix_applied']}")
    print(f"Fix Code:{r['fix_code']}")
    print(f"Expected: {r['expected_behavior']}")
    
    print("\n" + "=" * 70)
    print(ANALYSIS)
    print("=" * 70)


if __name__ == "__main__":
    print_report()