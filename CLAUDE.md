# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

FastClaw is a high-performance AI agent orchestration engine written in Rust. It provides:

- **Gateway**: Axum HTTP/WebSocket server with health probes, Prometheus metrics, CORS, graceful shutdown, and hot reload
- **Desktop App**: Tauri 2 + React 19 cross-platform application embedding the gateway in-process with zero-config launch
- **CLI**: TUI chat, gateway daemon management, and MCP server mode
- **Agents**: 5-tier routing, 13 built-in templates, multi-agent collaboration, intent-based prompt routing
- **Channels**: Native Rust extensions for Feishu, Telegram, Discord, Slack, WhatsApp, Matrix, and Microsoft Teams
- **LLM Support**: OpenAI, Anthropic, DeepSeek, Gemini, DashScope, and Ollama with concurrency control
- **Memory**: Three-layer architecture (working/episodic/semantic) with vector search and petgraph-based knowledge graph
- **DAG Engine**: 9 node types (LLM, Tool, Condition, Parallel, Join, HumanApproval, Loop, Reflect, Code) with SQLite checkpoints
- **Code Intelligence**: Tree-sitter parsing, CodeGraph, LSP integration (rust-analyzer), TestRunner, PatchEngine
- **WASM Plugins**: wasmtime host with fuel limits, HMAC signature verification, and hot reload
- **Security**: API keys, rate limiting, prompt injection guard, HMAC-signed agent bus, SSRF prevention, budget enforcement

## Common Development Commands

### Build

```bash
# Release build (all workspace members)
cargo build --release

# Build specific crate
cargo build --release -p fastclaw-cli
cargo build --release -p fastclaw-gateway

# Desktop app (development)
cd crates/fastclaw-app
cargo tauri dev

# Desktop app (production bundle)
cd crates/fastclaw-app
cargo tauri build

# Install CLI to toolchain
cargo install --path crates/fastclaw-cli
```

### Test

```bash
# All workspace tests
cargo test --workspace

# Specific crate tests
cargo test -p fastclaw-gateway
cargo test -p fastclaw-core

# Run single test
cargo test --package fastclaw-gateway --test gateway_tests test_name
```

### Lint and Format

```bash
# Format code
cargo fmt

# Clippy with warnings as errors (required before PR)
cargo clippy --workspace -- -D warnings

# Check for unused dependencies
cargo udeps
```

### Development Workflow

```bash
# Start gateway in foreground (alias of gateway run)
fastclaw serve

# Terminal UI against gateway
fastclaw tui --url ws://127.0.0.1:18789/ws

# Gateway daemon lifecycle
fastclaw gateway start
fastclaw gateway status
fastclaw gateway stop

# Launch desktop app (requires build first)
fastclaw app
```

### Docker

```bash
# Build and run with Docker Compose
docker compose up -d

# Check health
curl http://127.0.0.1:18789/health
```

## Architecture Overview

### Workspace Structure

```
FastClaw/
├── crates/
│   ├── fastclaw-app/              # Tauri 2 desktop app (React + embedded gateway)
│   │   ├── src-tauri/             # Rust backend: embedded gateway, IPC, LSP
│   │   └── src/                   # React frontend: components, stores, transport
│   ├── fastclaw-cli/              # CLI binary: TUI, daemon, MCP server
│   ├── fastclaw-core/             # Config types, routing, tool registry, message bus, config ACL
│   ├── fastclaw-gateway/          # Axum HTTP/WS, webhooks, REST API
│   ├── fastclaw-agent/            # Agent runtime, LLM providers, built-in tools, LSP manager
│   ├── fastclaw-session/          # SQLite WAL sessions, TTL, compression
│   ├── fastclaw-memory/           # Working/episodic/semantic memory + vectors + graph
│   ├── fastclaw-dag/              # DAG definition, execution, checkpoints
│   ├── fastclaw-plugin/           # WASM host, signatures, hot reload
│   ├── fastclaw-evolution/        # Feedback, evaluation, distillation, skills
│   ├── fastclaw-eval/             # Evaluation framework
│   ├── fastclaw-observe/          # Prometheus metrics, tracing
│   ├── fastclaw-security/         # API keys, rate limiting, prompt guard
│   ├── fastclaw-collab/           # Delegation, MCP client/server
│   ├── fastclaw-model-router/     # Model strategies, tiers, budgets
│   ├── fastclaw-context/          # Six-layer context, rolling compression, context window enforcement
│   ├── fastclaw-self-iter/        # Execution diagnosis, auto-recovery
│   ├── fastclaw-cron/             # Cron persistence + recovery
│   └── fastclaw-treesitter/       # Tree-sitter bindings
├── extensions/                    # Native channel crates (7)
│   ├── discord/ feishu/ matrix/ msteams/
│   ├── slack/ telegram/ whatsapp/
├── config/                        # JSON5 templates + agent profiles
├── docs/                          # Documentation
└── scripts/                       # Release, build, and utility scripts
```

