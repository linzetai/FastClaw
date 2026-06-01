## 1. Dispatcher 计划文件白名单 + 并行执行修复

- [x] 1.1 `DispatchContext` 新增 `plan_file_path: Option<PathBuf>` 字段；runtime 在构建 context 时从 `PlanContext` 注入计划文件路径
- [x] 1.2 `pre_execution_checks` 中 `ToolKind::Edit` 拦截处增加路径检查：当 tool 为 `write_file`/`edit_file` 且 `path` 参数 canonicalize 后匹配 `plan_file_path` 时放行
- [x] 1.3 `execute_unguarded_standalone` 增加 Plan mode 下 `ToolKind::Edit`/`Execute` 阻塞逻辑（复用 `is_blocked_for_tool` 语义），包含计划文件白名单例外
- [x] 1.4 更新 `dynamic.rs` 中的 Plan mode prompt，确认写入指导与新白名单一致
- [x] 1.5 `cargo clippy -- -D warnings` 验证

## 2. 审批闸门（exit_plan_mode 重构）

- [x] 2.1 修改 `ExitPlanModeTool::execute`：成功时不调用 `transition(Agent)`，返回 `metadata: { "approval_pending": true, "plan_path": "...", "plan_exists": ... }`，output 文案改为"等待用户审批"
- [x] 2.2 新增 WS RPC `execution.approve_plan`：接受 `{ sessionId, mode }` 参数，执行 `transition(target_mode)`，广播 `mode_change` + `plan_file_update`
- [x] 2.3 `transport.ts` 新增 `approvePlan(sessionId: string, mode?: string)` API 封装
- [x] 2.4 `PlanApprovalCard.tsx` 重构：检测 `metadata.approval_pending`（从 StepIndicator 传入），渲染"开始实现"和"继续规划"按钮，处理审批回调
- [x] 2.5 `StepIndicator.tsx` 向 `PlanApprovalCard` 传入 `onImplement` 回调（调用 `approvePlan` + `setChatExecutionMode`）和 `toolMetadata`

## 3. 模式切换事件广播统一

- [x] 3.1 `handle_execution_set_mode` 在 from != to 时通过 `bg_tx` 广播 `mode_change` 事件
- [x] 3.2 若存在 `PlanContext`，同时广播 `plan_file_update` 事件

## 4. 计划面板（PlanPanel）

- [x] 4.1 新建 `PlanPanel.tsx` 组件：接受 `sessionId` + `planFilePath` + `planFileExists`，调用 `getPlanFile` 获取内容，用 `react-markdown` + `remark-gfm` 渲染
- [x] 4.2 `StreamFooter.tsx` plan banner 改为可点击，切换 `showPlanPanel` 状态
- [x] 4.3 在 `MessageStream` 或 `AppLayout` 层集成 `PlanPanel` 作为右侧面板（宽度 ~360px），受 `showPlanPanel` 状态控制
- [x] 4.4 监听 `plan_file_update` 事件，面板打开时自动 refetch
- [x] 4.5 Agent 模式下若 `planFileExists`，StreamFooter 显示 subtle plan indicator（可点击打开面板）

## 5. 验证

- [x] 5.1 `cargo clippy -- -D warnings` 零警告
- [x] 5.2 `pnpm tsc --noEmit` 零错误
- [x] 5.3 Dev 测试：Plan 模式下 write_file 正确阻塞
- [x] 5.4 Dev 测试：exit_plan_mode 后审批 UI 正确渲染（"计划等待审批"、"开始实现"、"继续规划"）
- [x] 5.5 Dev 测试：PlanPanel 正确显示，点击 Plan Mode banner 打开侧面板

## 6. Bug 修复：exit_plan_mode 后 Agent 不停止

- [x] 6.1 runtime 工具结果循环中检测 `metadata.approval_pending == true`，设置 `plan_approval_pending` 标志
- [x] 6.2 工具结果循环结束后，若 `plan_approval_pending` 为 true，发送 TurnEnd 事件并 `return Ok(make_turn_summary(...))`
- [x] 6.3 `cargo clippy -- -D warnings` 零警告

## 7. Bug 修复：YOLO 模式下仍需审批

- [x] 7.1 `session_bridge.rs` 新增 `derive_approval_strategy` 函数：当 `tools_ask` 和 `require_confirmation_for` 均为空且 `file_access == Full` 时返回 `AutoApprove`，否则返回 `Interactive`
- [x] 7.2 替换 `session_bridge.rs` 中两处硬编码的 `ApprovalStrategy::Interactive` 为 `derive_approval_strategy(&config.behavior)`
- [x] 7.3 `cargo clippy -- -D warnings` 零警告
- [x] 7.4 Dev 测试：YOLO 模式下 plan 文件写入不弹审批窗
- [x] 7.5 Dev 测试：Default 模式下 shell 命令仍需审批
