## Why

XiaoLin 的后端已经实现了完整的人机协作基础设施（SessionActor interrupt、InteractionHandle、ToolOrchestrator 五阶段审批、ExecPolicy、Guardian、ChatSteer），但前端与后端之间存在多处关键断点：Stop 按钮不中断后端执行、ExecPolicy/Guardian 未加载、BriefMessage 无 UI、审批体验过于简陋。这些问题使得用户无法安全可控地使用 AI 助手，是产品发布的首要阻塞项。

## What Changes

- **Stop → Interrupt 贯通**：前端 Stop 按钮调用后端 `ChatCancel`，真正中断 agent 执行、停止 LLM 调用和工具运行
- **ExecPolicy 默认加载**：Gateway 初始化时加载内置默认策略文件，Shell 命令执行受策略约束
- **审批 UX 升级**：从复用 QuestionPanel 改为专用 ApprovalCard 组件，展示命令预览、Diff 预览、风险等级
- **BriefMessage UI**：监听 `brief_message` 事件，渲染 agent 主动推送的进度消息
- **Mid-turn Steer 输入**：streaming 状态下输入框变为"追加指令"模式，发送 `ChatSteer` 给后端
- **反馈闭环**：点赞/踩写回后端 Evolution 系统，Retry 按钮真正重新执行 turn
- **Pending 审批通知**：agent 等待用户审批时，发送 Tauri 系统通知 + 托盘图标闪烁
- **DenialTracker 生效**：用户拒绝操作后 agent 不再重复请求同类操作
- **默认策略收紧**：AutoApprove 不作为默认配置，默认使用 Interactive 策略

## Capabilities

### New Capabilities
- `stop-interrupt`: Stop 按钮到后端 Interrupt 的端到端贯通，包括前端调用、后端取消、流式清理
- `approval-ux`: 专业化的工具执行审批界面，包括命令预览、diff 展示、风险等级、Guardian 理由可视化
- `brief-message-ui`: Agent 主动推送消息（BriefMessage）的前端展示，支持 normal 和 proactive 模式
- `mid-turn-steer`: Streaming 期间用户追加指令的输入和传输能力
- `feedback-loop`: 用户反馈（点赞/踩/retry）写回后端的完整闭环
- `pending-approval-notify`: Agent 等待审批时的系统级通知（Tauri notification + 托盘提醒）

### Modified Capabilities
- `plan-approval-gate`: 审批流从 QuestionPanel 迁移到专用 ApprovalCard，增加 remember 粒度选项
- `quick-action-bar`: 接通 Gateway WebSocket 使全局快捷入口可用

## Impact

- **前端**：`useMessageStreamChat.ts`（Stop/Steer/BriefMessage handler）、新增 `ApprovalCard.tsx` 组件、修改 `StreamFooter.tsx`（Steer 模式）、修改 `MessageRenderer.tsx`（BriefMessage 渲染）、修改 `QuickActionBar.tsx`（接通 WebSocket）
- **后端**：`xiaolin-gateway/src/state.rs`（ExecPolicy 加载、Guardian 挂载）、`xiaolin-agent/src/runtime/orchestrator.rs`（DenialTracker 接入）
- **协议**：无新事件，所有 `AgentEvent` 和 `ClientOp` 已定义完整
- **配置**：新增默认 ExecPolicy 文件 `config/exec-policy.json`，默认策略从 AutoApprove 改为 Interactive
- **依赖**：无新 crate 依赖
