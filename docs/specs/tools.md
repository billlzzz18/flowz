# Specification: Tool Schemas (Input/Output/JSON Schemas)

Source: Flowz Implementation Specification (Part E)

## E.4 flowz_cron_create
Input:
```json
{
  "type": "object",
  "required": ["name", "expression", "timezone", "command", "execution_mode"],
  "properties": {
    "name": { "type": "string" },
    "expression": { "type": "string" },
    "timezone": { "type": "string" },
    "command": {
      "oneOf": [
        { "type": "object", "required": ["Shell"], "properties": { "Shell": { "type": "object", "required": ["command"], "properties": { "command": { "type": "string" }, "args": { "type": "array", "items": { "type": "string" } } } } } },
        { "type": "object", "required": ["Tool"], "properties": { "Tool": { "type": "object", "required": ["name", "args"], "properties": { "name": { "type": "string" }, "args": { "type": "object" } } } } },
        { "type": "object", "required": ["Skill"], "properties": { "Skill": { "type": "object", "required": ["path", "body"], "properties": { "path": { "type": "string" }, "body": { "type": "string" } } } } }
      ]
    },
    "execution_mode": { "enum": ["with_agent", "no_agent"] },
    "overlap_policy": { "enum": ["allow", "skip", "queue", "replace"], "default": "skip" },
    "misfire_policy": { "enum": ["skip", "run_once", "catch_up"], "default": "run_once" },
    "max_runs": { "type": ["integer", "null"] }
  }
}
```

## E.5 flowz_cron_list
Input:
```json
{
  "type": "object",
  "properties": { "include_disabled": { "type": "boolean", "default": false } }
}
```

## E.6 flowz_cron_cancel
Input:
```json
{
  "type": "object",
  "required": ["cron_id"],
  "properties": { "cron_id": { "type": "string" } }
}
```

## E.7 flowz_subagent_delegate

Input Schema (aligned with Hermes Public Subagent Lifecycle API):
```json
{
  "type": "object",
  "required": ["action"],
  "properties": {
    "action": {
      "oneOf": [
        {
          "type": "object",
          "required": ["spawn"],
          "properties": {
            "spawn": {
              "type": "object",
              "required": ["goal"],
              "properties": {
                "goal": { "type": "string" },
                "context": { "type": "string" },
                "role": { "enum": ["leaf", "orchestrator"], "default": "leaf" },
                "correlation_id": { "type": "string" },
                "allowed_toolsets": { "type": "array", "items": { "type": "string" } },
                "timeout_seconds": { "type": "integer" }
              }
            }
          }
        },
        { "type": "object", "required": ["list"], "properties": { "list": { "const": true } } },
        { "type": "object", "required": ["steer"], "properties": { "steer": { "type": "object", "required": ["handle_id", "instruction"], "properties": { "handle_id": { "type": "string" }, "instruction": { "type": "string" } } } } },
        { "type": "object", "required": ["stop"], "properties": { "stop": { "type": "object", "required": ["handle_id"], "properties": { "handle_id": { "type": "string" }, "reason": { "type": "string" } } } } },
        { "type": "object", "required": ["wait"], "properties": { "wait": { "type": "object", "required": ["handle_id"], "properties": { "handle_id": { "type": "string" }, "timeout_seconds": { "type": "integer" } } } } },
        { "type": "object", "required": ["result"], "properties": { "result": { "type": "object", "required": ["handle_id"], "properties": { "handle_id": { "type": "string" } } } } }
      ]
    }
  }
}
```

## E.8 flowz_harness_list
Output:
```json
{
  "type": "object",
  "required": ["current", "versions"],
  "properties": {
    "current": { "type": "string" },
    "versions": { "type": "array", "items": { "type": "string" } }
  }
}
```

## E.9 flowz_harness_show
Input:
```json
{ "type": "object", "required": ["version"], "properties": { "version": { "type": "string" } } }
```

## E.10 flowz_harness_rollback
Input:
```json
{ "type": "object", "required": ["version"], "properties": { "version": { "type": "string" } } }
```

## E.11 flowz_skill_list
Output:
```json
{
  "type": "array",
  "items": {
    "type": "object",
    "required": ["name", "description", "creator"],
    "properties": {
      "name": { "type": "string" },
      "description": { "type": "string" },
      "creator": { "type": "string" },
      "created_at": { "type": "string", "format": "date-time" }
    }
  }
}
```

## E.12 flowz_evolution_run
Input:
```json
{
  "type": "object",
  "properties": {
    "budget": { "$ref": "#/$defs/EvolutionBudget" },
    "force": { "type": "boolean", "default": false }
  }
}
```

## E.13 flowz_evolution_status
Output:
```json
{
  "type": "object",
  "required": ["enabled", "last_round"],
  "properties": {
    "enabled": { "type": "boolean" },
    "last_round": {
      "type": ["object", "null"],
      "properties": {
        "round_id": { "type": "string" },
        "started_at": { "type": "string", "format": "date-time" },
        "candidates_admitted": { "type": "integer" },
        "candidates_rejected": { "type": "integer" }
      }
    },
    "evolver_tier": { "enum": ["tier1", "tier2", "tier3", "tier4"] },
    "task_tier": { "enum": ["tier1", "tier2", "tier3", "tier4"] }
  }
}
```

## E.14 McpTool Trait & Handler Signature

```rust
use async_trait::async_trait;
use serde_json::Value;
use crate::error::FlowzError;
use crate::invocation::InvocationContext;
use crate::mcp::toolsets::Toolset;

#[async_trait]
pub trait McpTool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema(&self) -> Value;
    fn output_schema(&self) -> Value;
    fn toolset(&self) -> Toolset;

    async fn call(
        &self,
        args: Value,
        ctx: &InvocationContext,
    ) -> Result<Value, FlowzError>;
}
```

## E.15 Decision Tool: `flowz_decide` (System One Fast-Path)

- Toolset: `Toolset::Supervisor`
- Name: `flowz_decide`
- Description: Evaluate fast-path typed questions using System One models with multi-tier fallback and Langfuse tracing.

### Input Schema
```json
{
  "type": "object",
  "required": ["state", "questions"],
  "properties": {
    "state": { "description": "Context and environment state payload evaluated across all questions." },
    "questions": { "type": "object", "description": "Map of named questions (choice, noul, or score)." },
    "keep_alive": { "type": "string", "description": "Optional keep-alive setting for local models (e.g. 5m)." }
  }
}
```

### Output Schema
```json
{
  "type": "object",
  "required": ["status", "answers"],
  "properties": {
    "status": { "type": "string", "enum": ["success"] },
    "answers": { "type": "object" },
    "usage": { "type": ["object", "null"] }
  }
}
```

