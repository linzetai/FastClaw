## Why

代码审查发现 agent runtime 中存在 10 个 Bug，涵盖工具执行时序、事件丢失、上下文污染、状态残留等问题。其中多个 Bug 会导致用户可感知的功能异常（如 undo 失效、目标状态不同步、上下文膨胀），需要批量修复以提升运行时的健壮性和正确性。

## What Changes

- 修复 streaming 路径下 undo 快照在工具执行后才捕获的时序问题
- 修复 turn 取消时 GoalUpdated 事件因 injector 提前 abort 而丢失
- 修复 reactive loop 中 goal_store 未传入导致目标预算追踪失效
- 修复流式恢复重试时 partial assistant messages 累积污染上下文
- 修复 `had_tool_calls_this_round` 标志跨迭代残留导致 stop hooks 误触发
- 修复 model switch reminder 插入位置错误（插在 user message 之前）
- 修复 streaming 路径工具结果按位置匹配可能错位的问题
- 修复 `reactive_target_tokens` 设置为全上下文窗口未预留输出空间
- 修复 `PermissionSelector` 中途切换预设后当前 turn 不生效
- 修复 `create_file` 在 streaming 路径下 file_exists 判断时序错误

## Capabilities

### New Capabilities

_(无新增能力)_

### Modified Capabilities

- `approval-ux`: 增加中途切换预设后当前 turn 立即生效的支持
- `session-permission-override`: 修复 PermissionSelector 变更传播到运行中 turn 的能力

## Impact

- **后端 crates**：`xiaolin-agent`（runtime/tool_round, llm_call, end_turn, post_tool, turn_loop, orchestrator, session_bridge, mod, turn_setup）
- **协议层**：`xiaolin-protocol`（已在前置 commit 中修改 approval.rs）
- **前端**：`xiaolin-app`（ApprovalCard.tsx 已在前置 commit 中修改）
- **测试**：需要为修复的时序问题和事件传播补充单元测试
