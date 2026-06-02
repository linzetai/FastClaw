## Why

XiaoLin 的文档（README.md、docs/MANUAL.md）与代码严重脱节，存在多个 P0 级别的错误：文档引用的 `xiaolin` CLI 二进制不存在、配置 schema 50%+ 字段与 `XiaoLinConfig` 不匹配、WebSocket 协议文档描述错误。Dockerfile 和 K8s manifest 依赖不存在的 CLI，无法工作。`config/default.json` 包含开发者硬编码路径和过时字段。这些问题严重阻碍新用户上手和运维部署。

## What Changes

- **删除** README 和 MANUAL 中所有 `xiaolin` CLI 相关文档（`xiaolin setup`、`serve`、`tui`、`mcp-server`、`doctor` 等）
- **重写** MANUAL §5 配置章节，从 `XiaoLinConfig` 结构体和实际 `config.rs` 代码生成准确文档
- **修复** MANUAL §7 WebSocket 协议文档，改为正确的 `xiaolin-ws/2` JSON-RPC `method + params + id` 格式
- **修复** Dockerfile：移除 `--bin xiaolin` 构建，改为 gateway library embedding 或标记为 WIP
- **修复** K8s deployment.yaml：端口、环境变量、挂载路径对齐
- **清理** `config/default.json`：移除硬编码 `agentsDir` 路径、过时的 `session.dbPath`/`memory.vectorIndexPath`/`metrics`/`plugins` 字段
- **统一** 版本号（0.1.0）和默认端口（18789 vs 18888 取一个）
- **更新** README 项目结构、API 路由表、工具列表，补齐缺失的路由和工具

## Capabilities

### New Capabilities

_无新增功能性能力。_

### Modified Capabilities

_无 spec 级别的行为变更。本 change 仅涉及文档和配置文件的准确性修复。_

## Impact

- **文档**：README.md、docs/MANUAL.md 大幅重写
- **配置**：config/default.json 字段清理（可能影响依赖过时字段的现有部署）
- **构建**：Dockerfile、docker-compose.yml、deploy/kubernetes/deployment.yaml 修改
- **CI**：可能需要新增 Docker build 冒烟测试
- **无代码变更**：不涉及 Rust/TS 源码修改
