# Tool Usage Guide

## File Operations
- **read_file**: Read a file by path. Always read before editing.
- **write_file**: Create or fully overwrite a file.
- **edit_file**: Targeted find-and-replace edits within a file.
- **search_in_files**: Search file contents by pattern (regex/glob) across workspace.
- **apply_patch**: Apply a unified diff patch.
- **list_directory**: List files/dirs at a path.

## Shell
- **shell_exec**: Run shell commands. Prefer dedicated tools when they exist. Sandboxed.

## Web
- **web_search**: Search the web for current information. Requires backend config (Tavily API key or SearXNG instance); unconfigured returns a setup prompt.
- **web_fetch**: Fetch and extract readable text/markdown from an HTTP(S) document. Best for long HTML pages, docs, READMEs.
- **http_fetch**: HTTP GET a URL and return raw response text (≤4 KB). Best for small JSON APIs, health checks, version probes. SSRF policy blocks private/localhost URLs.

## Code Intelligence
- **workspace_symbols**: Search symbols by name across workspace.
- **go_to_definition**: Jump to a symbol's definition.
- **find_references**: Find all references to a symbol.

## Memory

Persistent long-term memory. Use actively.

**memory_store** — Store when: user says "记住"/"remember", states a preference, corrects you, key decision made, session ends with outcomes, non-obvious discovery. Use `type=fact` for preferences/context, `type=episode` for decisions/outcomes. Never store secrets/keys/tokens.

**memory_search** — Search before answering context-dependent questions, when user references past conversations, or when making assumptions about preferences.

## Interaction
- **ask_question**: Present structured questions with options.
- **confirm**: Yes/no confirmation before destructive actions.

## Session Management
- **sessions_spawn**: Start a new session with another agent.
- **sessions_send**: Send a message to an existing session.

## Scheduling
- **manage_cron**: CRUD for scheduled cron jobs.

## Skills
- **list_skills** / **read_skill** / **write_skill**: Manage agent skills.
  > Only available when `skills.prompt_mode` is `"compact"` or `"lazy"`. In `"full"` mode (default), skill content is injected directly into the system prompt and these tools are not registered. `write_skill` additionally requires an active AgentWorkspace.

## Identity
- **get_identity** / **set_identity**: Read/update agent persona files (SOUL.md, USER.md).

## Utilities
- **get_current_time**: Current date and time.
- **calculator**: Evaluate math expressions.
- **browser**: Chrome automation via CDP. Launches a visible Chrome window by default; persistent tab preserves session/cookies across calls. Requires `feature = "browser"` at compile time and Chrome/Chromium installed at runtime. Set `FASTCLAW_BROWSER_HEADLESS=true` for CI/server.

  **Navigation:** `navigate` (url, optional wait_for selector for SPAs) · `go_back` · `go_forward` · `reload` (optional ignore_cache)

  **DOM interaction:** `click` (selector) · `type` (selector + text, optional clear) · `press_key` (key name: Enter, Tab, Escape, etc.) · `hover` (selector)

  **DOM query:** `select` (selector, optional attribute) · `wait_for` (selector, timeout_ms) · `get_content` (optional selector for element-specific)

  **Visual:** `screenshot` (optional url, optional selector for element-level, optional output_path) · `scroll` (selector to scroll into view, or direction + pixels)

  **Data:** `cookies` (operation: get/set/delete/clear, cookie_name, cookie_value) · `pdf` (output_path)

  **Advanced:** `evaluate` (script, optional url — returns JSON) · `interact` (url, wait_seconds — hand browser to user for login/CAPTCHA)

  Typical form flow: `navigate` → `wait_for` → `type` → `click` → `wait_for` result.
  Login flow: `navigate` → `interact` (user logs in) → `cookies get` → `navigate` to protected page.
- **image_generate**: Generate images from text.
- **text_to_speech**: Convert text to audio.

## Agent Discovery
- **spawn_subagent**: Launch a sub-agent to handle a task in a separate session.
- **list_agents**: List all configured agents with their IDs and descriptions.
- **get_agent_info**: Get detailed info about a specific agent by ID.

## MCP Extensions

**mcp_***: Tools from external MCP servers (`mcp_{serverId}_{toolName}`). Use like built-in tools.

**manage_mcp_server**: Add/remove/list/reload MCP servers at runtime.
- `list` — show servers + status
- `add` — register server (`id`, `command`, `args`)
- `remove` — unregister (`id`)
- `reload` — restart all connections

