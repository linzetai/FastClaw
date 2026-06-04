## Why

应用当前所有 UI 文案和后端提示词均为中文硬编码，无法服务英文用户。需要引入完整的国际化（i18n）体系：前端使用 react-i18next 管理 UI 翻译，后端提示词默认改为英文以节省 token，同时通过语言偏好注入让 LLM 以用户选择的语言回复。

## What Changes

- 前端引入 `react-i18next` + `i18next`，建立翻译基础设施（初始化、hook、语言检测）
- 创建按命名空间拆分的 JSON 翻译文件（zh/en），覆盖 ~65 个组件文件中的硬编码中文
- 新增 `useLocaleStore`（zustand/persist）持久化用户的界面语言和回复语言偏好
- 设置页面（GeneralTab）添加"界面语言"和"回复语言"两个设置项
- 后端 `PromptContext.language_preference` 从硬编码 `None` 改为从前端传入的偏好值
- WebSocket 连接/消息协议扩展，携带用户的回复语言偏好

## Capabilities

### New Capabilities
- `frontend-i18n`: react-i18next 翻译基础设施、翻译文件组织、组件接入
- `locale-settings`: 界面语言和回复语言的用户设置、持久化、UI 切换
- `response-language-injection`: 前端回复语言偏好通过 WebSocket 传递到后端，注入 PromptContext

### Modified Capabilities
- `chat-input-bar`: 输入框相关文案需从硬编码改为 i18n key
- `app-sidebar`: 侧边栏文案（搜索、右键菜单等）需 i18n 化
- `unified-header-bar`: 顶栏按钮 tooltip 等文案需 i18n 化

## Impact

- **前端**：~65 个 TSX/TS 文件需替换硬编码中文为 `t()` 调用
- **新依赖**：`react-i18next`、`i18next`（前端 npm 包）
- **后端**：`xiaolin-agent` 的 `PromptContext` 构造处需读取语言偏好；`xiaolin-gateway` WebSocket 协议需扩展 language 字段
- **存储**：新增 `localStorage` key `xiaolin-locale`
- **翻译文件**：新增 `src/i18n/` 目录，含 `locales/zh/` 和 `locales/en/` 下的 JSON 文件
