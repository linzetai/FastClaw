## MODIFIED Requirements

### Requirement: Message content max-width
消息内容的 maxWidth SHALL 根据布局等级动态计算，而非硬编码 720px。

#### Scenario: Standard layout width
- **WHEN** 主内容区宽度为 600-900px
- **THEN** 消息内容 maxWidth 为 720px（保持当前行为）

#### Scenario: Wide layout expanded width
- **WHEN** 主内容区宽度超过 900px
- **THEN** 消息内容 maxWidth 扩展为 min(860px, 容器宽度-40px)

#### Scenario: Compact layout full width
- **WHEN** 主内容区宽度小于 600px
- **THEN** 消息内容 maxWidth 为 100%（减去 32px padding）

#### Scenario: Virtual list size estimation
- **WHEN** 布局等级从标准切换到宽屏
- **THEN** 虚拟列表的 `estimateSize` 保持不变（高度估算不受宽度影响），但已渲染的行通过 `measureElement` 重新测量高度
