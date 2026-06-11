# Tasks: Fix Permission Propagation

## Task 1: Live config 读取 (P0)

- [ ] `session_bridge.rs`: `RuntimeTurnExecutor` 新增字段 `live_agents: Option<Arc<ArcSwap<Arc<Vec<AgentConfig>>>>>`
- [ ] `session_bridge.rs`: `effective_behavior()` 增加第 2 层 fallback 读取 `live_agents`
- [ ] `builder.rs`: 创建 `RuntimeTurnExecutor` 时传入 `last_good_agents` 的 `ArcSwap` 引用
- [ ] 验证：修改 Settings → Security 后，已有 session 的下一个 turn 立即使用新配置

## Task 2: SubAgent 继承父级 context (P1)

- [ ] 定义 `SubAgentInheritedContext` struct（work_dir, file_access, additional_allowed_paths）
- [ ] `subagent_manager.rs`: `spawn()` 新增 `inherited_context: Option<SubAgentInheritedContext>` 参数
- [ ] `subagent_manager.rs`: `run_subagent()` 中将 inherited context 应用到 request.work_dir 和 config.behavior
- [ ] `session_bridge.rs`: 调用 `spawn()` 的地方传入当前 session 的 effective behavior
- [ ] 验证：父 session 为 Full mode 时，subagent 的 list_directory/read_file 不再报 path scope 错误

## Task 3: request.work_dir fallback (P2)

- [ ] `session_bridge.rs`: `execute_turn` 中，如果 `request.work_dir` 为 None，从 session store 读取
- [ ] `runtime/mod.rs`: 在 `execute_unified` 入口确认 `request.work_dir` 非空时不覆盖
- [ ] 验证：前端不传 workDir 时，session 存储的 work_dir 自动生效

## Task 4: 回归测试

- [ ] 单元测试：`effective_behavior()` 在 live_agents 更新后返回新值
- [ ] 单元测试：subagent inherited context 正确应用
- [ ] 集成测试：Settings 修改后工具调用使用新 file_access
- [ ] 确认 `cargo clippy -- -D warnings` 通过
