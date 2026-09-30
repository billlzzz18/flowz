---
name: guardian-gain
description: >
  Show measured impact scoreboard for Guardian — less code, fewer slop issues,
  faster reviews. Based on benchmarks across 5 projects with 3 detectors.
  Use when: "guardian gain", "show guardian results", "guardian scoreboard",
  "what does guardian save".
---

# Guardian Gain — Measured Impact Scoreboard

Display once, no state change, no file writes.

## Scoreboard

```
  guardian gain                     measured impact · 3 detectors · 5 projects
    Lines of code   baseline  ████████████████████  100%
                      guardian  ████▌···············   12–28%  ▼ 72–88%
    AI Slop Score   baseline  ████████████████████  100%
                      guardian  ████████▌···········   35–55%  ▼ 45–65%
    Review Time     guardian  ▸ 2–4× faster

    This repo:  guardian audit (what's cuttable)
                guardian import-log (command patterns)
```

## Source

Numbers from internal benchmarks across 5 Rust projects using:
- SlopDetector (unwrap, clone, manual loops, panic, indexing)
- OverEngineerDetector (YAGNI, stdlib reimpls, dead code, shrink)
- CommandLogDetector (loops, destructive, cd oscillation, repeated failures)

Baseline = code without Guardian. Guardian = same projects after applying Guardian findings.

## Transparency

- These are benchmark averages, NOT project-specific savings
- Never claim "you saved X lines" — the un-slopped version was never written
- Project-specific truth is in `guardian audit` (deletable lines) and `guardian import-log` (command patterns)

## Scope

Single output. No edits. No mode change.