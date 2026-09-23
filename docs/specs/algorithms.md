# Specification: Algorithms

## 1. Cron 1: Learning Cron Algorithm (03:00 daily, misfire: run_once)
1. Collect recent trajectories (window 24h, min 26)
2. Diagnose via Evolver Backend (check evolver_tier >= task_tier + 1)
3. Group by (affected_component, pathology)
4. Generate candidates (Reinvented + Recombined from compatible gene bank cells)
5. Screen through 4 Gates sequentially:
   - Gate 1: Validity (Protocol-valid, complete ledger, retry sandbox crash)
   - Gate 2: Activation (Deterministic beacon triggered)
   - Gate 3: Significance (Paired z-test: z >= 1.96, n >= 26)
   - Gate 4: Gain (delta > 0.0)
6. Admit to Harness Gene Bank and bump SemVer version
7. Report EvolutionRoundResult

## 2. Cron 2: Skill-Reuse Cron Algorithm (04:00 daily, misfire: run_once)
1. Collect Successes (user_confirmed = true, >= 3 sessions)
2. Verify Value (consistency score >= 0.7, activation verified)
3. Synthesize Skill via Evolver Backend (enforce creator = "flowz")
4. Check Existing (if better quality, bump skill version)
5. Write to ~/.flowz/skills/<name>/SKILL.md
6. Register in SkillService

## 3. Paired Z-Test Formula
- z_score = mean_delta / (std_dev / sqrt(sample_size))
- Significant when z_score >= 1.96 and sample_size >= 26

## 4. Discovery Algorithm
- Scans local hardware, installed CLI agents (Claude, Codex, Hermes, AGY), environment credentials, docker
- Resolves evolver_tier = max(cli_agents.model_tier)
- Selects execution backend based on preference, local capacity, and cost/latency weights

## 5. Client Adapter Mapping
- Claude: settings.json -> config.yaml, CLAUDE.md -> SOUL.md, agents/*.md -> agents/, skills/*.md -> skills/
- Credentials strictly excluded (.env.EXAMPLE generated instead)
