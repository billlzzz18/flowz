# Specification: Algorithms

Source: Flowz Implementation Specification (Part F)

## F.1 Cron 1: Learning Cron Algorithm

```
INPUT: window (default 24h)
OUTPUT: EvolutionRoundResult

1. COLLECT
   trajectories = trajectory_service.query_recent(window)
   if trajectories.is_empty() or trajectories.len() < MIN_TRAJECTORIES(=26):
       log "insufficient data"
       return Skipped

2. DIAGNOSE
   evolver_backend = select_evolver_backend()
   if evolver_backend.tier < task_tier + 1:
       log "evolver tier too low"
       return Skipped
   
   diagnosis = await evolver_backend.diagnose(
       trajectories=trajectories,
       current_harness=harness_service.current_version()
   )
   // diagnosis = Vec<(TrajectoryId, PathologyType)>

3. GROUP
   groups = group_by(trajectories, key=trajectory => (affected_component, pathology))
   // affected_component = evolver-determined
   // pathology = diagnosed

4. GENERATE
   candidates = []
   for (cell_key, group) in groups:
       // Reinvented
       reinvented = await evolver_backend.evolve(
           trajectories=group.trajectories,
           pathology_hypothesis=cell_key.pathology,
           current_harness=harness
       )
       candidates.push(reinvented)

       // Recombined (จาก gene bank cell อื่น)
       compatible = gene_bank.find_compatible(cell_key)
       if compatible.is_some():
           recombined = await evolver_backend.recombine(
               base=harness,
               patterns=compatible,
               pathology=cell_key.pathology
           )
           candidates.push(recombined)

5. SCREEN
   for candidate in candidates:
       // Gate 1: Validity
       validity = await validity_gate(candidate)
       if validity == RepairAndRetry:
           candidate = await repair(candidate)
           retry validity
       if validity != Pass:
           results.push(Rejected{reason: "validity"})
           continue

       // Gate 2: Activation
       activation = await activation_gate(candidate)
       if activation != Pass:
           results.push(RejectedInert)
           continue

       // Gate 3: Significance
       sig = await significance_gate(candidate, parent=harness)
       if sig != Pass:
           results.push(sig)
           continue

       // Gate 4: Gain
       gain = await gain_gate(candidate, parent=harness)
       if gain != Pass:
           results.push(gain)
           continue

       // All passed → Admitted
       results.push(Admitted{
           delta=sig.delta,
           z_score=sig.z_score,
           sample_size=sig.sample_size
       })

6. ADMIT
   for admitted in results.filter(Admitted):
       version = bump_version(harness)
       harness_version_store.save(version)
       gene_bank.admit(
           patch=admitted.patch,
           preserved=PreservedHarness{
               harness=version,
               parent=harness.version,
               z_score=admitted.z_score,
               sample_size=admitted.sample_size,
               verified_at=now(),
               source=Generated{by: LearningCron}
           }
       )

7. REPORT
   return EvolutionRoundResult{
       round_id=uuid(),
       candidates_generated=candidates.len(),
       candidates_admitted=results.filter(Admitted).len(),
       candidates_rejected=results.len() - admitted.len(),
       screening_details=results,
       duration_ms=elapsed,
       cost=sum(costs)
   }
```

## F.2 Cron 2: Skill-Reuse Algorithm

```
INPUT: min_sessions (default 3), window (default 7d)
OUTPUT: Vec<SkillDefinition>

1. COLLECT SUCCESSES
   groups = trajectory_service.query_successful(
       min_sessions=min_sessions,
       window=window
   )
   // group criteria:
   //   - all trajectories: outcome = Success{user_confirmed=true}
   //   - all trajectories: harness_version = current
   //   - group size >= min_sessions

2. VERIFY VALUE (per group)
   for group in groups:
       // Consistency check
       if group.consistency_score < 0.7:
           skip group
       
       // Activation check
       if !activation_gate_for_skill(group):
           skip group

3. SYNTHESIZE SKILL
   for group in verified_groups:
       evolver_backend = select_evolver_backend()
       skill = await evolver_backend.synthesize_skill(
           SkillSynthesisRequest{
               trajectories=group.trajectories,
               intent=group.intent_signature,
               success_criteria=group.success_criteria
           }
       )
       
       // Enforce creator
       skill.frontmatter.creator = "flowz"
       skill.frontmatter.source_trajectory = group.trajectories[0].id
       skill.frontmatter.provenance = SkillProvenance{
           source_trajectory_ids: group.trajectories.map(|t| t.id),
           harness_version_at_creation: current_harness.version,
           created_by_cron: SKILL_REUSE_CRON_ID,
           first_used_at: None,
           usage_count: 0
       }

4. CHECK EXISTING
   existing = skill_service.find_by_name(skill.name)
   if existing.is_some():
       if existing.quality_score >= skill.quality_score:
           skip
       else:
           bump_skill_version(skill)

5. WRITE
   path = ~/.flowz/skills/<skill.name>/SKILL.md
   write_skill_markdown(path, skill)

6. REGISTER
   skill_service.register(skill)

7. RETURN
   return created_skills
```

