## 1. ToolExposure 枚举 + Tool trait 扩展

- [ ] 1.1 在 `fastclaw-core/src/tool.rs` 中定义 `ToolExposure` 枚举（Direct / Deferred）
- [ ] 1.2 为 `Tool` trait 增加 `fn exposure(&self) -> ToolExposure` 默认方法（默认 Direct）
- [ ] 1.3 `ExitPlanModeTool` 和 `EnterPlanModeTool` override `exposure()` 返回 Deferred
- [ ] 1.4 `ToolSearchTool` 等现有 deferred 工具改用 `exposure()` 自声明

## 2. ToolProfile + mode-aware 工具提升

- [ ] 2.1 定义 `ToolProfile` struct（promote: Vec<String>, demote: Vec<String>）
- [ ] 2.2 实现 `ToolProfile::for_mode(ExecutionMode)` 预定义 profile
- [ ] 2.3 在 `ToolRegistry` 中增加 `definitions_with_profile(&ToolProfile)` 方法
- [ ] 2.4 重构 `execute_unified` 中的工具收集逻辑，使用 `definitions_with_profile`
- [ ] 2.5 移除 `execute_unified` 中的临时 `activate_deferred("exit_plan_mode")` 调用

## 3. ToolRegistry 内部重构

- [ ] 3.1 将 `deferred: HashSet<String>` 改为基于 `Tool::exposure()` 驱动的过滤
- [ ] 3.2 保留 `register_deferred()` 作为向后兼容方法（内部设置 exposure override）
- [ ] 3.3 确保 `search_deferred()` 仍然正确工作（兼容新的 exposure 机制）
- [ ] 3.4 运行 `cargo clippy -- -D warnings` 确认无警告

## 4. AgentToolsConfig.profile 接入

- [ ] 4.1 定义预置 profile 映射（"plan" / "readonly" / "full"）
- [ ] 4.2 在 `SubAgentManager::build_child_registry_from_def` 中检查并应用 profile
- [ ] 4.3 在子 agent 工具过滤逻辑中，profile 的 demote 列表与 `SubAgentToolFilter.denied` 合并
- [ ] 4.4 更新配置文档说明 `tools.profile` 的可用值

## 5. Mode Attachment 基础设施

- [ ] 5.1 创建 `fastclaw-agent/src/runtime/mode_attachments.rs` 模块
- [ ] 5.2 定义 `ModeAttachment` struct（full_template, sparse_template, turns_between, full_every_n）
- [ ] 5.3 实现 Plan 模式的完整版和简短版 attachment 模板
- [ ] 5.4 实现节流逻辑（turn 计数 + full/sparse 交替）

## 6. ExecutionModeState 增强

- [ ] 6.1 在 `ExecutionModeState` 中增加 `plan_turn_counter: AtomicU32`
- [ ] 6.2 增加 `has_exited_plan: AtomicBool` 用于 reentry 检测
- [ ] 6.3 实现集中的 `transition_mode(from, to)` 方法，更新所有状态字段
- [ ] 6.4 在 `EnterPlanModeTool` 和 `ExitPlanModeTool` 中使用 `transition_mode`

## 7. 注入 Mode Attachment

- [ ] 7.1 在 `execute_unified` 的每轮迭代中调用 `inject_mode_attachment`
- [ ] 7.2 将 attachment 作为 user-role 消息注入到 LLM 请求的消息列表中
- [ ] 7.3 从 `session_guidance_section()` 移除 Plan 模式相关的指令块
- [ ] 7.4 前端 `MessageStream` 组件过滤掉 mode attachment 消息（不展示给用户）

## 8. 集成测试 + 验证

- [ ] 8.1 单元测试：`ToolProfile::for_mode` 返回正确的 promote/demote 列表
- [ ] 8.2 单元测试：`definitions_with_profile` 正确过滤工具
- [ ] 8.3 集成测试：Plan 模式下 exit_plan_mode 出现在工具列表中
- [ ] 8.4 集成测试：attachment 节流逻辑（turn 1 完整、turn 2-5 无、turn 6 完整）
- [ ] 8.5 `cargo clippy -- -D warnings` 全项目零警告
