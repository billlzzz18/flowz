#!/bin/sh
set -eu
cargo build --release
mkdir -p .guardian/metrics .guardian/patterns .guardian/baseline .guardian/presets
cp presets/*.json .guardian/presets/ 2>/dev/null || true
if [ -d .git/hooks ]; then echo "Git repository detected; run guardian hook installation from the CLI."; fi
cat > .guardian/config.json <<'JSON'
{"enabled":true,"detectors":{"slop":true,"yagni":true,"frontend":true,"api_security":true,"ai_behavior":true},"git_ai":{"enabled":true,"track_authors":true,"track_models":true},"lsp":{"enabled":true,"analyze_on_save":true,"show_ai_indicators":true},"thresholds":{"slop_score":30.0,"hallucination_risk":20.0,"panic_loops":0,"quality_score":70.0}}
JSON
echo "AI Quality Guardian setup complete"