## F.3 Gate 1: Validity

```rust
async fn validity_gate(
    candidate: &CandidatePatch,
    ctx: &EvaluationContext,
) -> GateResult {
    // 1. Protocol check — patch structure valid?
    if !candidate.patch.is_protocol_valid() {
        return Reject { reason: "invalid patch structure" };
    }

    // 2. Ledger completeness — trace complete?
    let ledger = ctx.collect_ledger(candidate).await;
    if !ledger.is_complete() {
        return Reject { reason: "incomplete ledger" };
    }

    // 3. Infrastructure check — sandbox crashed?
    if ledger.has_sandbox_crash() {
        return RepairAndRetry { reason: "sandbox crash" };
    }

    if ledger.has_verifier_timeout() {
        return RepairAndRetry { reason: "verifier timeout" };
    }

    Pass
}
```

## F.4 Gate 2: Activation

```rust
async fn activation_gate(
    candidate: &CandidatePatch,
    ctx: &EvaluationContext,
) -> GateResult {
    // Deterministic beacon: patch ที่ execute จริงจะ trigger marker
    let beacon = candidate.patch.activation_beacon();
    let traces = ctx.run_eval_trajectories(candidate).await;
    let beacon_triggered = traces.iter().any(|t| t.contains_beacon(&beacon));

    if !beacon_triggered {
        return Reject { reason: "inert — beacon not triggered" };
    }

    Pass
}
```

## F.5 Gate 3: Significance

```rust
async fn significance_gate(
    candidate: &CandidatePatch,
    parent: &HarnessVersion,
    ctx: &EvaluationContext,
) -> GateResult {
    let eval_tasks = ctx.eval_tasks();  // >= 26 tasks
    if eval_tasks.len() < 26 {
        return Reject { reason: "insufficient eval tasks" };
    }

    let mut deltas = Vec::new();
    for task in eval_tasks {
        let cand_score = ctx.evaluate(&candidate.patch, task).await;
        let parent_score = ctx.evaluate(parent, task).await;
        deltas.push(cand_score - parent_score);
    }

    let test = PairedTest {
        sample_size: deltas.len() as u64,
        mean_delta: mean(&deltas),
        std_dev: std_dev(&deltas),
    };

    if test.z_score() < 1.96 {
        return Reject {
            reason: format!("not significant: z={}", test.z_score())
        };
    }

    Pass
}
```

## F.6 Gate 4: Gain

```rust
async fn gain_gate(
    candidate: &CandidatePatch,
    parent: &HarnessVersion,
    ctx: &EvaluationContext,
) -> GateResult {
    let delta = ctx.compute_mean_delta(&candidate.patch, parent).await;
    if delta <= 0.0 {
        return Reject { reason: format!("no gain: delta={}", delta) };
    }
    Pass
}
```

## F.7 Discovery Algorithm