### Request Flow: Message → Channel → Routing → Agent → LLM → Response

1. **Ingress**: Message arrives via channel (Feishu/Slack/etc), TUI, HTTP, or WebSocket
2. **Channel Layer**: Extension converts vendor protocol to internal unified message
3. **Routing**: Static bindings and dynamic routing rules select target `agent_id`
4. **Session**: Resolve session ID, read/write history from SQLite
5. **Agent Runtime**: Assemble system prompt, tools, memory injection
6. **Context Window Enforcement**: Resolve effective `contextWindow` (model config > Agent config > `ModelRouter::max_context_for_model` > safe default), call `ContextEngine::fit_to_context_window` to ensure total tokens don't exceed limit. Strategy: importance-based compression → sliding window fallback, never dropping system messages or current user turn
7. **LLM & Tool Loop**: Call model; if tool_calls returned, execute built-in/WASM/MCP tools and feed results back
8. **Egress**: Push final response via channel SDK or WebSocket events. Streaming completion events include `contextTokens` and `contextWindow` for client-side context usage display

### Key Design Principles

- **Config-Driven**: Agents, bindings, channels, models, and memory mostly in JSON5 for Git management and auditability
- **Hot Reload & Safe Fail**: Agent directory file changes or `SIGHUP` trigger reload; validation failure keeps previous working version (atomic rollback)
- **Observable**: Prometheus metrics and health check endpoints for existing ops stacks
- **Extension-First**: Channels as extension crates; tools via WASM and MCP to reduce core changes
- **OpenClaw Compatible**: Config keys, session semantics, and ops patterns aligned for easier migration

## Important Implementation Details

### Context Window Management

FastClaw enforces per-model context windows with automatic compaction before LLM calls:

- Priority order: explicit model config → Agent-level override → `ModelRouter::max_context_for_model` lookup → safe default
- Compaction strategy in `fastclaw-context`: importance-based compression first, sliding window fallback
- System messages and current user turn are never dropped
- Real-time `ctx 10.2k / 128k` usage streams to clients during execution
- Configurable at both agent and model levels

### Memory Architecture

Three-layer memory system in `fastclaw-memory`:

1. **Working Memory**: Current conversation window with rolling compression
2. **Episodic Memory**: Time-series interaction records with embeddings
3. **Semantic Memory**: Facts and relationships, optionally organized as petgraph knowledge graph

Auto-capture mechanisms:
- **Keyword Interception**: Bilingual trigger words (remember/note/记住/记一下) auto-store facts
- **Importance Scoring**: 5-signal weighted evaluation (length, tool_calls, keywords, depth, corrections)
- **LLM Consolidation**: Asynchronous summarization and fact extraction on high-scoring conversations
- **Dreaming Pipeline**: Background job (hourly by default) for relationship/fact extraction and embedding backfill

### Agent Configuration

Agents are defined in `config/default.json` or per-directory profiles:

```json5
{
  "agents": {
    "defaults": { "model": "dashscope/qwen-plus", "workspace": "workspace" },
    "list": [
      {
        "id": "main",
        "name": "FastClaw Assistant",
        "default": true,
        "model": { "contextWindow": 128000 },  // Override for this agent
        "tools": { "allow": ["web_search", "read_file"], "deny": [] },
        "skills": ["feishu-channel-rules"]
      }
    ]
  }
}
```

