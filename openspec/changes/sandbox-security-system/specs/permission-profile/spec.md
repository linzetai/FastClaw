## ADDED Requirements

### Requirement: Composite permission profile
系统 SHALL 提供 `PermissionProfile` 数据模型，组合文件系统策略和网络策略，作为 ExecPolicy 和 SandboxManager 之间的桥梁。

#### Scenario: 创建受限 profile
- **WHEN** 创建 PermissionProfile 指定 `FileSystemPolicy::Restricted` 和 `NetworkPolicy::Disabled`
- **THEN** 生成的 profile SHALL 同时包含文件系统白名单和网络禁用配置

#### Scenario: 创建不受限 profile
- **WHEN** 创建 PermissionProfile 指定 `FileSystemPolicy::Unrestricted` 和 `NetworkPolicy::Enabled`
- **THEN** 生成的 profile SHALL 表示对文件系统和网络不施加额外限制

### Requirement: File system policy definition
`FileSystemPolicy` SHALL 支持三种模式：Unrestricted（无限制）、Restricted（白名单 + deny glob）。

#### Scenario: Restricted 模式定义可写目录
- **WHEN** FileSystemPolicy 设置为 Restricted，writable_roots 为 `["/home/user/project", "/tmp"]`
- **THEN** profile SHALL 表示仅这些目录及其子目录可写

#### Scenario: deny_globs 排除敏感文件
- **WHEN** FileSystemPolicy 设置 deny_globs 为 `["**/.env", "**/.ssh/**", "**/.gnupg/**"]`
- **THEN** profile SHALL 表示匹配这些 glob 的路径即使在 writable_roots 下也不可访问

#### Scenario: readable_roots 控制可读范围
- **WHEN** FileSystemPolicy 设置 readable_roots 为 `["/home/user/project", "/usr"]`
- **THEN** profile SHALL 表示仅这些目录及其子目录可读

### Requirement: Network policy definition
`NetworkPolicy` SHALL 支持三种模式：Enabled（无限制）、Disabled（禁止所有网络）、AllowList（白名单主机）。

#### Scenario: AllowList 模式
- **WHEN** NetworkPolicy 设置为 AllowList，hosts 为 `["api.github.com", "registry.npmjs.org"]`
- **THEN** profile SHALL 表示仅允许连接这些主机

#### Scenario: Disabled 模式
- **WHEN** NetworkPolicy 设置为 Disabled
- **THEN** profile SHALL 表示禁止所有出站网络连接

### Requirement: Profile serialization
PermissionProfile SHALL 支持 JSON 序列化/反序列化，以便通过命令行参数或环境变量传递给沙箱子进程。

#### Scenario: 序列化为 JSON
- **WHEN** 调用 `serde_json::to_string(&profile)`
- **THEN** SHALL 生成有效的 JSON 字符串，包含所有策略配置

#### Scenario: 从 JSON 反序列化
- **WHEN** 从 JSON 字符串反序列化 PermissionProfile
- **THEN** SHALL 还原完整的策略配置，与原始 profile 相等

### Requirement: Profile to sandbox mapping
PermissionProfile SHALL 提供 `to_sandbox_config()` 方法，将高层策略转换为平台特定的沙箱配置参数。

#### Scenario: 映射到 Landlock ruleset
- **WHEN** 在 Linux 平台调用 `to_sandbox_config()`
- **THEN** SHALL 返回包含 Landlock 文件系统规则和网络规则的配置结构体

#### Scenario: 映射到 Seatbelt policy
- **WHEN** 在 macOS 平台调用 `to_sandbox_config()`
- **THEN** SHALL 返回 `.sb` 格式的 Seatbelt 策略字符串
