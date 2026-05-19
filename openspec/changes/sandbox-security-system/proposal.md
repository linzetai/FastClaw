## Why

FastClaw 当前的安全体系仅覆盖「命令发送前的静态分析」（regex/AST 注入检测、dangerous_ops 模式匹配、tool 级 allow/deny 权限规则），但子进程一旦通过检查并执行，就拥有与 FastClaw 主进程完全相同的文件系统和网络权限。这意味着：

- Agent 执行的 shell 命令可以读写 `~/.ssh`、`~/.gnupg` 等敏感目录
- 子进程可以不受限制地发起任意网络连接
- 没有 OS 级别的进程隔离，静态分析被绕过即全面失守

参考 OpenAI Codex 的四层安全体系（OS Sandbox → PermissionProfile → ExecPolicy → Guardian），我们需要补齐 OS 级沙箱隔离、命令级策略引擎、LLM 辅助审核三个缺失层次，将 FastClaw 的安全能力从「静态检测」提升到「纵深防御」。

## What Changes

- **新增 `fastclaw-sandbox` crate** — 多平台 OS 级进程沙箱，支持 Linux (Landlock + Seccomp)、macOS (Seatbelt/sandbox-exec)、Windows (RestrictedToken)。`SandboxManager` 将 shell 命令包裹在受限环境中执行，子进程只能读写白名单目录、按策略控制网络访问。
- **新增 `fastclaw-execpolicy` crate** — 基于 TOML 配置的命令级策略规则引擎。支持前缀匹配规则（`["git", "push", "--force"] → forbidden`）、网络域名规则、批量只读命令快捷方式，以及内联测试验证。策略可分层叠加（system → project → session）。
- **新增 `fastclaw-guardian` crate** — LLM 辅助安全审核 Agent。对需要审核的操作，用独立 LLM session 评估是否符合用户意图、风险是否可接受。fail-closed 设计（超时/失败默认拒绝）。
- **增强 `fastclaw-security` crate** — 新增 `PermissionProfile` 模块，定义文件系统策略（Restricted/Unrestricted）和网络策略（Enabled/Disabled/AllowList），作为连接 execpolicy 和 sandbox 的桥梁。
- **改造 `fastclaw-agent` 的 ShellTool** — 命令执行路径从直接 `tokio::Command` 改为经过 ExecPolicy 检查 → Guardian 审核（可选）→ SandboxManager 包裹执行的完整链路。

## Capabilities

### New Capabilities

- `os-sandbox`: OS 级进程沙箱隔离，支持 Linux/macOS/Windows 三平台，将子进程限制在白名单文件系统和网络策略内
- `exec-policy`: 基于 TOML 配置的命令级策略规则引擎，支持前缀匹配、网络域名规则、分层叠加、内联测试验证
- `guardian-review`: LLM 辅助安全审核，用独立 Agent 评估操作安全性，fail-closed 设计
- `permission-profile`: 文件系统和网络访问权限配置模型，连接策略引擎与 OS 沙箱

### Modified Capabilities

（无现有 spec 需要修改，但 `fastclaw-agent` 的 ShellTool 执行路径会发生变化）

## Impact

- **新增 crate**: `fastclaw-sandbox`、`fastclaw-execpolicy`、`fastclaw-guardian` 加入 workspace
- **修改 crate**: `fastclaw-security`（新增 permission_profile 模块）、`fastclaw-agent`（ShellTool 执行链路改造）
- **新增依赖**: `landlock`（Linux Landlock API）、`seccompiler`（Seccomp BPF，可选）；macOS 和 Windows 使用系统 API 无额外依赖
- **配置文件**: 新增 `.fastclaw/exec-policy.toml` 配置格式
- **API 变化**: ShellTool 的执行接口内部重构，对外 tool calling 协议不变
- **性能影响**: 沙箱启动有微量开销（~1-5ms），Guardian 审核增加一次 LLM 调用（可选启用）
- **向后兼容**: 默认不启用沙箱和 Guardian，现有行为完全保持；通过配置逐步开启