Routing via `bindings` array and dynamic `/api/v1/routes` API. Multi-agent collaboration patterns (delegation, pipeline, debate, committee) via `fastclaw-collab`.

### Security Model

Key security features in `fastclaw-security`:

- **API Keys**: Constant-time comparison to prevent timing attacks
- **Rate Limiting**: Per-IP request throttling
- **Prompt Injection Guard**: Input sanitization
- **Agent Bus**: HMAC-SHA256 signed messages with replay protection and hop-depth limits
- **WASM Sandbox**: Epoch-based graceful shutdown, fuel limits
- **SSRF Prevention**: Private IP blocking + DNS resolution checks
- **Path Traversal Guards**: Validation on all file operations
- **Webhook Signature Verification**: Slack/WhatsApp/Feishu signature validation
- **Budget Enforcement**: Atomic reserve/release for cost control
- **Code Sandbox**: Shell disabled, size limits on generated code
- **Config ACL**: Readable/writable key allowlists with secret masking for UI exposure

### Code Intelligence

In `fastclaw-agent` and `fastclaw-treesitter`:

- **Tree-sitter**: Rust/JS/TS/Python with regex fallback for Go/Java
- **CodeGraph**: BFS callers/callees, impact analysis, SCC cycle detection
- **LSP Integration**: `LspSessionManager` with `workspace_symbols`, `go_to_definition`, `find_references`; bundled rust-analyzer in release artifacts
- **TestRunner**: cargo/pytest/npm/go test execution
- **PatchEngine**: Atomic multi-file patch apply/rollback/verify
- **Cross-file Rename**: Symbol renaming across file boundaries

## Testing Requirements

- **665+ workspace tests** must pass before PR submission
- **Zero warnings** in `cargo clippy --workspace -- -D warnings`
- **Zero warnings** in clean CI builds
- Large behavior changes must update corresponding docs in `docs/` and `docs/design/technical-design.md`

## Configuration

Config resolution order:
1. `$FASTCLAW_CONFIG_PATH` environment variable
2. `~/.fastclaw/config/default.json`
3. `$OPENCLAW_CONFIG_PATH` (for OpenClaw compatibility)
4. `~/.openclaw/openclaw.json`
5. Bundled defaults

Agent files in `config/agents/` hot-reload on change with atomic rollback on validation failure.

First run auto-initializes base directories and minimal config (`~/.fastclaw/config/default.json`), plus workspace identity files (`SOUL.md`, `USER.md`, `AGENTS.md`) when creating/updating agents.

## Platform-Specific Notes

### Windows Build Environment

Required one-time setup:

```powershell
# Rust toolchain
winget install --id Rustlang.Rustup
$env:RUSTUP_DIST_SERVER = "https://mirrors.ustc.edu.cn/rust-static"
$env:RUSTUP_UPDATE_ROOT = "https://mirrors.ustc.edu.cn/rust-static/rustup"
rustup default stable

# MSVC Build Tools
winget install --id Microsoft.VisualStudio.2022.BuildTools --override "--quiet --wait --norestart --nocache --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"

# Optional Cargo mirror (in .cargo-home/config.toml)
[source.crates-io]
replace-with = "rsproxy"
[source.rsproxy]
registry = "sparse+https://rsproxy.cn/index/"
```

Build commands require MSVC environment:

```powershell
cmd /c '"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" && cd /d "%REPO%" && cargo build --release'
```

### usearch Backend

Vector search in `fastclaw-memory` can use optional `usearch` backend (C++17, GCC 12+ required). On older compilers, build without the `usearch-backend` feature flag.

## Release Artifacts

- **CLI binary**: `<repo-root>/target/release/fastclaw` (or `.exe` on Windows)
- **MSI installer**: `<repo-root>/target/release/bundle/msi/FastClaw_0.0.2_x64_en-US.msi`
- **NSIS installer**: `<repo-root>/target/release/bundle/nsis/FastClaw_0.0.2_x64-setup.exe`
- **AppImage**: `<repo-root>/target/release/bundle/appimage/FastClaw.AppImage`
- **DMG**: `<repo-root>/target/release/bundle/macos/FastClaw_0.0.2_aarch64.dmg`

Release guide: `docs/release-guide.md`
