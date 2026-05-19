## ADDED Requirements

### Requirement: TOML-based policy configuration
系统 SHALL 支持通过 TOML 文件（`.fastclaw/exec-policy.toml`）定义命令执行策略，包含前缀规则、网络规则和批量快捷方式。

#### Scenario: 解析前缀规则
- **WHEN** 配置文件包含 `[[rules]] pattern = ["git", "push", "--force"] decision = "forbidden"`
- **THEN** 策略引擎 SHALL 将匹配 `git push --force *` 前缀的命令判定为 Forbidden

#### Scenario: 解析批量只读命令
- **WHEN** 配置文件包含 `[defaults] allow_readonly = ["ls", "cat", "head"]`
- **THEN** 策略引擎 SHALL 为每个列出的命令自动生成 `decision = "allow"` 的前缀规则

#### Scenario: 解析网络规则
- **WHEN** 配置文件包含 `[[network]] host = "api.github.com" protocol = "https" decision = "allow"`
- **THEN** 策略引擎 SHALL 允许目标为 `api.github.com` 的 HTTPS 连接

### Requirement: Prefix-based command matching
策略引擎 SHALL 使用前缀匹配算法，将命令 token 数组与规则 pattern 逐位比较，支持精确匹配和备选项匹配。

#### Scenario: 精确前缀匹配
- **WHEN** 规则 pattern 为 `["npm", "install"]` 且命令为 `["npm", "install", "express"]`
- **THEN** 策略引擎 SHALL 判定该规则匹配

#### Scenario: 前缀不匹配
- **WHEN** 规则 pattern 为 `["npm", "install"]` 且命令为 `["npm", "run", "build"]`
- **THEN** 策略引擎 SHALL 判定该规则不匹配

#### Scenario: 备选项匹配
- **WHEN** 规则 pattern 包含备选项 `["git", ["merge", "rebase"]]` 且命令为 `["git", "rebase", "main"]`
- **THEN** 策略引擎 SHALL 判定该规则匹配

### Requirement: Three-level policy layering
策略 SHALL 支持三层叠加：Session（运行时临时）> Project（.fastclaw/exec-policy.toml）> System（~/.fastclaw/exec-policy.toml），高优先级规则覆盖低优先级。

#### Scenario: Session 规则覆盖 Project 规则
- **WHEN** Project 层定义 `["curl"] → prompt` 且 Session 层动态添加 `["curl"] → allow`
- **THEN** 策略引擎 SHALL 判定 `curl` 命令为 Allow

#### Scenario: Forbidden 不可被低优先级 Allow 覆盖
- **WHEN** System 层定义 `["rm", "-rf", "/"] → forbidden` 且 Session 层添加 `["rm"] → allow`
- **THEN** 策略引擎 SHALL 判定 `rm -rf /` 命令为 Forbidden（Forbidden 优先级最高）

### Requirement: Policy evaluation with decision types
策略引擎 SHALL 返回三种决策之一：Allow（允许执行）、Forbidden（拒绝执行并报错）、Prompt（需要用户确认或 Guardian 审核）。

#### Scenario: 无匹配规则时使用启发式回退
- **WHEN** 命令未匹配任何规则
- **THEN** 策略引擎 SHALL 调用配置的启发式回退函数（默认为 Prompt）

#### Scenario: Forbidden 决策拒绝执行
- **WHEN** 策略引擎返回 Forbidden
- **THEN** ShellTool SHALL 拒绝执行命令并返回包含 justification 的错误信息

### Requirement: Inline test validation
配置文件 SHALL 支持 `[[tests]]` 段定义验证示例，用于 CI 或启动时验证策略规则的正确性。

#### Scenario: 验证通过
- **WHEN** 配置包含 `[[tests]] command = "ls -la" expect = "allow"` 且 `ls` 有 allow 规则
- **THEN** 验证 SHALL 通过

#### Scenario: 验证失败报错
- **WHEN** 配置包含 `[[tests]] command = "rm -rf /" expect = "forbidden"` 但无匹配规则
- **THEN** 验证 SHALL 失败并报告不匹配的测试用例
