# Guardian Python SDK

Python bindings for the Guardian AI code quality library.

## Installation

```bash
pip install guardian
```

Or from source:

```bash
cd sdks/python/guardian
maturin develop --release
```

## Usage

```python
from guardian import Guardian, GuardianConfig

# Default config
g = Guardian()

# Custom config
config = GuardianConfig(
    db_path="/custom/path/metrics.db",
    slop_threshold=15,
    loop_threshold=5
)
g = Guardian(config)

# Analyze a file
result = g.analyze_file("src/main.rs")
print(f"Quality: {result.grade} ({result.quality_score:.1f})")
print(f"Slop issues: {len(result.slop_issues)}")

# Analyze multiple files
results = g.analyze_files(["src/main.rs", "src/lib.rs", "src/utils.rs"])

# Import and analyze command log
cmd_report = g.import_and_analyze_commands("~/.guardian/command.log")
print(f"Scanned: {cmd_report.scanned} commands")
for finding in cmd_report.findings:
    print(f"  [{finding.kind}] {finding.message}")
```

## Types

- `GuardianConfig` - Configuration options
- `FileAnalysis` - Complete analysis result
- `SlopIssue` - Individual slop finding
- `YAGNIReport` - Dead code / over-engineering report
- `OverEngineeringReport` - Over-engineering findings with net deletable lines
- `PonytailCommentReport` - Deliberate simplifications marked with `# ponytail:`
- `MinimalCheckReport` - Test coverage for non-trivial functions
- `CommandLogReport` - Shell command pattern analysis
- `CommandFinding` - Individual command pattern finding