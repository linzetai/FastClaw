## 1. PermissionProfile 基础模型

- [x] 1.1 在 `fastclaw-security` crate 中新增 `permission_profile` 模块，定义 `PermissionProfile`、`FileSystemPolicy`、`NetworkPolicy` 结构体和枚举
- [x] 1.2 实现 `PermissionProfile` 的 `Serialize`/`Deserialize`，支持 JSON 序列化/反序列化
- [x] 1.3 为 `PermissionProfile` 编写单元测试（构造、序列化往返、边界情况）
- [x] 1.4 在 `fastclaw-security/src/lib.rs` 中导出 `permission_profile` 模块

## 2. OS 级沙箱 — fastclaw-sandbox crate

- [x] 2.1 创建 `crates/fastclaw-sandbox` crate 骨架（Cargo.toml、lib.rs），加入 workspace members
- [x] 2.2 定义 `SandboxManager` trait 和 `SandboxedCommand` 结构体（程序路径、参数、环境变量、工作目录、沙箱类型）
- [x] 2.3 实现 `SandboxManager::detect()` — 运行时检测当前平台可用的沙箱实现
- [x] 2.4 实现 Linux Landlock 后端 — 根据 `PermissionProfile` 生成 Landlock ruleset 并限制子进程文件系统访问
- [x] 2.5 实现 Linux Landlock 网络规则（需 Landlock v4+，不可用时优雅降级）
- [x] 2.6 实现 macOS Seatbelt 后端 — 根据 `PermissionProfile` 生成 `.sb` 策略文件，通过 `sandbox-exec` 执行
- [x] 2.7 实现 Windows RestrictedToken 后端 — 使用 `CreateRestrictedToken` + Job Object 限制子进程权限
- [x] 2.8 实现 `transform(command, profile) -> SandboxedCommand` 统一接口
- [x] 2.9 实现不支持平台的 NoopSandbox 后端（warn 日志 + 透传命令）
- [x] 2.10 为每个平台后端编写集成测试（Linux 测试需在 Linux CI 上运行）

## 3. 命令级策略引擎 — fastclaw-execpolicy crate

- [x] 3.1 创建 `crates/fastclaw-execpolicy` crate 骨架，加入 workspace members
- [x] 3.2 定义 TOML 配置 schema（`PolicyConfig`、`PrefixRule`、`NetworkRule`、`PolicyTest`、`Defaults`）
- [x] 3.3 实现 TOML 文件解析器，支持 `defaults.allow_readonly` 批量展开为前缀规则
- [x] 3.4 实现前缀匹配算法 — 命令 token 数组与规则 pattern 逐位比较，支持备选项 `["git", ["merge", "rebase"]]`
- [x] 3.5 实现三层策略叠加（System → Project → Session），合并规则：Forbidden > Prompt > Allow
- [x] 3.6 实现 `evaluate(command_tokens) -> PolicyDecision` 主评估函数
- [x] 3.7 实现网络规则评估（host + protocol 匹配）
- [x] 3.8 实现 `[[tests]]` 段的内联验证逻辑，支持 CI 和启动时自检
- [x] 3.9 为规则匹配、策略叠加、边界情况编写全面的单元测试
- [x] 3.10 创建示例配置文件 `examples/exec-policy.toml`

## 4. Guardian Agent — fastclaw-guardian crate

- [x] 4.1 创建 `crates/fastclaw-guardian` crate 骨架，加入 workspace members
- [x] 4.2 定义 `GuardianAssessment` 结构体（decision、risk_level、rationale）
- [x] 4.3 实现 intent transcript 重建 — 从对话历史提取用户意图，限制 token 数（默认 10000）
- [x] 4.4 实现审核 prompt 构建 — 包含用户意图 transcript + 待执行操作 + 评估指令
- [x] 4.5 实现 `review(operation, context) -> GuardianAssessment` 主审核函数，调用 LLM 获取结构化 JSON 输出
- [x] 4.6 实现 fail-closed 逻辑 — 超时（默认 60s）、解析失败、调用失败时返回 deny
- [x] 4.7 实现可选启用逻辑 — 配置 `guardian.enabled` 控制，未启用时直接跳过
- [x] 4.8 为 Guardian 编写单元测试（mock LLM 响应、超时、格式异常、禁用场景）

## 5. ShellTool 执行链路改造

- [x] 5.1 在 `fastclaw-agent/Cargo.toml` 中添加 `fastclaw-sandbox`、`fastclaw-execpolicy`、`fastclaw-guardian` 依赖
- [x] 5.2 重构 `SandboxedShellTool::execute()` — 在现有 shell_security 检查后插入 ExecPolicy 评估
- [x] 5.3 在 ExecPolicy 返回 Prompt 时，接入 Guardian 审核（如已启用）或用户确认流程
- [x] 5.4 在 ExecPolicy 返回 Allow 后，通过 `SandboxManager::transform()` 包裹命令并执行
- [x] 5.5 将现有 `tokio::process::Command::new("bash")` 替换为通过 `SandboxedCommand` 执行
- [x] 5.6 确保沙箱未启用时行为与改造前完全一致（向后兼容）
- [x] 5.7 为完整执行链路（ExecPolicy → Guardian → Sandbox → Execute）编写集成测试

## 6. 配置与文档

- [x] 6.1 在 FastClaw 配置体系中添加沙箱相关配置项（sandbox.enabled、sandbox.profile、guardian.enabled 等）
- [x] 6.2 创建默认 exec-policy 配置模板（合理的只读命令白名单 + 常见危险命令 Forbidden）
- [ ] 6.3 在 README 或独立文档中说明沙箱安全体系的使用方法
- [x] 6.4 添加 deny_globs 默认值（`.env`、`.ssh`、`.gnupg` 等敏感路径）

## 7. CI 与质量保障

- [x] 7.1 在 CI 中为 Linux 平台添加 Landlock 集成测试（需 Linux 5.13+ runner）
- [x] 7.2 确保所有新 crate 通过 `cargo clippy` 和 `cargo fmt` 检查
- [x] 7.3 确保 exec-policy 的 `[[tests]]` 段在 CI 中自动运行
- [ ] 7.4 验证 `cargo deny check` 对新增依赖无安全警告
