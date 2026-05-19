## Context

FastClaw 当前安全体系的核心问题：所有安全检查都发生在命令执行前（静态分析），一旦命令通过检查，子进程以 `tokio::process::Command::new("bash").arg("-c")` 直接执行，拥有与主进程完全相同的权限。

现有安全组件：
- `fastclaw-security`: API Key 认证、速率限制、SSRF 防护、prompt 注入检测、dangerous_ops 模式匹配
- `fastclaw-agent/shell_security.rs`: 15 种 shell 注入模式检测（regex + tree-sitter AST）
- `fastclaw-agent/runtime/permissions.rs`: tool 级 allow/deny 权限规则引擎（Session/Global 双层）

参考对象：OpenAI Codex 的四层安全体系（sandboxing/execpolicy/PermissionProfile/guardian），约 96 个 crate 中有 ~15 个直接服务于安全。

约束条件：
- 必须支持 Linux、macOS、Windows 三平台
- 不能破坏现有行为（默认关闭，渐进启用）
- 单二进制 < 50MB 目标不能被额外依赖破坏
- 服务端（多用户网关）和本地（TUI/桌面）两种部署场景

## Goals / Non-Goals

**Goals:**
- 实现 OS 级进程沙箱，子进程只能访问白名单目录和网络端点
- 实现命令级策略引擎，基于 TOML 配置控制哪些命令允许/禁止/需确认
- 实现 LLM Guardian Agent，对高风险操作进行语义级审核
- 定义 PermissionProfile 模型，统一文件系统和网络权限配置
- 改造 ShellTool 执行链路：ExecPolicy → Guardian → Sandbox
- 保持向后兼容，默认行为不变

**Non-Goals:**
- 容器级隔离（Docker/cgroup）— 属于部署层面，不在本次范围
- 自定义 DSL 策略语言 — 使用 TOML 而非 Starlark
- WebAssembly 插件的沙箱增强 — 已有 wasmtime 沙箱，不在本次范围
- 文件系统操作工具（read_file/write_file/edit_file）的沙箱化 — 本次聚焦 shell 命令执行

## Decisions

### D1: 沙箱技术选型

**决定**: 每个平台使用各自原生最佳方案。

| 平台 | 技术 | 理由 |
|------|------|------|
| Linux | Landlock LSM (v4+) + Seccomp BPF | 内核原生，无需额外二进制，非特权进程可用，Linux 5.13+ 广泛支持 |
| macOS | Seatbelt (sandbox-exec) | 系统自带，Apple 官方沙箱机制，无需额外依赖 |
| Windows | CreateRestrictedToken + Job Object | Win32 API 原生，降低进程令牌权限 + 限制资源访问 |

**替代方案考虑:**
- Bubblewrap (bwrap): Codex 在 Linux 上同时用 Landlock + bwrap。bwrap 提供 mount namespace 级别的隔离更强，但需要额外二进制分发。**决定**: 先用纯 Landlock，后续按需加 bwrap。
- Firejail: 功能丰富但过重，不适合嵌入式使用。
- 统一用 Docker/容器: 过重，不适合本地 CLI 场景。

### D2: 策略配置格式

**决定**: 使用 TOML 格式（`.fastclaw/exec-policy.toml`）。

**理由:**
- 与项目现有风格一致（`config/default.json`、`Cargo.toml`、`deny.toml`）
- 零额外依赖（已有 serde + toml）
- 策略配置文件不应该是可执行代码（安全原则）
- 通过 `defaults.allow_readonly` 等语法糖解决批量声明

**替代方案:**
- Starlark DSL（Codex 的选择）: 支持动态规则生成（变量/循环/条件），但引入 ~3MB 编译体积，Codex 选它是因为整个构建系统基于 Bazel/Starlark。对我们而言收益不足以覆盖成本。

### D3: Guardian 实现方式

**决定**: 作为独立 crate `fastclaw-guardian`，利用现有 Agent 框架的 LLM 调用能力，而非内嵌在 core 中。

**理由:**
- FastClaw 是多 Agent 编排引擎，Guardian 天然是一个「安全审核 Agent」
- 独立 crate 可以单独测试、独立启用/禁用
- 与 Codex 的区别：Codex 的 guardian 是 core 模块的一部分，因为 Codex 是单 Agent 架构

**设计要点:**
- fail-closed: 超时（默认 60 秒）或 LLM 调用失败时默认拒绝
- 结构化 JSON 输出: `{ "decision": "allow|deny", "risk_level": "low|medium|high", "rationale": "..." }`
- 上下文重建: 从对话历史中提取用户意图的紧凑 transcript，限制 token 数量
- 可选启用: 通过配置 `guardian.enabled = true` 开启，默认关闭

