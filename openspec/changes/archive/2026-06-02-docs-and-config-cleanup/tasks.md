## 1. config/default.json 清理

- [x] 1.1 移除 `paths.agentsDir` 中的开发者硬编码路径，改为相对路径或注释说明
- [x] 1.2 移除过时字段：`session.dbPath`、`session.maxMessagesPerSession`、`session.compressionThresholdTokens`
- [x] 1.3 移除过时字段：`memory.vectorIndexPath`、`memory.knowledgeGraphPath`、`memory.forgetting.*`
- [x] 1.4 移除 `metrics` 和 `plugins` 顶层 key（引起 unknown key 警告）
- [x] 1.5 将 `gateway.port` 从 18888 统一为 18789（与代码默认值一致）
- [x] 1.6 验证清理后的 config 可被 `XiaoLinConfig` 正确反序列化（写测试或手动验证启动）

## 2. README.md 修正

- [x] 2.1 将快速上手入口从 CLI 改为 Tauri 桌面应用（`cargo tauri dev`）
- [x] 2.2 将 CLI 相关章节（`xiaolin setup/serve/tui/mcp-server/doctor` 等）标记为"计划中"
- [x] 2.3 更新项目结构表：补齐缺失的 crate（sandbox, execpolicy, guardian, hardening, linux-sandbox, network-proxy, path, protocol, session-actor, wechat）
- [x] 2.4 更新 API 路由表：补齐 skills, channels, subagents, llm-plugins, wechat 等路由
- [x] 2.5 更新内置工具列表：对齐 `register_builtin_tools` 实际注册的工具
- [x] 2.6 修正 git clone URL 从 placeholder 改为实际仓库地址
- [x] 2.7 统一版本号为 0.1.0

## 3. MANUAL.md 配置章节重写

- [x] 3.1 从 `XiaoLinConfig` 结构体（config.rs）导出所有实际字段和类型，逐节重写 §5
- [x] 3.2 修正 `session.dmScope` 枚举值为 `main`, `per-peer`, `per-channel-peer`, `per-account-channel-peer`
- [x] 3.3 修正 `gateway.rateLimit` 字段为 `enabled`, `maxRequests`, `windowSecs`
- [x] 3.4 修正 `modelRouter` 字段为 `strategy`, `fallbackChain`, `dailyBudget`
- [x] 3.5 移除 WASM 插件章节，改写为 LLM/Channel process plugin 模型
- [x] 3.6 更新版本号从 0.0.5 到 0.1.0

## 4. MANUAL.md WebSocket 协议修正

- [x] 4.1 重写 §7 WebSocket 协议为 `xiaolin-ws/2` JSON-RPC 格式（method + params + id）
- [x] 4.2 更新事件类型：stream, turn_end, ask_question, approval_required 等
- [x] 4.3 添加连接握手示例（protocol header、auth）

## 5. Docker / K8s 处理

- [x] 5.1 Dockerfile 标记为 WIP，注释原有 `--bin xiaolin`，添加说明
- [x] 5.2 docker-compose.yml 标记为 WIP，添加 `XIAOLIN_STATE_DIR` 环境变量
- [x] 5.3 K8s deployment.yaml 标记为 WIP，注释说明端口和路径需要对齐
- [x] 5.4 README 中明确说明当前推荐运行方式为桌面应用

## 6. 验证

- [x] 6.1 全文搜索文档中引用的文件路径、函数名、配置字段，确认在代码中存在
- [x] 6.2 确认 `cargo check --workspace` 仍通过
