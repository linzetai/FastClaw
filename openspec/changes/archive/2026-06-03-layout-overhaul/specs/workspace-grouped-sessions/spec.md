## MODIFIED Requirements

### Requirement: Sessions grouped by workspace
AppSidebar 中的会话列表 SHALL 支持按 `workDir`（工作目录）分组，渲染位置从原 SessionList 组件移到 AppSidebar 的 "Projects" 分组区域。

#### Scenario: Group sessions by normalized workDir
- **WHEN** sessions 有不同的 `workDir` 值
- **THEN** 在 AppSidebar 的 "Projects" 分组下按工作目录分组显示
- **AND** 每个项目名称旁有彩色圆点标识

#### Scenario: Sessions without workDir
- **WHEN** 会话没有 `workDir`（null）
- **THEN** 显示在 "Chats" 分组下（而非 "Projects"）

#### Scenario: Session group display
- **WHEN** 渲染 AppSidebar 的 "Projects" 区域
- **THEN** 每个项目显示为一行：彩色圆点 + 项目名称（目录最后一段）
- **AND** 点击项目名称展开该项目下的会话列表

### Requirement: Workspace context awareness
- **WHEN** 用户创建新会话
- **THEN** 会话自动继承当前工作目录作为 `workDir`
- **AND** 会话出现在 AppSidebar 的对应项目分组下

## ADDED Requirements

### Requirement: Pinned sessions section
AppSidebar SHALL 支持 "Pinned" 分组，显示用户手动固定的会话。Pinned 分组位于 Projects 和 Chats 分组之上。

#### Scenario: Pin a session
- **WHEN** 用户右键某个会话选择 "固定"
- **THEN** 该会话移到 "Pinned" 分组
- **AND** 在 Pinned 分组内按最后更新时间排序

#### Scenario: Unpin a session
- **WHEN** 用户右键已固定的会话选择 "取消固定"
- **THEN** 该会话从 "Pinned" 分组移回原分组（Projects 或 Chats）
