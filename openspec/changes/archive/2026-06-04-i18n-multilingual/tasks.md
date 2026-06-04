## 1. i18n 基础设施搭建

- [x] 1.1 安装 react-i18next 和 i18next npm 依赖
- [x] 1.2 创建 `src/i18n/index.ts` — i18next 初始化配置（resources、lng、fallbackLng、ns、defaultNS）
- [x] 1.3 创建翻译文件目录结构 `src/i18n/locales/{zh,en}/` 及初始命名空间 JSON 文件（common、chat、settings、sidebar、header、onboarding、notification）
- [x] 1.4 在 `main.tsx` 中导入 `src/i18n/index.ts` 确保 i18next 初始化
- [x] 1.5 创建 `useLocaleStore`（zustand/persist）— locale、responseLang 字段及 setter，persist 到 `xiaolin-locale`
- [x] 1.6 连接 `useLocaleStore.locale` 与 `i18n.changeLanguage()` — locale 变更时同步切换 i18next 语言

## 2. 核心组件翻译替换

- [x] 2.1 替换 `ComposerCore.tsx` 中的硬编码中文为 t() 调用（chat 命名空间）
- [x] 2.2 替换 `StreamEmptyState.tsx` 中的硬编码中文为 t() 调用（chat 命名空间）
- [x] 2.3 替换 `StreamFooter.tsx` 中的硬编码中文为 t() 调用（chat 命名空间）
- [x] 2.4 替换 `MessageStream.tsx` 中的硬编码中文为 t() 调用（chat 命名空间）
- [x] 2.5 替换 `AppHeader.tsx` 中的硬编码中文为 t() 调用（header 命名空间）
- [x] 2.6 替换 `AppSidebar.tsx` 中的硬编码中文为 t() 调用（sidebar 命名空间）
- [x] 2.7 替换 `AppLayout.tsx` 中的硬编码中文为 t() 调用（common 命名空间）

## 3. 消息流组件翻译替换

- [x] 3.1 替换 `ApprovalCard.tsx`、`PermissionSelector.tsx`、`DiffCard.tsx` 中的硬编码中文
- [x] 3.2 替换 `MessageRenderer.tsx`、`UserInput.tsx`、`MarkdownContent.tsx` 中的硬编码中文
- [x] 3.3 替换 `SubAgentMonitor.tsx`、`SubAgentCard.tsx`、`StepIndicator.tsx`、`StepGroup.tsx` 中的硬编码中文
- [x] 3.4 替换 `PlanApprovalCard.tsx`、`PlanPanel.tsx`、`ToolCallGroup.tsx` 中的硬编码中文
- [x] 3.5 替换 `ToolCallCard.tsx`、`TodoCard.tsx`、`QueueIndicator.tsx`、`QueuePanel.tsx` 中的硬编码中文
- [x] 3.6 替换 `ThinkingIndicator.tsx`、`MentionInput.tsx`、`StickyContextBar.tsx`、`BriefMessageCard.tsx` 中的硬编码中文

## 4. 设置页面翻译替换

- [x] 4.1 替换 `SettingsPanel.tsx`、`GeneralTab.tsx` 中的硬编码中文（settings 命名空间）
- [x] 4.2 替换 `SecurityTab.tsx`、`ModelTab.tsx`、`WebSearchTab.tsx` 中的硬编码中文
- [x] 4.3 替换 `AboutTab.tsx`、`LlmPluginTab.tsx`、`SubAgentsTab.tsx` 中的硬编码中文
- [x] 4.4 替换 `NotificationTab.tsx`、`McpManager.tsx`、`SkillsTab.tsx`、`GatewayTab.tsx`、`MigrationTab.tsx`、`CronTab.tsx` 中的硬编码中文

## 5. 其他组件翻译替换

- [x] 5.1 替换引导页组件 `OnboardingWizard.tsx`、`WelcomeStep.tsx`、`FeaturesStep.tsx`、`DoneStep.tsx` 及 model 子步骤中的硬编码中文
- [x] 5.2 替换 `NotificationCenter.tsx`、`NotificationDetailPanel.tsx` 中的硬编码中文
- [x] 5.3 替换 `VoiceButton.tsx`、`QuickActionBar.tsx`、`ComingSoon.tsx`、`ImageLightbox.tsx`、`ContextMenu.tsx` 中的硬编码中文
- [x] 5.4 替换 `TasksPage.tsx`、`ConnectionsPage.tsx`、`PluginsView.tsx`、`AutomationView.tsx` 中的硬编码中文
- [x] 5.5 替换 `WorkspacePanel.tsx`、`WelcomeView.tsx`、`UpdateBanner.tsx`、`TitleBar.tsx` 中的硬编码中文
- [x] 5.6 替换 `theme.ts` 中主题预设标签（"经典"、"素雅"等）为 i18n key
- [x] 5.7 替换 `model-utils.ts`、`model-registry.ts`、`chat-helpers.ts`、`chat-meta-store.ts`、`useMessageStreamChat.ts` 中的硬编码中文

## 6. 英文翻译文件编写

- [x] 6.1 编写 `en/common.json` 英文翻译
- [x] 6.2 编写 `en/chat.json` 英文翻译
- [x] 6.3 编写 `en/settings.json` 英文翻译
- [x] 6.4 编写 `en/sidebar.json`、`en/header.json`、`en/onboarding.json`、`en/notification.json` 英文翻译

## 7. 语言设置 UI

- [ ] 7.1 在 `GeneralTab.tsx` 添加"语言 / Language"设置分区 — 界面语言选择器
- [ ] 7.2 在 `GeneralTab.tsx` 添加回复语言选择器（中文/英文/跟随界面语言/自动）

## 8. 后端回复语言注入

- [ ] 8.1 前端 `transport.ts` 在 WebSocket 消息中添加 `response_language` 字段，值从 `useLocaleStore` 解析
- [ ] 8.2 后端 `xiaolin-gateway` WebSocket channel 层解析 `response_language` 字段
- [ ] 8.3 后端 `xiaolin-agent` runtime `PromptContext` 构造处从 gateway 传入的 language preference 赋值（替换硬编码 `None`）

## 9. 验证

- [x] 9.1 TypeScript 编译检查 `npx tsc --noEmit` 零错误
- [ ] 9.2 E2E 验证：切换 UI 语言后所有可见文案变更
- [ ] 9.3 E2E 验证：切换回复语言后 LLM 以指定语言回复