```rust
impl DiscoveryEngine {
    pub async fn scan() -> Result<DiscoveryResult, FlowzError> {
        let local = Self::probe_local().await?;
        let cli_agents = Self::scan_cli_agents().await;
        let providers = Self::scan_providers().await;
        let credentials = Self::scan_credentials().await;
        let docker = Self::check_docker().await;
        let evolver_tier = Self::detect_evolver_tier(&cli_agents);

        Ok(DiscoveryResult {
            local, cli_agents, providers, credentials, docker,
            evolver_tier, scanned_at: Utc::now(),
        })
    }

    async fn probe_local() -> Result<LocalCapabilities, FlowzError> {
        let sys = sysinfo::System::new_all();
        Ok(LocalCapabilities {
            cpu_cores: sys.cpus().len() as u32,
            ram_gb: sys.total_memory() as f64 / 1_073_741_824.0,
            gpu: Self::probe_gpu(),
            disk_gb: Self::probe_disk(),
        })
    }

    async fn scan_cli_agents() -> Vec<DetectedCliAgent> {
        let candidates = [
            ("claude", "~/.claude/"),
            ("codex", "~/.codex/"),
            ("hermes", "~/.hermes/"),
            ("agy", "~/.gemini/"),
        ];
        let mut found = Vec::new();
        for (name, config_dir) in candidates {
            if let Some(path) = which::which(name).ok() {
                found.push(DetectedCliAgent {
                    name: name.into(),
                    path,
                    version: Self::probe_version(name).await,
                    subscription_tier: Self::probe_tier(name).await,
                    model_tier: None,
                });
            }
        }
        found
    }

    async fn scan_credentials() -> Vec<CredentialSource> {
        let mut creds = Vec::new();
        for var in ["MODAL_TOKEN_ID", "DAYTONA_API_KEY", "FLY_API_TOKEN", "E2B_API_KEY"] {
            if std::env::var(var).is_ok() {
                creds.push(CredentialSource::Env { var: var.into() });
            }
        }
        for service in ["modal", "daytona", "fly", "e2b"] {
            if keyring::Entry::new(service, "default").get_password().is_ok() {
                creds.push(CredentialSource::Keychain {
                    service: service.into(),
                    account: "default".into(),
                });
            }
        }
        creds
    }

    async fn check_docker() -> Option<DockerInfo> {
        let output = Command::new("docker").arg("version").output().await.ok()?;
        if !output.status.success() { return None; }
        let version = String::from_utf8(output.stdout).ok()?;
        Some(DockerInfo {
            version: version.lines().next()?.to_string(),
            gpu_runtime: Self::has_nvidia_runtime().await,
        })
    }

    fn detect_evolver_tier(agents: &[DetectedCliAgent]) -> Option<ModelTier> {
        agents.iter()
            .filter_map(|a| a.model_tier)
            .max()
    }
}
```

## F.8 Backend Selection Algorithm

```rust
impl BackendSelector {
    pub fn select(&self, spec: &WorkerSpec) -> Result<BackendId, FlowzError> {
        if let Some(pref) = spec.backend_preference.first() {
            if let Some(backend) = self.find_by_id(pref) {
                if backend.supports(spec) {
                    return Ok(backend.id());
                }
            }
        }

        let required = spec.resource_requirements();
        if self.local_resources.can_run(&required) {
            return Ok(BackendId("local"));
        }

        let mut candidates: Vec<_> = self.backends.iter()
            .filter(|b| b.supports(spec))
            .collect();

        candidates.sort_by_key(|b| {
            let cost = b.cost_estimate(spec).map(|c| c.usd).unwrap_or(f64::MAX);
            let latency = self.estimate_latency(b);
            match self.preferences.cost_preference {
                CostPreference::Cheap => cost as i64,
                CostPreference::Balanced => (cost + latency * 0.1) as i64,
                CostPreference::Fast => latency as i64,
            }
        });

        candidates.first()
            .map(|b| Ok(b.id()))
            .unwrap_or(Err(FlowzError::Backend(BackendError::Unavailable(
                "no backend matches requirements".into()
            ))))
    }
}
```

## F.9 Adapter Mapping Algorithm (Claude)

```rust
impl ImportAdapter for ClaudeAdapter {
    fn can_import(&self, path: &Path) -> bool {
        path.join("settings.json").exists()
    }

    fn import(&self, path: &Path) -> Result<FlowzProfile, AdapterError> {
        let mut profile = FlowzProfile::default();

        let settings: serde_json::Value = read_json(&path.join("settings.json"))?;
        profile.config.model = settings["model"].as_str().map(String::from);
        profile.config.permissions = settings["permissions"].clone();
        profile.config.hooks = settings["hooks"].clone();

        if let Ok(soul) = read_string(&path.join("CLAUDE.md")) {
            profile.soul = Some(soul);
        }

        for entry in read_dir(path.join("agents"))? {
            let md = parse_subagent_markdown(&entry)?;
            profile.agents.push(Agent {
                name: md.frontmatter.name,
                description: md.frontmatter.description,
                model: md.frontmatter.model,
                tools: md.frontmatter.tools,
                prompt: md.body,
            });
        }

        for entry in read_dir(path.join("skills"))? {
            let md = parse_skill_markdown(&entry)?;
            profile.skills.push(Skill {
                name: md.frontmatter.name,
                description: md.frontmatter.description,
                body: md.body,
                frontmatter: md.frontmatter,
            });
        }

        // Credentials — NEVER imported
        Ok(profile)
    }
}
```

## F.10 Hermes-Learn Plugin Pattern (Self-Contained Subagent Skill Extraction)

Source: `~/.hermes/plugins/hermes-learn/` (Architecture Reference for Flowz Evolution / Skill Synthesis)

