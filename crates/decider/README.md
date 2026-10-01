# `decider`

Small Rust client for System One-compatible decision endpoints (`/v1/systemone`). It sends typed `Choice`, `Noul` (yes/no), and `Score` questions and returns the provider's probability/confidence fields without making application control-flow decisions.

## Live capability probe

The `capability_probe` example submits the same synthetic state and three question types to each configured endpoint. It checks that the response contains the expected typed answer fields; it does not measure model accuracy or calibration.

```bash
export JEV_BASE_URL="https://api.typesafe.ai"
export JEV_MODEL="<model from your TypeSafe account>"
export JEV_API_KEY="<supply through your secret manager, not source control>"

export LAYA_BASE_URL="http://<reachable-laya-host>:<port>"
export LAYA_MODEL="<multilingual-laya-model>"
export LAYA_API_KEY="<optional key from your secret manager>"

export TEV1_BASE_URL="http://<reachable-ollama-host>:11434"
export TEV1_MODEL="tev1:4b" # or tev1:0.8b
export TEV1_API_KEY="<optional key from your secret manager>"

cargo run -p decider --example capability_probe
```

The Jev endpoint must be reachable from the process running the probe. Laya and Ollama URLs must likewise be reachable from that process; `localhost` means the current machine/container. The probe only sends a synthetic support-ticket example. The caller must sanitize real state before sending it to a remote provider.

Unset backends are reported as `SKIP`. A configured backend passes only when the response contains a choice, a Noul probability, and a numeric score. The output uses the configured backend label. HTTP failures report the status code only; keys and response bodies are not printed.

The probe sends the same `Choice`, `Noul`, and `Score` questions to each configured target. The Choice criteria deliberately include both `null` and a structured JSON description; endpoint acceptance is not assumed. Support can vary by provider, model, and server version, so use the probe against the exact deployment target rather than inferring compatibility from a model name. This crate does not download or load models and does not make payments; those steps are managed by the operator.

## Offline tests

```bash
cargo test -p decider
cargo clippy -p decider --all-targets -- -D warnings
```

The integration tests use a local mock HTTP server and do not require provider credentials or a model download.
