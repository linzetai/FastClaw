# Phase 0 — 依赖图分析：待删模块依赖矩阵

> 生成日期: 2026-04-28
> 工具: `cargo tree --workspace -i <pkg>` + `rg` 源码搜索

## 1. 依赖矩阵总览

| 待删模块 | 被依赖方 (reverse deps) | 复杂度 |
|----------|------------------------|--------|
| `fastclaw-telegram` | gateway | 低 — 仅 gateway state/mod.rs |
| `fastclaw-discord` | gateway | 低 — 仅 gateway state/mod.rs |
| `fastclaw-slack` | gateway | 低 — 仅 gateway state/mod.rs |
| `fastclaw-whatsapp` | gateway | 低 — 仅 gateway state/mod.rs |
| `fastclaw-matrix` | gateway | 低 — 仅 gateway state/mod.rs |
| `fastclaw-msteams` | gateway | 低 — 仅 gateway state/mod.rs |
| `fastclaw-eval` | cli | 低 — 仅 cli main.rs |
| `fastclaw-dag` | gateway | 中 — gateway routes/state/lib 多处引用 |
| `fastclaw-plugin` | gateway | 中 — gateway routes/state/helpers 多处引用 |
| `fastclaw-collab` | gateway, cli | 高 — MCP 需先拆出再删残余 |

## 2. 待删模块间互依赖

待删的 10 个模块之间**无互相依赖**，可以任意顺序独立删除。

各模块的内部依赖（仅引用保留模块）：

| 模块 | 依赖的保留模块 |
|------|---------------|
| `fastclaw-dag` | `fastclaw-core` |
| `fastclaw-plugin` | `fastclaw-core` |
| `fastclaw-eval` | `fastclaw-core`, `fastclaw-agent` |
| `fastclaw-collab` | `fastclaw-core`, `fastclaw-agent` |
| 6 个 channel extensions | `fastclaw-core` (feishu 额外依赖 agent, session) |

## 3. 推荐删除顺序

基于"叶子节点优先、影响范围从小到大"的原则：

```
第 1 批 (独立叶子，仅 gateway state/mod.rs)：
  ① telegram → ② discord → ③ slack → ④ whatsapp → ⑤ matrix → ⑥ msteams

第 2 批 (仅 cli 引用)：
  ⑦ fastclaw-eval

第 3 批 (gateway 多文件引用)：
  ⑧ fastclaw-dag  (routes/dag.rs → state/builder.rs → state/mod.rs → lib.rs → Cargo.toml)
  ⑨ fastclaw-plugin (routes/plugin.rs → state/helpers.rs → state/builder.rs → state/mod.rs → Cargo.toml)

第 4 批 (需先拆分 MCP，再删残余)：
  ⑩ fastclaw-collab → 先拆出 fastclaw-mcp → 再删 collab 残余
```

**无循环依赖风险**：所有待删模块均为"被依赖方为 gateway/cli"的叶子节点，互不依赖。

## 4. 各模块删除涉及的代码文件清单

### 4.1 fastclaw-telegram

| 操作 | 文件 |
|------|------|
| 删除目录 | `extensions/telegram/` |
| 移除 workspace member | `Cargo.toml` (根) |
| 移除依赖 | `crates/fastclaw-gateway/Cargo.toml` |
| 移除代码引用 | `crates/fastclaw-gateway/src/state/mod.rs` |

### 4.2 fastclaw-discord

| 操作 | 文件 |
|------|------|
| 删除目录 | `extensions/discord/` |
| 移除 workspace member | `Cargo.toml` (根) |
| 移除依赖 | `crates/fastclaw-gateway/Cargo.toml` |
| 移除代码引用 | `crates/fastclaw-gateway/src/state/mod.rs` |

### 4.3 fastclaw-slack

| 操作 | 文件 |
|------|------|
| 删除目录 | `extensions/slack/` |
| 移除 workspace member | `Cargo.toml` (根) |
| 移除依赖 | `crates/fastclaw-gateway/Cargo.toml` |
| 移除代码引用 | `crates/fastclaw-gateway/src/state/mod.rs` |

### 4.4 fastclaw-whatsapp

| 操作 | 文件 |
|------|------|
| 删除目录 | `extensions/whatsapp/` |
| 移除 workspace member | `Cargo.toml` (根) |
| 移除依赖 | `crates/fastclaw-gateway/Cargo.toml` |
| 移除代码引用 | `crates/fastclaw-gateway/src/state/mod.rs` |

### 4.5 fastclaw-matrix

| 操作 | 文件 |
|------|------|
| 删除目录 | `extensions/matrix/` |
| 移除 workspace member | `Cargo.toml` (根) |
| 移除依赖 | `crates/fastclaw-gateway/Cargo.toml` |
| 移除代码引用 | `crates/fastclaw-gateway/src/state/mod.rs` |

### 4.6 fastclaw-msteams

| 操作 | 文件 |
|------|------|
| 删除目录 | `extensions/msteams/` |
| 移除 workspace member | `Cargo.toml` (根) |
| 移除依赖 | `crates/fastclaw-gateway/Cargo.toml` |
| 移除代码引用 | `crates/fastclaw-gateway/src/state/mod.rs` |