Install workflow: `shell_exec` to install package → `manage_mcp_server(action:"add")` to register.

## Channel Integrations

**list_channels** / **add_channel** / **remove_channel**: Manage IM channel connections.

Supported channels and required credentials:

| Channel   | Required                            | Optional                                   |
|-----------|-------------------------------------|-------------------------------------------|
| feishu    | appId, appSecret                    | connectionMode (websocket/webhook), domain, replyMode, userAccessToken |
| slack     | appSecret (xoxb-... Bot Token)      | verificationToken (Signing Secret), appId  |
| discord   | appSecret (Bot Token), appId        |                                            |
| telegram  | appSecret (Bot Token from BotFather)|                                            |
| whatsapp  | appId (Phone Number ID), appSecret (Token) | verificationToken (Webhook Verify Token) |
| matrix    | domain (Homeserver URL), appId (User ID), appSecret (Access Token) |          |
| msteams   | appId (Bot App ID), appSecret (Password) |                                     |

Workflow: `list_channels` → ask user which channel → collect credentials one by one via `ask_question` → `add_channel`. Never guess credentials; always ask the user. After adding, remind about webhook URL setup if applicable: `/webhook/{channelId}`.

## Sub-Agent Delegation

Use `spawn_subagent` to delegate tasks to specialized child agents. Each sub-agent runs independently with its own context, tools, and session — it does **not** share your conversation history.

### Agent Discovery

Before delegating, use the agent discovery tools to choose the right target:

- **`list_agents`** — Returns all available agents with their IDs, names, descriptions, models, and capabilities. Call this first to see what agents exist.
- **`get_agent_info`** — Returns detailed configuration for a specific agent (model, tools, behavior, delegation policy). Use when you need to understand an agent's capabilities before delegating.

Workflow: `list_agents` → pick agent → optionally `get_agent_info` for details → `spawn_subagent` with the chosen `agent_id`.

### When to Use

**DO** use sub-agents when:
- The task has 2+ independent sub-problems that benefit from parallel execution.
- A subtask requires a different tool set (e.g., browser automation while you do code analysis).
- The task is complex enough that dedicated focus improves quality.
- You need to research/explore while continuing your main reasoning chain.

**DO NOT** use sub-agents for:
- Simple single-tool operations — just call the tool directly.
- Tasks that need your current conversation context (sub-agents start fresh).
- Sequential steps where each depends on the previous result (do them yourself).

### Sub-Agent Types

| Type | Best For | Available Tools |
|------|----------|-----------------|
| `general` | Full-capability subtask | Inherits parent's tool set |
| `explore` | Read-only research, code exploration, codebase questions | file_read, search, web_search, web_fetch, memory_search |
| `shell` | Command execution, builds, tests, git operations | shell_exec, file_read, file_write |
| `browser` | Web interaction, UI testing, scraping | browser_*, web_fetch |

### Parameters

```json
{
  "task": "Clear, self-contained description of what the sub-agent should accomplish",
  "agent_id": "main",
  "subagent_type": "explore",
  "context": "Optional: key facts/data the sub-agent needs but cannot discover on its own"
}
```

**Important**: Always use a valid `agent_id` from the `list_agents` results. If unsure which agent to use, call `list_agents` first.

### Writing Good Task Descriptions

1. **Self-contained** — include all context needed. The sub-agent cannot see your conversation.
2. **Specific outcome** — state exactly what to return (e.g., "Return the file path and function name").
3. **Bounded scope** — one clear objective per sub-agent.
4. **Include constraints** — mention language, framework, or files to focus on when relevant.

### Parallel Execution

Batch multiple `spawn_subagent` calls in **one response** for parallel execution:

```
Thought: I need to research the auth module AND run the test suite simultaneously.
→ spawn_subagent(type=explore, task="Find all authentication middleware in src/...")
  AND spawn_subagent(type=shell, task="Run `cargo test` and report failures...")
```

The runtime executes them concurrently and returns all results before your next reasoning step.

## Quick Reference

1. Context-dependent task? → `memory_search` first
2. Known file? → `read_file` · Find file/symbol? → `search_in_files` / `workspace_symbols`
3. External info? → `web_search` · Run commands? → `shell_exec`
4. Learned something? → `memory_store` immediately
5. Connect IM channel? → `list_channels` → `add_channel` with user-provided credentials
