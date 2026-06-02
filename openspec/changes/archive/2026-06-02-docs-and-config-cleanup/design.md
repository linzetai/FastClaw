## Context

XiaoLin 文档体系由 README.md（项目介绍 + 快速上手）和 docs/MANUAL.md（详细手册）组成。项目经历了快速迭代，CLI 入口被移除但文档未同步，配置结构体多次重构但 MANUAL 的示例未更新，WebSocket 协议从简单 JSON 升级到 JSON-RPC 但文档仍描述旧格式。`config/default.json` 作为配置模板包含了开发者本机路径。

当前工作区只产出两个二进制：`xiaolin-app`（Tauri 桌面应用）和 `xiaolin-linux-sandbox`（Linux 沙箱辅助进程）。所有文档中引用的 `xiaolin` CLI 命令均不可用。

## Goals / Non-Goals

**Goals:**
- README 和 MANUAL 的每一条信息都可验证为正确
- 新用户按文档操作能成功运行项目
- config/default.json 是一个可直接使用的模板（无硬编码路径、无过时字段）
- Docker/K8s 配置至少标记为 WIP 或能正确构建 xiaolin-app

**Non-Goals:**
- 不恢复 CLI 二进制（那是独立的 change）
- 不重写完整的 API 参考文档（只修正已有文档的错误）
- 不修改 Rust/TS 源码

## Decisions

### D1: CLI 文档处理策略

**决定**：将 CLI 相关章节标记为"计划中"而非删除，保留架构愿景。

**理由**：完全删除会丢失设计意图，标记为 WIP 既诚实又保留了未来方向。README 的快速上手部分改为以 Tauri 桌面应用为主入口。

### D2: 配置文档生成方式

**决定**：手动从 `XiaoLinConfig` 及嵌套结构体编写文档，不做代码生成。

**替代方案**：用 `schemars` 自动生成 JSON Schema → 文档。但当前 config 结构体没有 derive `JsonSchema`，引入新依赖超出本 change 范围。

**理由**：一次性手动对齐更快，后续可以考虑自动生成。

### D3: 默认端口统一

**决定**：统一为 `18789`（代码默认值），`config/default.json` 中的 `18888` 改为 `18789`。

**理由**：代码默认值是 18789，文档也写 18789，只有 config/default.json 用了 18888。统一到代码默认值最小化变更。

### D4: Dockerfile 处理

**决定**：将 Dockerfile 改为构建 `xiaolin-gateway` 库作为嵌入式服务的示例，或标记为 WIP。

**理由**：当前 Dockerfile `--bin xiaolin` 会直接构建失败。在 CLI 恢复之前，Docker 部署方式需要重新设计。

## Risks / Trade-offs

- **config/default.json 字段删除** → 已有用户的配置文件如果包含这些字段，启动时可能产生新的 unknown key 警告。风险低：`serde` 的 `deny_unknown_fields` 未启用，只是 warn。
- **文档大幅修改** → 可能引入新的不准确。缓解：逐章节与代码交叉验证。
- **Dockerfile 标记 WIP** → Docker 用户暂时无法使用。缓解：在 README 明确说明当前推荐的运行方式。
