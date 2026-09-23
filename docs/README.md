4 ไฟล์

docs/README.md

```markdown
# Flowz Docs

| File | Contents |
|---|---|
| [decisions.csv](./decisions.csv) | ADR 0001–0037 |
| [plans.csv](./plans.csv) | task + status + verify point |
| [specs.csv](./specs.csv) | spec 7 เล่ม |
| [decisions/](./decisions/) | ADR files |
| [plans/](./plans/) | plan files |
| [specs/](./specs/) | spec files |
```

docs/decisions.csv

```csv
id,status,topic,decision,rationale
0001,✅,TimeBudget กับ Cron,แยกคนละ type/field/validation,subagent งานสั้นมี deadline — cron ปลุก client ตามเวลา
0002,✅,บังคับ Budget,ทุก item/reducer มี TimeBudget + IterationBudget,กัน agent วน loop
0003,✅,ห้าม cron ใน workflow,RunRequest ห้ามมี schedule/cron/run_at,ป้องกันปนกัน
0004,✅,Cron แยกโมดูล,src/cron/ แยกจาก workflow,ClientDispatcher ≠ WorkerSpawner
0005,✅,Cron user เท่านั้น,agent ปกติไม่มี cron tool,ผู้ใช้ควบคุมเอง
0006,✅,ตั้งชื่อ tool,prefix flowz_ ทุก tool,กัน collision MCP อื่น
0007,✅,Service layer,MCP/CLI เป็นแค่ adapter,logic อยู่ที่เดียว
0008,✅,Invocation context,source Mcp/Cli/Cron/Evolution,output/permission/log ต่างกัน
0009,✅,Typed events,struct ไม่อ่าน log,อ่าน log เปลือง + ไม่ deterministic
0010,✅,Rule ก่อน LLM,rule engine ชั้นแรก,LLM แพง/ช้า/ไม่แน่นอน
0011,✅,ขอบเขต supervisor,เห็นเฉพาะ flowz traffic,ทางเทคนิคเห็นได้แค่นั้น
0012,✅,Skills เขียนเมื่อไหร่,หลัง API freeze,ชื่อ tool เปลี่ยนได้
0013,✅,อ้างชื่อใน skill,flowz_* จริง ห้าม generic,ไม่งั้นหยิบ system cron
0014,✅,Prompts per mode,cron/subagent แยก,context window ไม่บวม
0015,✅,ลำดับ phase,15 phases ห้ามข้าม,checkpoint ก่อน
0016,✅,Context compression,two-layer 50% + micro,context window ไม่พอ
0017,✅,Toolsets,จัดกลุ่มตาม mode,โหลดเฉพาะที่ใช้
0018,✅,Live orchestration,spawn/list/steer/stop,4 action เดียว
0019,✅,Role subagent,Leaf vs Orchestrator,กัน recursive
0020,✅,Wall-clock budget,RunBudget แยก TimeBudget,งานรวมมี deadline
0021,✅,Tool register ตัวเอง,แต่ละ file register เอง,ลด coupling
0022,✅,Process registry,แยกจาก WorkerHandle,เห็น OS process จริง
0023,✅,Markdown-first,md + frontmatter,เขียน JSON มือง่ายพัง
0024,✅,Source tracking,Markdown/Api/Imported/Generated,regenerate ถูกตัว
0025,✅,Adapter layer,src/adapter/ แยก,core ไม่รู้จัก client
0026,✅,Hermes profile,git + distribution.yaml,git มี versioning แล้ว
0027,✅,Client interop,แปลง 4 client ↔ flowz,ผู้ใช้มี client ต่างกัน
0028,✅,Remote execution,ExecutionBackend trait,แยก "ทำอะไร" กับ "รันที่ไหน"
0029,✅,Zero-config,flowz init ครั้งเดียว,ผู้ใช้ไม่ต้องรู้ backend
0030,✅,Learning cron,Cron 1 ทุกวัน 03:00 misfire run_once,evolution round
0031,✅,Skill-reuse cron,Cron 2 success → SKILL.md,เก็บกระบวนการดี
0032,✅,Model tier,Evolver ≥ Task + 1,self-eval bias
0033,✅,Gene bank,cell=(component,pathology),ไม่ collapse
0034,✅,4 gates,Validity→Activation→Significance→Gain,กรองตามลำดับ
0035,✅,Statistical rigor,z≥1.96 n≥26,กัน admit noise
0036,✅,Misfire run_once,last_run_at + startup,ไม่ thundering herd
0037,✅,Rollback,SemVer + content_hash + 1 cmd,ย้อนได้ทุก patch
```

docs/plans.csv