### Architecture
- **Storage Layer (`skills.py`):** Standalone stdlib-only module (`pathlib`, `shutil`, `re`). Manages SKILL.md discover, list, view, create, patch, delete, write_file, remove_file without external dependencies.
- **Dispatch Tools (`__init__.py`):** Registers `skills_list`, `skill_view`, `skill_manage` via `ctx.register_tool(name, toolset="skills", schema, handler, override=True)`.
- **Slash Command (`/learn`):** Uses Hermes `subagent_lifecycle.launch(SubagentLaunchRequest(...))` to spawn an isolated leaf subagent.
- **Prompt Isolation:** The synthesis prompt is passed directly as the subagent's `goal` with scoped toolsets (`allowed_toolsets=("file", "web", "skills")`), decoupling parent context from raw extraction noise.

### Mapping to Flowz Evolution Layer (ADR-0030, ADR-0031)
1. **Subagent Spawning:** Flowz uses the exact same pattern for Evolver Agent execution:
   ```rust
   // Flowz equivalent in src/service/evolution.rs / src/cron/skill_reuse.rs
   let req = SubagentLaunchRequest {
       goal: build_skill_synthesis_goal(&trajectories),
       role: SubagentRole::Leaf,
       allowed_toolsets: vec!["file", "skills"],
       effort_level: EffortLevel::High,
   };
   ```
2. **Skill Synthesis Criteria:**
   - Frontmatter enforces: `name`, `description` (<=60 chars), `creator: "flowz"`, `source_trajectory`, `harness_version`, `verification`, `provenance`.
   - File target: `~/.flowz/skills/<name>/SKILL.md` (or `$HERMES_HOME/skills/<cat>/<name>/SKILL.md`).
3. **Execution Fallback & Error Handling:**
   - Plugin handles API signature divergence (`TypeError` fallback on tool registration).
   - Subagent result retrieval falls back across `text`, `output`, or `str(result)`.

## F.11 Dual-Track Evolution: Live Concurrent Observer & Background Evolver

Source: Architectural Decision on Self-Evolution Runtime Strategy

### Architectural Principle
Self-Evolution (Evo) **ไม่ได้จำกัดอยู่แค่การเรียกผ่านคำสั่ง /learn หรือ cron job ครั้งคราว** แต่ทำงานในรูปแบบ **Dual-Track Runtime**:
1. **Live Concurrent Observer (รันควบคู่ Agent หลัก):**
   - ในขณะที่ Primary Task Agent กำลังทำงาน Live Observer (Evo Tracker) จะดักจับ Trajectory Steps, Tool Calls, Errors และ Decision Points แบบ non-blocking เบื้องหลัง
   - ไม่รบกวน Context Window ของ Agent หลัก
2. **Background Evolver Loop (ประมวลผลการเรียนรู้เบื้องหลัง):**
   - นำพรอมป์ต์และ Goal Template จาก `hermes-learn` มาใช้เป็น **Background Worker Goal**
   - เมื่อตรวจพบรูปแบบความล้มเหลว (Pathology) หรือความสำเร็จซ้ำๆ (Success Pattern $ge 3$) ระบบจะ Spawn Subagent ระดับ Evolver (Tier $ge$ Task Tier + 1) ในโหมด Isolated Leaf เพื่อทำการ:
     - วินิจฉัยข้อผิดพลาด (Diagnose)
     - สร้าง Patch หรือสังเคราะห์ Skill ใหม่ (Synthesize)
     - ผ่าน 4 Gates (Validity $	o$ Activation $	o$ Significance $	o$ Gain)
   - ไม่ต้องรอให้ผู้ใช้สั่ง `/learn` เอง แต่รันแบบ Autonomous Evolution ใน Background ทันที

```
┌────────────────────────────────────────────────────────┐
│                   Flowz Runtime Engine                 │
│                                                        │
│  [ Primary Task Agent ] (User-facing Session)          │
│            │                                           │
│            ▼ (Emits execution steps / telemetry)       │
│  [ Live Concurrent Observer ] (Non-blocking Tap)       │
│            │                                           │
│            ▼ (Buffers Trajectory JSONL)                │
│  [ Background Evolver Daemon ]                         │
│            │                                           │
│            ├─► Trigger: Pathology or Recurring Success │
│            │                                           │
│            ▼                                           │
│  [ Evolver Subagent (Leaf) ]                           │
│     • Goal: Injected Learner/Synthesizer Prompt        │
│     • Toolsets: ("file", "web", "skills")              │
│     • Tier: Task Tier + 1                              │
│     • Output: Candidate Patch or SKILL.md              │
│            │                                           │
│            ▼                                           │
│  [ 4-Gate Statistical Verification (z >= 1.96) ]        │
│            │                                           │
│            ▼                                           │
│  [ Admit to Gene Bank / ~/.flowz/skills/ ]             │
└────────────────────────────────────────────────────────┘
```