### 4.7 fastclaw-eval

| 操作 | 文件 |
|------|------|
| 删除目录 | `crates/fastclaw-eval/` |
| 移除 workspace member | `Cargo.toml` (根) |
| 移除依赖 | `crates/fastclaw-cli/Cargo.toml` |
| 移除代码引用 | `crates/fastclaw-cli/src/main.rs` (cmd_eval 函数 + Commands::Eval 变体) |

### 4.8 fastclaw-dag

| 操作 | 文件 |
|------|------|
| 删除目录 | `crates/fastclaw-dag/` |
| 移除 workspace member | `Cargo.toml` (根) |
| 移除依赖 | `crates/fastclaw-gateway/Cargo.toml` |
| 删除路由文件 | `crates/fastclaw-gateway/src/routes/dag.rs` |
| 移除路由注册 | `crates/fastclaw-gateway/src/routes/mod.rs` (mod dag, use dag::*, 4 条路由) |
| 移除 state 引用 | `crates/fastclaw-gateway/src/state/builder.rs` (CheckpointStore, DagDefinition, DagGraph, DagExecutor) |
| 移除 state 类型 | `crates/fastclaw-gateway/src/state/mod.rs` (CheckpointStore 字段) |
| 移除 lib 引用 | `crates/fastclaw-gateway/src/lib.rs` (DAG 执行相关代码) |

### 4.9 fastclaw-plugin

| 操作 | 文件 |
|------|------|
| 删除目录 | `crates/fastclaw-plugin/` |
| 移除 workspace member | `Cargo.toml` (根) |
| 移除依赖 | `crates/fastclaw-gateway/Cargo.toml` |
| 删除路由文件 | `crates/fastclaw-gateway/src/routes/plugin.rs` |
| 移除路由注册 | `crates/fastclaw-gateway/src/routes/mod.rs` (mod plugin, 2 条路由) |
| 移除 state 引用 | `crates/fastclaw-gateway/src/state/builder.rs` (WasmHost, PluginRegistry, discover_plugins, bridge_plugins, start_watching) |
| 移除 state 类型 | `crates/fastclaw-gateway/src/state/mod.rs` (PluginRegistry 字段) |
| 移除 helpers 引用 | `crates/fastclaw-gateway/src/state/helpers.rs` (PluginRegistry 引用) |
| 注意 | `crates/fastclaw-observe/src/lib.rs` 中 `record_plugin_invocation` 函数使用 `"fastclaw_plugin_invocations_total"` 作为 metrics 名 — 这是纯字符串，不是 crate 依赖，可保留 |
| 注意 | `examples/plugin-template/` 是独立 WASM 项目（非 workspace member），仅文档引用，可选择性保留或删除 |

### 4.10 fastclaw-collab（分两步）

**步骤 A: 拆分 MCP 到 fastclaw-mcp**

| 操作 | 文件 |
|------|------|
| 创建新 crate | `crates/fastclaw-mcp/` |
| 迁移 MCP 代码 | 从 `crates/fastclaw-collab/src/mcp/` 移入 |
| 更新 gateway 引用 | `crates/fastclaw-gateway/src/mcp_tool.rs` (改 `fastclaw_collab::mcp::*` → `fastclaw_mcp::*`) |
| 更新 gateway 引用 | `crates/fastclaw-gateway/src/state/builder.rs` |
| 更新 gateway 引用 | `crates/fastclaw-gateway/src/state/mod.rs` |
| 更新 gateway 依赖 | `crates/fastclaw-gateway/Cargo.toml` |

**步骤 B: 删除 collab 残余**

| 操作 | 文件 |
|------|------|
| 删除目录 | `crates/fastclaw-collab/` |
| 移除 workspace member | `Cargo.toml` (根) |
| 移除 cli 依赖 | `crates/fastclaw-cli/Cargo.toml` |
| 移除 cli 引用 | `crates/fastclaw-cli/src/main.rs` |

## 5. 风险评估

| 风险项 | 等级 | 缓解措施 |
|--------|------|----------|
| gateway state 结构变更导致编译错误 | 低 | 每删一个模块后 `cargo check -p fastclaw-gateway` |
| 路由移除后前端/CLI 调用 404 | 低 | v5 重构，前端尚未对接 DAG/Plugin 路由 |
| collab MCP 拆分遗漏引用 | 中 | 拆分后 `rg fastclaw_collab` 全局验证零匹配 |
| eval 删除后 CLI eval 子命令不可用 | 低 | 属计划内移除 |
| observe 的 plugin metrics 函数成为死代码 | 低 | clippy 会警告，可在 P0-16 中一并处理 |

## 6. 验证检查清单

每个模块删除后执行：

```bash
# 1. 零匹配验证
rg fastclaw_<module> --type rust --type toml

# 2. 编译验证
cargo check -p fastclaw-gateway   # 大多数模块的依赖方
cargo check -p fastclaw-cli       # eval/collab 的依赖方

# 3. 全量验证（批次完成后）
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace
```
