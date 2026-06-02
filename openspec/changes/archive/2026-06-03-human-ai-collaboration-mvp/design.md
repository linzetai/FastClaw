## Context

XiaoLin 后端已实现完整的人机协作基础设施：

- **中断**：`SessionActor` 支持 `SessionOp::Interrupt`，Gateway WS handler `handle_chat_cancel` 已实现，`ClientOp::ChatCancel` 已在路由表
- **审批**：`ToolOrchestrator` 五阶段流水线（Requirement → Approval → Sandbox → Execute → Escalate），`InteractionHandle` 统一 approval/answer
- **Steer**：`handle_chat_steer` 已实现，`ClientOp::ChatSteer` 已路由到 SessionActor
- **BriefMessage**：`AgentEvent::BriefMessage` 已定义，`brief_message` 已在前端事件订阅列表
- **ExecPolicy**：`PolicyEngine` 完整实现三层规则（session > project > system），但 Gateway 未加载
- **Guardian**：`xiaolin-agent/src/guardian/mod.rs` 有 `GuardianReviewer`，但 Orchestrator 创建时未挂载

前端断点：`stopStream()` 未发 WS cancel、transport.ts 无 `chatCancel` 函数、无 BriefMessage handler、无 Steer 发送函数、审批复用 QuestionPanel。

## Goals / Non-Goals

**Goals:**
- 前端 Stop → 后端 Interrupt 端到端贯通
- Gateway 启动时加载默认 ExecPolicy，Shell 命令受策略约束
- 新增 `ApprovalCard` 组件替代 QuestionPanel 处理审批，展示命令/diff/风险/理由
- BriefMessage 在聊天流中渲染为进度卡片
- Streaming 期间用户可通过输入框发送 Steer 指令
- 点赞/踩写回后端 Evolution API，Retry 重新执行 turn
- Pending 审批时发送 Tauri 系统通知
- 默认策略从 AutoApprove 收紧为 Interactive

**Non-Goals:**
- Guardian LLM 全面启用（本次只做 Orchestrator 挂载点预留，不强制开启）
- `request_permissions` 完整实现（scope 扩展涉及沙箱改动，scope 太大）
- 多端（IM）审批（飞书/微信渠道仍 auto-deny）
- 工作室 / 文件页面（独立 change）
- 语音输入（独立 change）

## Decisions

### D1: Stop → Interrupt 的实现路径

**选择**：在 `transport.ts` 新增 `chatCancel(sessionId)` 函数，发送 `chat.cancel` WS 消息；`stopStream()` 在现有 UI 清理之前先调用 `transport.chatCancel()`。

**替代方案**：通过 Tauri IPC 直接调用 Rust 命令。但 Gateway 已有 WS handler，且 WS 是当前所有 chat 操作的统一通道，保持一致性更重要。

**rationale**：最小改动、最快贯通。后端 `handle_chat_cancel` 已经处理了 cancel token 取消和 TurnAborted 事件发送。

### D2: 默认 ExecPolicy 策略设计

**选择**：在 `config/exec-policy.json` 提供内置策略文件，Gateway `State::init()` 阶段加载。默认规则：
- 只读命令（ls, cat, git status, cargo check 等）→ Allow
- 文件修改命令（rm, mv, chmod 等）→ Prompt
- 系统管理命令（sudo, systemctl, reboot 等）→ Forbidden
- 网络出站（curl, wget 到非 localhost）→ Prompt
- 无匹配 → Prompt（保守默认）

**rationale**：ExecPolicy 代码已完整，只需在初始化时 `PolicyEngine::load_file()`。用户可通过配置 `exec_policy_path` 覆盖。

### D3: ApprovalCard 组件设计

**选择**：新增 `ApprovalCard.tsx` 组件，从 `approval_required` 事件的 payload 中提取：
- `action`（工具名 + 参数摘要）→ 命令预览区
- `reason`（ExecPolicy 匹配规则或 Guardian 理由）→ 风险说明
- `available_decisions`（Approve / ApproveForSession / Deny / Abort）→ 操作按钮

与 `QuestionPanel` 区分：QuestionPanel 保留给 `ask_question` 工具的结构化问答，ApprovalCard 专门处理 `approval:` 前缀的 requestId。

**替代方案**：扩展 QuestionPanel 增加审批模式。但两者的 UI 需求差异大（审批需要命令预览、风险标签、remember 粒度），拆开更清晰。

### D4: BriefMessage 渲染策略

**选择**：在 `useMessageStreamChat.ts` 的事件分发中增加 `brief_message` handler，将消息插入聊天流作为特殊类型的 StreamItem。前端渲染为轻量级的内联卡片（区别于 assistant message）。

`mode: "proactive"` 的消息带蓝色左边框和 info 图标，`mode: "normal"` 的消息样式更低调。

### D5: Mid-turn Steer 交互模式

**选择**：Streaming 状态下，输入框 placeholder 变为 "追加指令..."，发送行为从 `chat.send` 改为 `chat.steer`。输入框保持相同组件（MentionInput），只是发送目标不同。

在 `transport.ts` 新增 `chatSteer(sessionId, messages)` 函数。在 `useMessageStreamChat` 中根据 `streaming` 状态决定调用 `chatSend` 还是 `chatSteer`。

**替代方案**：独立的 Steer 输入框浮在消息流上方。但这增加 UI 复杂度，且用户心智模型是"在同一个地方输入"。

### D6: 反馈写回路径

**选择**：
- 点赞/踩：在 `transport.ts` 新增 `submitFeedback(sessionId, turnId, rating)`，调用后端 `POST /api/v1/evolution/feedback`
- Retry：在 `transport.ts` 新增 `retryTurn(sessionId, turnId)`，发送 `chat.retry` WS 消息（后端 SessionActor 已有 turn rollback 基础）

### D7: 系统通知策略

**选择**：当 `approval_required` 或 `ask_question` 事件到达且当前窗口不在前台时，通过 `@tauri-apps/plugin-notification` 发送系统通知。点击通知聚焦到对应会话。

不在每条消息都通知，只在需要用户操作时通知。

### D8: 默认审批策略收紧

**选择**：修改 `session_bridge.rs` 中的默认策略推导逻辑，将无显式配置时的默认值从 `AutoApprove` 改为 `Interactive`。用户仍可通过 agent 配置设置 `approval_strategy: "auto_approve"` 恢复 YOLO 模式。

## Risks / Trade-offs

- **[Stop 中断可能导致工具执行中途断开]** → 后端已有 100ms 宽限期 + `AbortOnDropHandle` 清理；Shell 进程通过 process group kill 处理
- **[默认策略收紧增加用户操作成本]** → 提供 "ApproveForSession" 选项减少重复确认；后续可加信任等级机制
- **[Steer 注入可能导致 agent 上下文混乱]** → 后端 SteerInput 会作为 user message 追加到当前 turn，agent 能看到
- **[BriefMessage 频繁推送可能打扰用户]** → 渲染为低调样式，不中断流式输出，不触发通知
- **[ExecPolicy 默认规则可能误拦合法命令]** → 默认规则宽松（只 Forbid 系统管理类），用户可自定义覆盖
