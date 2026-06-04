## Context

XiaoLin 是一个 Tauri v2 桌面应用，前端 React + Zustand，后端 Rust。当前所有用户可见文案（~65 个 TSX/TS 文件，~500+ 行中文）和后端提示词（~30 个 Rust 文件，~580+ 行中文）均为硬编码。后端 PromptEngine 已具备完整的中英双语提示词系统（每个 section 都有 `_en()` / `_zh()` 函数），通过 `PromptContext.language_preference` 切换，但该字段当前硬编码为 `None`，从未从前端传入。

现有持久化机制使用 `zustand/persist` + `localStorage`（主题、UI 状态、侧栏宽度），语言设置可复用同一模式。

## Goals / Non-Goals

**Goals:**
- 前端 UI 支持中文/英文两种语言，用户可在设置中切换，选择持久化
- 用户可控制 LLM 回复语言（中文/英文/跟随 UI 语言/自动），通过提示词注入实现
- 后端系统提示词默认使用英文版本（节省 ~30-50% prompt token）
- 回复语言偏好通过 WebSocket 协议从前端传递到后端
- 翻译文件按命名空间组织，便于后续扩展更多语言

**Non-Goals:**
- 不支持中英文以外的语言（架构预留扩展性但不实现）
- 不国际化后端日志和内部错误消息（仅面向用户的消息）
- 不使用服务端翻译方案（所有翻译在前端完成）
- 不国际化后端系统提示词的内容（保持英文）——仅通过 language_section 控制 LLM 回复语言

## Decisions

### D1: 前端 i18n 库选择 — react-i18next

**选择**: `react-i18next` + `i18next`

**理由**: 社区最成熟的 React i18n 方案，支持命名空间、懒加载、插值、复数等完整功能。虽然只有两种语言，但标准化的 `useTranslation()` hook 比自建方案更规范，未来扩展语言零成本。

**替代方案**:
- 自建 `useI18n()` hook — 更轻量（<2KB），但缺少命名空间、ICU 支持等
- Lingui / Paraglide — 编译时方案，学习成本高，社区较小

### D2: 翻译文件组织 — JSON + 命名空间拆分

**选择**: `src/i18n/locales/{zh,en}/{namespace}.json`

命名空间拆分：
- `common.json` — 通用文案（按钮、状态、错误）
- `chat.json` — 聊天/消息流相关
- `settings.json` — 设置页面
- `sidebar.json` — 侧边栏
- `header.json` — 顶栏
- `onboarding.json` — 引导页
- `notification.json` — 通知中心

**理由**: 按功能模块拆分避免单文件过大，i18next namespace 机制天然支持。JSON 格式是 i18next 默认格式，工具链支持最好。

### D3: 语言设置持久化 — 独立 Zustand Store

**选择**: 新建 `useLocaleStore`（zustand/persist → `localStorage` key: `xiaolin-locale`）

```typescript
type Locale = "zh" | "en";
type ResponseLang = "zh" | "en" | "follow-ui" | "auto";

interface LocaleState {
  locale: Locale;           // UI 界面语言
  responseLang: ResponseLang; // LLM 回复语言
  setLocale: (l: Locale) => void;
  setResponseLang: (r: ResponseLang) => void;
}
```

**理由**: 复用现有 zustand/persist 模式，与 ThemeStore 对齐。独立 store 而非嵌入 UIStore，因为语言设置跨越 UI 和后端两个层面。

### D4: 回复语言传递协议 — WebSocket 连接级参数

**选择**: 在 WebSocket 连接建立时（或首次消息中）传递 `response_language` 字段

前端在 `transport.ts` 的连接/消息中附加 `response_language` 字段。后端 gateway 在 channel 层解析该字段，传递到 `AgentRuntime`，最终设置 `PromptContext.language_preference`。

解析逻辑：
- `"zh"` → `Some("zh-CN")`
- `"en"` → `Some("en")`
- `"follow-ui"` → 根据前端传入的 `locale` 解析
- `"auto"` → `None`（不注入语言指令，模型自行判断）

### D5: 后端提示词默认语言 — 英文

**选择**: `language_preference: None` 时，所有提示词 section 走 `_en()` 版本（当前已如此实现）。

**理由**: 英文提示词比中文节省 ~1,500-2,000 tokens/请求。对 Claude/GPT-4 级别模型，英文提示词不影响中文输出质量，前提是有显式的 language_section 注入回复语言指令。

### D6: i18next 初始化配置

```typescript
i18n
  .use(initReactI18next)
  .init({
    resources: { zh: {...}, en: {...} },
    lng: savedLocale,        // 从 localStorage 读取
    fallbackLng: "zh",       // 降级语言
    ns: ["common", "chat", "settings", ...],
    defaultNS: "common",
    interpolation: { escapeValue: false }, // React 已处理 XSS
  });
```

不使用 `i18next-browser-languagedetector`，因为应用有明确的默认语言（中文）且用户可手动切换。

## Risks / Trade-offs

**[大量文件修改] → 分阶段替换**
~65 个文件需要替换硬编码中文为 `t()` 调用，工作量大且容易引入拼写错误。通过按模块分批替换（先核心组件，后设置/引导页）降低风险，每批提交后验证 TypeScript 编译。

**[翻译遗漏] → 编译时检查**
可能存在遗漏的硬编码文案。初期可接受，后续通过 ESLint 规则（`no-literal-string`）或翻译覆盖率检查工具逐步完善。

**[提示词缓存失效] → 可控影响**
切换 `language_preference` 会导致 PromptEngine 静态 section 缓存失效需重算。由于切换频率很低（用户设置变更），影响可忽略。

**[WebSocket 协议兼容] → 向后兼容**
新增 `response_language` 字段为可选字段，缺失时后端使用默认值（`None` → 英文提示词 + 无语言注入），不影响旧版客户端。
