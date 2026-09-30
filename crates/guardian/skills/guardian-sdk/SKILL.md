---
name: guardian-sdk
description: >
  Use Guardian as a library in other projects. Covers Rust crate API,
  Python SDK (pyo3), and TypeScript SDK. Shows imports, basic usage,
  and common patterns.
  Use when: "how to use guardian as library", "guardian sdk", "guardian python",
  "guardian typescript", "embed guardian", "use guardian in my project".
---

# Guardian SDK — Embed in Your Projects

Guardian is designed as a reusable library with three SDKs.

## Rust Crate (Primary)

```toml
[dependencies]
guardian = { git = "https://github.com/bl1nk-bot/guardian", features = ["cli"] }
```

```rust
use guardian::{Guardian, GuardianConfig, FileAnalysis};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let g = Guardian::default();

    // Single file
    let analysis = g.analyze_file("src/main.rs")?;
    println!("Grade: {}", analysis.grade);

    // Multiple files
    let files = vec!["src/main.rs", "src/lib.rs"];
    let results = g.analyze_files(&files);

    // Command log analysis
    let cmd_report = g.import_and_analyze_commands("~/.guardian/command.log")?;
    for f in &cmd_report.findings {
        println!("[{:?}] {}", f.kind, f.message);
    }

    Ok(())
}
```

### Key Types

| Type | Purpose |
|------|---------|
| `Guardian` | Main API entry point |
| `GuardianConfig` | DB path, thresholds |
| `FileAnalysis` | Full analysis result |
| `SlopIssue` | AI slop patterns (unwrap, clone, etc) |
| `OverEngineeringReport` | YAGNI, stdlib, shrink findings |
| `YAGNIReport` | Unused functions/variables/imports |
| `PonytailCommentReport` | `# ponytail:` markers |
| `MinimalCheckReport` | Test coverage for non-trivial logic |
| `MetricsDb` | Direct SQLite access |

## Python SDK

```bash
pip install guardian
```

```python
from guardian import Guardian, GuardianConfig

g = Guardian()

# Analyze
result = g.analyze_file("src/main.rs")
print(f"Quality: {result.grade} ({result.quality_score:.1f})")

# Command patterns
cmd = g.import_and_analyze_commands("~/.guardian/command.log")
for f in cmd.findings:
    print(f"[{f.kind}] {f.message}")
```

### Install from Source

```bash
cd sdks/python/guardian
maturin develop --release
```

## TypeScript SDK

```bash
npm install @guardian/guardian
```

```typescript
import Guardian, { GuardianConfig, FileAnalysis } from '@guardian/guardian';

const g = new Guardian({ slopThreshold: 15 });

// Note: TypeScript SDK wraps CLI or uses FFI
// Full functionality via Rust crate or Python SDK
```

### Build

```bash
cd sdks/typescript/guardian
npm install
npm run build
```

## Common Patterns

### CI Integration (Rust)

```rust
// In your CI pipeline
fn check_quality() {
    let g = Guardian::default();
    let results = g.analyze_files(&["src/**/*.rs"]);
    for r in results {
        let a = r.unwrap();
        if a.quality_score < 70.0 {
            eprintln!("FAIL: {} scored {:.1}", a.file, a.quality_score);
            std::process::exit(1);
        }
    }
}
```

### Editor Integration (Python)

```python
# LSP-style quick check
def quick_check(file: str) -> dict:
    g = Guardian()
    r = g.analyze_file(file)
    return {
        "score": r.quality_score,
        "grade": r.grade,
        "issues": len(r.slop_issues),
        "over_eng": len(r.over_engineering.findings),
    }
```

### Pre-commit Hook (any language)

```bash
#!/bin/sh
# .git/hooks/pre-commit
files=$(git diff --cached --name-only -- '*.rs' '*.py' '*.ts')
for f in $files; do
    guardian analyze "$f" || exit 1
done
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `cli` | Build CLI binary (default) |
| `python` | Enable pyo3 Python bindings |
| `python-abi3` | Stable ABI for Python wheels |

## Versioning

Guardian follows semver. Breaking changes only on major version.
Use `guardian --version` or `GuardianConfig::VERSION` in code.

## Support

- Issues: GitHub repository
- API docs: `cargo doc --open` / `pdoc guardian`
- TypeScript types: `npm run build && cat dist/index.d.ts`