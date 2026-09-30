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

export TEV1_BASE_URL="http://<reachable-ollama-host>:11434"
export TEV1_MODEL="tev1:4b" # or tev1:0.8b

cargo run -p decider --example capability_probe
```

The Jev endpoint must be reachable from the process running the probe. Laya and Ollama URLs must likewise be reachable from that process; `localhost` means the current machine/container. The probe only sends a synthetic support-ticket example. The caller must sanitize real state before sending it to a remote provider.

Unset backends are reported as `SKIP`. A configured backend passes only when the response contains a choice, a Noul probability, and a numeric score. HTTP failures report the status code only; keys and response bodies are not printed.

Ollama's official model documentation lists Choice, Noul, and Score for tev1 and specifies Ollama 0.35 or newer. That documentation is not a substitute for running this live probe against the exact model and server version used in deployment.

## Offline tests

```bash
cargo test -p decider
cargo clippy -p decider --all-targets -- -D warnings
```

The integration tests use a local mock HTTP server and do not require provider credentials or a model download.
