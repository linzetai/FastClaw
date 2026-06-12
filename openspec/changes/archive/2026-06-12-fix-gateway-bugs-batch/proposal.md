## Why

第二轮代码审查覆盖 gateway、session-actor、builtin_tools、context、前端组件等模块，发现 10 个 Bug（编号 11-20），涵盖事件丢弃、消息重复持久化、全局状态竞态、配置丢失、安全隐患等问题。多个 Bug 直接影响用户可感知的功能（如消息重复、工具卡片缺失、语言设置失效），需要批量修复。

## What Changes

- 修复 WS chat 事件循环中 Error+TurnEnd 序列导致 assistant 消息重复持久化
- 修复 `enriched_request.response_language` 被硬编码为 `None` 导致用户语言偏好失效
- 修复 session actor relay task 不尊重 BackpressurePolicy 导致事件静默丢弃
- 修复 GoalStore `set_session_id` 不重置 `idle_rounds`/`budget_warning_sent`
- 修复 WS turn 10 分钟超时不向前端发送错误通知
- 修复 reactive compactor `ensure_system_messages` 仅比较数量不比较内容
- 修复前端 `handleRecallLastMessage` 不处理 multimodal content
- 修复 GoalStore 全局单例在多 session 场景下的竞态条件
- 修复 MCP tools prompt 中硬编码下标可能导致 panic
- 修复 turn 取消时不完整 assistant 消息被持久化到 session 历史

## Capabilities

### New Capabilities

_(无新增能力)_

### Modified Capabilities

_(这批 Bug 均为内部实现修复，不涉及已有 spec 的需求层变更)_

## Impact

- **Gateway crate**：`ws/chat.rs`（事件循环、turn 超时），`chat_pipeline.rs`（response_language、MCP prompt）
- **Session-actor crate**：`actor.rs`（relay task 分发策略）
- **Agent crate**：`builtin_tools/goal.rs`（GoalStore 状态管理）
- **Context crate**：`reactive.rs`（system message 恢复逻辑）
- **前端**：`StreamFooter.tsx`（消息回显）
- **测试**：需要为竞态、事件丢弃、消息重复补充测试
