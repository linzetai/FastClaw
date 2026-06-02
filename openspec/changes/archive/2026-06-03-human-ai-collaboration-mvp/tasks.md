## 1. Stop → Interrupt 贯通

- [x] 1.1 在 `transport.ts` 新增 `chatCancel(sessionId: string)` 函数，发送 `chat.cancel` WS 消息
- [x] 1.2 修改 `useMessageStreamChat.ts` 的 `stopStream()` 回调，在 UI 清理之前先调用 `transport.chatCancel()`
- [x] 1.3 处理 `turn_aborted` 事件：在事件 handler 中清理 streaming 状态、关闭 pending approval/question
- [x] 1.4 E2E 验证：启动 dev server，触发 shell 长命令，点 Stop，确认后端进程被 kill

## 2. 默认 ExecPolicy 加载

- [x] 2.1 创建 `config/exec-policy.json` 默认策略文件（只读 Allow、文件修改 Prompt、系统管理 Forbidden）
- [x] 2.2 修改 `xiaolin-gateway/src/state.rs` 的初始化逻辑，当无用户配置时加载内置默认策略
- [x] 2.3 将 `PolicyEngine` 实例注入到 `ToolOrchestrator::with_policy()` 调用
- [x] 2.4 修改 `session_bridge.rs` 默认策略推导：无显式配置时默认 `Interactive` 而非 `AutoApprove`
- [x] 2.5 验证：执行 `rm` 类命令时应弹出审批，`ls` 类命令应直接放行

## 3. 审批 UX 升级（ApprovalCard）

- [x] 3.1 新建 `ApprovalCard.tsx` 组件，支持命令预览、风险等级（红/黄/绿）、操作按钮（Approve / ApproveForSession / Deny / Abort）
- [x] 3.2 修改 `StreamFooter.tsx` 的 pendingQuestion 处理逻辑：`requestId` 以 `approval:` 开头时渲染 ApprovalCard 而非 QuestionPanel
- [x] 3.3 ApprovalCard 中解析 `approval_required` 事件的 `action`、`reason`、`available_decisions` 字段并渲染
- [x] 3.4 为 `write_file` / `edit_file` 审批添加内容 diff 预览（可折叠）
- [x] 3.5 E2E 验证：让 agent 执行 shell 命令，确认 ApprovalCard 正确渲染并能操作

## 4. BriefMessage UI

- [x] 4.1 在 `useMessageStreamChat.ts` 的事件分发中增加 `brief_message` handler
- [x] 4.2 将 brief_message 数据插入到 chat stream 作为新的 StreamItem 类型 `"brief"`
- [x] 4.3 新建 `BriefMessageCard.tsx` 组件，区分 `normal`（灰色边框）和 `proactive`（蓝色边框）模式
- [x] 4.4 在 `MessageStream.tsx` 的渲染逻辑中处理 `"brief"` 类型的 StreamItem
- [ ] 4.5 验证：通过 agent 的 `send_user_message` 工具发送消息，确认 UI 正确展示

## 5. Mid-turn Steer

- [x] 5.1 在 `transport.ts` 新增 `chatSteer(sessionId, messages)` 函数，发送 `chat.steer` WS 消息
- [x] 5.2 修改 `useMessageStreamChat.ts` 的 `handleMentionSend`：当 `streaming === true` 时调用 `chatSteer` 而非 `chatSend`
- [x] 5.3 修改 `StreamFooter.tsx`：streaming 时 placeholder 变为 "追加指令..."，发送按钮样式调整
- [x] 5.4 Steer 消息在 chat stream 中显示时带 "追加" 徽章区分
- [x] 5.5 验证：agent 执行长任务时输入追加指令，确认 agent 收到并响应

## 6. 反馈闭环

- [x] 6.1 在 `transport.ts` 新增 `submitFeedback(sessionId, turnId, rating)` 函数，调用 Evolution API
- [x] 6.2 修改 `AiReactionBar`（或 MessageRenderer 内的反馈按钮）：点赞/踩调用 `submitFeedback` 并更新 UI 状态
- [x] 6.3 在 `transport.ts` 新增 `retryTurn(sessionId, turnId)` 函数
- [x] 6.4 修改 Retry 按钮的 onClick handler：调用 `retryTurn` 并触发新的 streaming
- [x] 6.5 在 `ToolOrchestrator` 中接入 `DenialTracker`：用户 Deny 后记录操作类型，后续同类操作自动 Deny
- [ ] 6.6 验证：点赞/踩后检查后端 Evolution 日志；拒绝操作后 agent 重试同类操作应自动被拒

## 7. Pending 审批系统通知

- [x] 7.1 在 `useMessageStreamChat.ts` 中添加窗口焦点检测（`document.hasFocus()`）
- [x] 7.2 当 `approval_required` 或 `ask_question` 事件到达且窗口不在焦点时，通过 `@tauri-apps/plugin-notification` 发送系统通知
- [x] 7.3 点击通知时通过 Tauri event 唤起窗口并定位到对应会话
- [x] 7.4 添加托盘图标状态：有 pending interaction 时显示提醒徽章
- [ ] 7.5 验证：切到其他应用，让 agent 触发审批，确认系统通知弹出并能点击回到应用

## 8. QuickActionBar 接通

- [x] 8.1 修改 `QuickActionBar.tsx`：将 `console.log` 替换为调用 `transport.chatSend()` 发送消息到 Gateway
- [x] 8.2 发送后自动创建新会话或使用当前活跃会话，然后显示主窗口
- [ ] 8.3 验证：通过全局快捷键唤出 QuickActionBar，输入消息，确认进入正常对话流
