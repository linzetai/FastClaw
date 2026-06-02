## ADDED Requirements

### Requirement: Virtual scroll renders all visible messages
消息列表 SHALL 使用 `@tanstack/react-virtual` 的 `useVirtualizer` hook 实现虚拟滚动，仅渲染视口内及 overscan 范围内的消息 DOM 节点。

#### Scenario: 初始渲染可见消息
- **WHEN** 用户切换到一个包含 N 条消息的聊天会话
- **THEN** 视口内的消息 SHALL 在 500ms 内完成渲染并可见（opacity > 0，非 visibility:hidden）

#### Scenario: WebKitGTK 环境正常渲染
- **WHEN** 应用运行在 Linux Tauri（WebKitGTK）+ React 19 环境下
- **THEN** 消息列表 SHALL 正常渲染，不得出现 visibility:hidden 死锁

### Requirement: End-anchored scroll behavior
消息列表 SHALL 使用 `anchorTo: 'end'` 配置，以底部为锚定边缘。

#### Scenario: 初始加载滚动到底部
- **WHEN** 聊天会话首次加载
- **THEN** 视口 SHALL 定位到最新消息（列表底部）

#### Scenario: 新消息自动跟随
- **WHEN** 用户处于列表底部且有新消息到达
- **THEN** 视口 SHALL 自动滚动以显示新消息

#### Scenario: 阅读历史时不抢夺滚动
- **WHEN** 用户已向上滚动查看历史消息，此时新消息到达
- **THEN** 视口 SHALL 保持当前位置不变

### Requirement: Streaming output keeps bottom pinned
Streaming 响应期间，最后一条消息的高度会随 token 流入持续增长。

#### Scenario: Streaming 时视口跟随增长
- **WHEN** AI 正在 streaming 输出，且用户处于列表底部
- **THEN** 视口 SHALL 随消息高度增长保持底部对齐

#### Scenario: Streaming 时用户回看不被拉回
- **WHEN** AI 正在 streaming 输出，但用户已向上滚动
- **THEN** 视口 SHALL 保持当前位置不变

### Requirement: Dynamic message height measurement
每条消息的高度 SHALL 通过 `measureElement` 回调动态测量，支持不同类型消息的可变高度。

#### Scenario: 不同消息类型正确测量
- **WHEN** 列表包含用户消息、AI 消息、工具调用卡片等不同类型
- **THEN** 每条消息 SHALL 按实际渲染高度正确定位，无重叠或空白

#### Scenario: 消息内容变化后重新测量
- **WHEN** 一条消息的内容发生变化（如 streaming 追加内容）
- **THEN** 该消息的高度 SHALL 被重新测量，后续消息位置 SHALL 自动调整

### Requirement: Stable item keys for prepend
每个虚拟列表项 SHALL 使用基于消息 ID 的稳定 key，而非数组索引。

#### Scenario: Prepend 历史消息后视口稳定
- **WHEN** 用户滚动到顶部触发加载更多历史消息
- **THEN** 当前可见消息 SHALL 保持在视口中相同位置，不发生跳动

### Requirement: Scroll FAB (Jump to bottom)
当用户不在列表底部时，SHALL 显示"滚到底部"浮动按钮。

#### Scenario: 向上滚动后显示 FAB
- **WHEN** 用户从底部向上滚动超过阈值
- **THEN** FAB 按钮 SHALL 出现

#### Scenario: 点击 FAB 滚动到底部
- **WHEN** 用户点击 FAB 按钮
- **THEN** 视口 SHALL 平滑滚动到列表底部，FAB 消失

### Requirement: Search result scroll positioning
搜索定位功能 SHALL 能将视口滚动到匹配的消息位置。

#### Scenario: 跳转到搜索结果
- **WHEN** 用户在搜索框中输入关键词并按 Enter
- **THEN** 视口 SHALL 滚动到对应消息位置，高亮标记可见

### Requirement: Pagination (load more history)
当消息总数超过一页时，SHALL 支持向上滚动加载更多历史消息。

#### Scenario: 触发加载更多
- **WHEN** 用户滚动到列表顶部附近（scrollTop < 200px）且有更多历史
- **THEN** 系统 SHALL 加载下一页历史消息，并保持当前可见消息位置稳定

#### Scenario: 无更多历史时不触发
- **WHEN** 所有消息已加载完毕，用户滚动到顶部
- **THEN** 系统 SHALL 不触发加载操作

### Requirement: Chat key isolation
切换不同聊天会话时，虚拟列表 SHALL 完全重置。

#### Scenario: 切换会话后列表重建
- **WHEN** 用户从会话 A 切换到会话 B
- **THEN** 虚拟列表 SHALL 使用会话 B 的消息重新渲染，滚动位置独立

### Requirement: No react-virtuoso dependency
项目 SHALL 不依赖 `react-virtuoso` 包。

#### Scenario: 依赖清理
- **WHEN** 迁移完成后
- **THEN** `package.json` 中 SHALL 不包含 `react-virtuoso`，代码中 SHALL 无 `react-virtuoso` 的 import
