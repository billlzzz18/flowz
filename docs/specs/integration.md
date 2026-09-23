# Specification: External Integration

## 1. pmcp Integration
- Server name "flowz", standard stdio transport, instructions prefix, self-registering registry

## 2. Remote Backends
- Modal: POST /v1/sandboxes, token headers, callback webhooks, heartbeat endpoint
- Daytona: POST /v1/workspaces, bearer token, WebSocket event streaming for heartbeat

## 3. Client Adapters
- Claude Code (~/.claude/)
- OpenAI Codex (~/.codex/config.toml)
- Google Antigravity (~/.gemini/config/)
- Hermes Agent (~/.hermes/profiles/)