### D4: 执行链路架构

**决定**: 分层管道设计，每一层可独立启用/禁用。

```
ShellTool.execute(command)
  │
  ├─ 1. 现有检查（保留）
  │    ├─ shell_security: 注入模式检测
  │    ├─ dangerous_ops: 用户自定义危险命令检测
  │    └─ PermissionRuleEngine: tool 级权限
  │
  ├─ 2. ExecPolicy 检查（新增）
  │    ├─ 解析命令为 token 数组
  │    ├─ 匹配 prefix_rules → Allow/Forbidden/Prompt
  │    ├─ 匹配 network_rules（如果命令涉及网络）
  │    └─ Forbidden → 直接拒绝; Prompt → 进入 Guardian 或用户确认
  │
  ├─ 3. Guardian 审核（新增，可选）
  │    ├─ 构建审核 prompt（用户意图 + 待执行操作）
  │    ├─ 调用 LLM 获取结构化评估
  │    └─ deny → 拒绝; allow → 继续
  │
  └─ 4. Sandbox 执行（新增）
       ├─ SandboxManager.transform(command, profile) → SandboxedCommand
       ├─ 平台分发: Landlock / Seatbelt / RestrictedToken
       └─ 执行受限子进程，返回结果
```

### D5: PermissionProfile 设计

**决定**: 组合式 profile，独立于沙箱实现。

```rust
pub struct PermissionProfile {
    pub file_system: FileSystemPolicy,
    pub network: NetworkPolicy,
}

pub enum FileSystemPolicy {
    Unrestricted,
    Restricted {
        writable_roots: Vec<PathBuf>,
        readable_roots: Vec<PathBuf>,
        deny_globs: Vec<String>,  // e.g., "**/.env", "**/.ssh/**"
    },
}

pub enum NetworkPolicy {
    Enabled,
    Disabled,
    AllowList { hosts: Vec<String> },
}
```

### D6: 策略分层与叠加

**决定**: 三层策略叠加，优先级从高到低。

1. **Session 层**: 运行时动态添加的临时规则（如用户在 TUI 中确认 "always allow git push"）
2. **Project 层**: 项目目录下的 `.fastclaw/exec-policy.toml`
3. **System 层**: `~/.fastclaw/exec-policy.toml`（用户全局默认）

合并规则：高优先级的 Forbidden 不可被低优先级的 Allow 覆盖。同层内 Forbidden > Prompt > Allow。

### D7: Crate 依赖关系

```
fastclaw-sandbox       ← 纯 OS 交互，无其他 fastclaw 依赖
    ↑
fastclaw-security      ← 新增 PermissionProfile，依赖 sandbox
    ↑
fastclaw-execpolicy    ← 规则引擎，依赖 security（用 PermissionProfile）
    ↑
fastclaw-guardian       ← 依赖 core（LLM 调用能力）
    ↑
fastclaw-agent         ← 集成所有层，改造 ShellTool
```

## Risks / Trade-offs

**[R1] Landlock 内核版本要求** → Linux 5.13+ 才支持 Landlock。旧内核优雅降级到纯静态检查（warn + 继续执行）。运行时检测 `prctl(PR_GET_NO_NEW_PRIVS)` 判断可用性。

**[R2] macOS Seatbelt 为私有 API** → Apple 未公开 sandbox-exec 的长期支持承诺。但 Codex、Homebrew 等大量工具都在使用，短期风险可控。策略文件使用 `.sb` 格式（Scheme-like），需要生成器。

**[R3] Guardian LLM 调用延迟** → 每次审核增加 1-5 秒延迟 + API 费用。通过以下方式缓解：默认关闭；缓存已审核的命令模式；只在 ExecPolicy 返回 Prompt 时触发。

**[R4] 沙箱逃逸风险** → Landlock/Seatbelt 不是绝对安全的（内核漏洞、符号链接攻击）。定位为「纵深防御的一层」而非唯一防线，与静态检查、Guardian 配合使用。

**[R5] Windows RestrictedToken 功能有限** → Windows 沙箱在文件系统粒度控制上不如 Linux/macOS。第一期实现基本的令牌降权 + Job Object 限制，后续迭代增强。

**[R6] 编译体积增长** → `landlock` crate 很小（~50KB），`seccompiler` 较大但可选。macOS/Windows 使用系统 API 零额外依赖。预计总增量 < 500KB。

**[R7] 与现有 SandboxedShellTool 的兼容** → 现有 `SandboxedShellTool` 只做命令黑名单拦截，新的 `SandboxManager` 是进程级隔离。两者互补而非替代。迁移策略：保留现有检查作为快速前置过滤，sandbox 作为执行层兜底。
