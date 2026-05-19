## ADDED Requirements

### Requirement: Multi-platform sandbox manager
系统 SHALL 提供 `SandboxManager` 抽象，根据运行时平台自动选择沙箱实现（Linux: Landlock + Seccomp, macOS: Seatbelt, Windows: RestrictedToken），将 shell 命令包裹在受限环境中执行。

#### Scenario: Linux 平台自动选择 Landlock
- **WHEN** FastClaw 运行在 Linux 5.13+ 系统上且沙箱功能已启用
- **THEN** `SandboxManager` SHALL 选择 Landlock 作为沙箱实现，使用 `prctl(PR_SET_NO_NEW_PRIVS)` + Landlock ruleset 限制子进程

#### Scenario: macOS 平台自动选择 Seatbelt
- **WHEN** FastClaw 运行在 macOS 系统上且沙箱功能已启用
- **THEN** `SandboxManager` SHALL 选择 Seatbelt 作为沙箱实现，通过 `sandbox-exec -f <policy>` 执行子进程

#### Scenario: Windows 平台自动选择 RestrictedToken
- **WHEN** FastClaw 运行在 Windows 系统上且沙箱功能已启用
- **THEN** `SandboxManager` SHALL 选择 RestrictedToken 作为沙箱实现，通过 `CreateRestrictedToken` + Job Object 限制子进程

#### Scenario: 不支持的平台优雅降级
- **WHEN** 当前平台不支持任何沙箱实现（如旧版 Linux 内核）
- **THEN** 系统 SHALL 记录警告日志并以无沙箱模式执行命令，不阻塞正常流程

### Requirement: File system access control
沙箱 SHALL 根据 `PermissionProfile` 中的 `FileSystemPolicy` 限制子进程的文件系统访问，仅允许读写白名单目录。

#### Scenario: 子进程只能写入白名单目录
- **WHEN** FileSystemPolicy 配置为 Restricted，writable_roots 包含 `/home/user/project`
- **THEN** 子进程 SHALL 能够读写 `/home/user/project` 及其子目录
- **AND** 子进程尝试写入 `/home/user/.ssh/` 时 SHALL 收到权限拒绝错误

#### Scenario: 子进程受 deny_globs 限制
- **WHEN** FileSystemPolicy 配置了 deny_globs 包含 `**/.env`
- **THEN** 子进程 SHALL 无法读取任何路径匹配 `.env` 的文件

#### Scenario: Unrestricted 模式不限制文件访问
- **WHEN** FileSystemPolicy 配置为 Unrestricted
- **THEN** 沙箱 SHALL 不对文件系统访问施加额外限制

### Requirement: Network access control
沙箱 SHALL 根据 `PermissionProfile` 中的 `NetworkPolicy` 控制子进程的网络访问能力。

#### Scenario: 禁用网络访问
- **WHEN** NetworkPolicy 配置为 Disabled
- **THEN** 子进程 SHALL 无法发起任何网络连接（TCP/UDP）

#### Scenario: 白名单模式仅允许指定主机
- **WHEN** NetworkPolicy 配置为 AllowList，hosts 包含 `api.github.com`
- **THEN** 子进程 SHALL 能够连接 `api.github.com`
- **AND** 子进程尝试连接其他主机时 SHALL 被拒绝

#### Scenario: 启用网络访问不限制
- **WHEN** NetworkPolicy 配置为 Enabled
- **THEN** 沙箱 SHALL 不对网络访问施加额外限制

### Requirement: Sandbox transform interface
`SandboxManager` SHALL 提供 `transform(command, profile) -> SandboxedCommand` 接口，将原始命令和权限配置转换为平台特定的沙箱化命令。

#### Scenario: 转换为 Landlock 沙箱命令
- **WHEN** 调用 `transform()` 且平台为 Linux
- **THEN** SHALL 返回包含 Landlock ruleset 设置逻辑的 `SandboxedCommand`，其中包含程序路径、参数、工作目录、环境变量和沙箱类型

#### Scenario: 转换为 Seatbelt 沙箱命令
- **WHEN** 调用 `transform()` 且平台为 macOS
- **THEN** SHALL 返回以 `sandbox-exec -f <generated-policy-file>` 为前缀的 `SandboxedCommand`