```csv
id,status,task,path,done_when,verify_at,adr
H1,⬜,Harness types,src/harness/patch.rs,serialize + hash deterministic,src/harness/patch.rs::test_hash,0030 0033 0037
H2,⬜,Trajectory types,src/trajectory/model.rs,JSONL roundtrip,src/trajectory/model.rs::test_roundtrip,0030
H3,⬜,Skill types,src/skill/model.rs,parse SKILL.md,src/skill/model.rs::test_parse,0031
H4,⬜,Discovery types,src/discovery/model.rs,serialize,src/discovery/model.rs::test_serialize,0029
H5,⬜,Service traits,src/service/,mock test,src/service/mod.rs::test_traits,0007
H6,⬜,TrajectoryCollector,src/trajectory/collector.rs,event→step ครบ,src/trajectory/collector.rs::test_mapping,0030
H7,⬜,Cron 1 learning,src/cron/learning.rs,4 gates + misfire,src/cron/learning.rs::test_round,0030 0036
H8,⬜,Cron 2 skill-reuse,src/cron/skill_reuse.rs,creator=flowz + ≥3 sessions,src/cron/skill_reuse.rs::test_synth,0031
H9,⬜,4 gates,src/harness/gates.rs,เรียงลำดับ + z-test,src/harness/gates.rs::test_sequential,0034 0035
H10,⬜,PairedTest,src/harness/stats.rs,z=5.0 ที่ mean=.5 std=1 n=100,src/harness/stats.rs::test_z,0035
H11,⬜,GeneBank,src/harness/gene_bank.rs,z สูงสุดชนะ,src/harness/gene_bank.rs::test_competitive,0033
H12,⬜,VersionStore,src/harness/version_store.rs,rollback ได้,src/harness/version_store.rs::test_rollback,0037
H13,⬜,Evolver backend,src/evolution/,tier check,src/evolution/mod.rs::test_tier,0032
H14,⬜,Skill synthesis,src/evolution/skill_synth.rs,เขียน ~/.flowz/skills/,src/evolution/skill_synth.rs::test_write,0031
H15,⬜,Profile format,src/profile/,install→update ไม่แตะ user,src/profile/mod.rs::test_ownership,0026
H16,⬜,Adapters,src/adapter/,roundtrip + ไม่มี credential,src/adapter/mod.rs::test_roundtrip,0027
H17,⬜,Backends,src/execution/,dispatch→collect,src/execution/mod.rs::test_dispatch,0028
H18,⬜,Discovery,src/discovery/,flowz init ไม่ถาม,src/discovery/mod.rs::test_init,0029
H19,⬜,flowz doctor,src/cli/doctor.rs,check backend + tier,src/cli/doctor.rs::test_doctor,0029
H20,⬜,harness_* skill_* evolution_* tools,src/mcp/tools/,register + schema,src/mcp/tools/mod.rs::test_register,0006 0031 0033
H21,⬜,Fixtures,tests/fixtures/,coverage ≥ 80%,tests/mod.rs::test_coverage,0015
H22,⬜,Skills,skills/,flowz_* จริง + negative example,skills/_test,0012 0013
OB1,⬜,TimeBudget,src/workflow/time_budget.rs,reject 0 + heartbeat ≥ max,src/workflow/time_budget.rs::test,0002
OB2,⬜,IterationBudget,src/workflow/iteration_budget.rs,reject 0 + pressure 0-1,src/workflow/iteration_budget.rs::test,0002
OB3,⬜,RunBudget,src/workflow/run_budget.rs,serialize,src/workflow/run_budget.rs::test,0020
OB4,⬜,WorkerEvent JSONL,src/worker/protocol.rs,parse ทุก variant,src/worker/protocol.rs::test,0009
OB5,⬜,WorkerSupervisor,src/worker/supervisor.rs,timeout + grace + kill,src/worker/supervisor.rs::test,0002
OB6,⬜,EventBus,src/supervisor/bus.rs,broadcast + lagged,src/supervisor/bus.rs::test,0009
OB7,⬜,Rules 7 ข้อ,src/supervisor/rules.rs,ทุก rule pass,src/supervisor/rules.rs::test,0010
OB8,⬜,Finding,src/supervisor/findings.rs,severity + evidence,src/supervisor/findings.rs::test,0009
OB9,⬜,ProcessRegistry,src/supervisor/process_registry.rs,list + poll + kill,src/supervisor/process_registry.rs::test,0022
OB10,⬜,PreCompact hook,src/workflow/compression.rs,dump state,src/workflow/compression.rs::test,0016
MF1,✅,Frontmatter parser,src/adapter/markdown/frontmatter.rs,parse YAML + body,src/adapter/markdown/frontmatter.rs::test,0023
MF2,✅,Wikilink extractor,src/adapter/markdown/links.rs,extract 3 ชนิด,src/adapter/markdown/links.rs::test,0023
MF3,✅,MarkdownCompiler,src/adapter/markdown/compiler.rs,merge defaults,src/adapter/markdown/compiler.rs::test,0024
MF4,✅,MarkdownRegistry,src/adapter/markdown/registry.rs,recursive + ignore,src/adapter/markdown/registry.rs::test,0024
MF5,✅,ADR 0023-0025,docs/decisions/,3 ไฟล์มี,docs/decisions/,0023 0024 0025
```

docs/specs.csv

```csv
id,status,name,scope,path,adr
data-dict,📝,Data Dictionary,type ทั้งหมด — Harness/Trajectory/Skill/Evolution/Discovery,docs/specs/data-dictionary.md,0009 0023 0029 0030 0031 0033
algo,📝,Algorithms,Cron 1 + Cron 2 + 4 gates + discovery + backend select,docs/specs/algorithms.md,0030 0031 0034 0029 0028
storage,📝,Storage,harness/versions + bank + skill + trajectory + cron.db + jobs.db,docs/specs/storage.md,0023 0024 0030 0031
tools,📝,Tool Schemas,input/output/error ทุก tool,docs/specs/tools.md,0006 0017
services,📝,Service Contracts,trait signature ทุก service,docs/specs/services.md,0007 0008
integration,📝,Integration,pmcp + Modal + Daytona + Claude/Codex/AGY/Hermes,docs/specs/integration.md,0026 0027 0028
tests,📝,Test Vectors,fixture + expected output + CI pipeline,docs/specs/tests.md,0015 0035
```