## Context

XiaoLin 前端当前采用固定三栏布局：NavRail(54px) + SessionList(240px) + 主内容区。主内容区顶部有两层 header（ContentHeader 44px + ChatTabsBar），消息区内容限制在 720px 宽。SubAgentMonitor 以固定 280px 右侧面板呈现。窗口缩放时无响应式适配，代码块没有行号。

前端技术栈：React 19 + Zustand + TailwindCSS v4 + TanStack Virtual，运行在 Tauri v2 WebView 中。

## Goals / Non-Goals

**Goals:**
- 提升信息密度：消除冗余 header，让更多垂直空间给消息内容
- 提升操作效率：侧边栏可调宽度适应不同使用习惯，多 Tab 一目了然
- 适配不同窗口尺寸：紧凑模式在小窗口下可用，宽屏模式充分利用空间
- SubAgentMonitor 按需显示，不浪费常态下的水平空间
- 代码块行号提升代码阅读体验

**Non-Goals:**
- 不做移动端适配（Tauri mobile 不在本次范围）
- 不重构 NavRail 的导航项结构，仅调整其折叠行为
- 不改变 TitleBar 的结构和窗口控件布局
- 不修改 StreamFooter（输入区）的布局
- 不引入新的 UI 库或组件框架

## Decisions

### D1: 侧边栏拖拽实现方案

**选择**：纯 CSS + React state + pointer event 方案

在 SessionList 右侧边缘渲染一个 4px 宽的透明拖拽手柄。通过 `onPointerDown` → `onPointerMove`（全局）→ `onPointerUp` 控制拖拽，`useUIStore.sidebarWidth` 持久化。折叠状态改用 `width: 0` + `overflow: hidden`，不再用 `opacity: 0`。

**替代方案**：
- `react-resizable-panels` 库：增加 9KB bundle、额外依赖。拖拽逻辑简单，无需引入库。
- CSS `resize` 属性：浏览器原生，但样式不可控且 WebView 兼容性差。

**范围限制**：`minWidth: 180px, maxWidth: 400px, defaultWidth: 240px`。

### D2: ContentHeader 与 ChatTabsBar 合并

**选择**：删除 ContentHeader 组件，将其功能（侧边栏展开按钮、搜索按钮）移入 ChatTabsBar

合并后的 header 结构：
```
[ 展开侧边栏按钮(仅collapsed时) | Tab1 Tab2 Tab3 ... | + 新建 | 搜索 ]
```

高度从 44px 降为 36px，节省 52px 垂直空间（原 ContentHeader 44px + ChatTabsBar 内嵌在 MessageStream 中的额外间距）。

**替代方案**：
- 保留 ContentHeader 但减少高度：仍有冗余，不够干净。

### D3: Tab 栏恢复横排

**选择**：恢复为横向可滚动 Tab 栏

每个 Tab 包含：关闭按钮 + 标题 + streaming/attention 指示器。溢出时显示左右滚动箭头。使用 `scrollIntoView` 确保激活的 Tab 可见。拖拽排序通过 HTML5 drag-and-drop API 实现。

**替代方案**：
- 保留下拉但增加预览：无法一览所有 Tab 的状态。
- Chrome 风格的 Tab 形状：实现复杂，与当前设计语言不匹配。

### D4: SubAgentMonitor 改为浮动面板

**选择**：底部抽屉（drawer）模式

无活跃子智能体时完全不渲染。有活跃子智能体时，从底部滑入一个可折叠的面板（高度 200-300px），叠加在消息区上方（`position: absolute; bottom: StreamFooter高度`）。用户可拖拽调整高度或点击折叠为一行摘要栏。

**替代方案**：
- 右侧 overlay 面板：挡住消息区右侧内容，阅读体验差。
- Toast 通知风格：信息量不够，无法展示工具调用详情。
- 保持右侧固定但缩窄到 200px：仍然常态占空间。

### D5: 响应式断点

**选择**：基于容器宽度（非 viewport）的三级适配

| 等级 | 主内容区宽度 | 消息 maxWidth | NavRail | SessionList |
|------|-------------|--------------|---------|-------------|
| 紧凑 | < 600px | 100% - 32px | 折叠为 0 | 折叠为 0 |
| 标准 | 600-900px | 720px | 54px | 用户设定 |
| 宽屏 | > 900px | min(860px, 100%-40px) | 54px | 用户设定 |

使用 `ResizeObserver` 监听主内容区宽度变化。紧凑模式下 NavRail 折叠为图标浮动按钮（hamburger menu），SessionList 以 overlay 呈现。

**替代方案**：
- CSS media query：检测的是 viewport 而非内容区实际宽度，不准确。
- 不做响应式：桌面应用也常被调成小窗口，体验差。

### D6: 代码块行号

**选择**：CSS counter + `::before` 伪元素方案

在 `<pre>` 内部的每个 `<code>` 中，将内容按行分割为 `<span class="code-line">` 元素。使用 CSS counter 自动编号，通过 `::before` 渲染行号 gutter。行号列宽固定 40px，颜色使用 `var(--fill-quaternary)`。

通过 `useConfigStore.display.showLineNumbers` 控制开关，默认开启。

**替代方案**：
- 使用 `rehype-line-numbers` 插件：额外依赖，但实际实现类似。
- 表格布局（行号列 + 代码列）：复制时会带出行号文本。

## Risks / Trade-offs

- **[Tab 栏溢出]** → 当打开 10+ 个 Tab 时可能拥挤。缓解：超过 8 个时在末尾显示 `+N` 溢出指示器，点击展开下拉列表
- **[拖拽手柄精度]** → 4px 的手柄可能不好点中。缓解：hover 时扩展到 8px hit area，使用 `cursor: col-resize` 提示
- **[响应式阈值]** → 阈值可能不适合所有用户。缓解：基于内容区实际宽度而非窗口宽度，并在未来可设置
- **[代码块行号复制]** → 用户复制代码时不应包含行号。缓解：行号使用 `::before` 伪元素 + `user-select: none`
- **[SubAgentMonitor overlay]** → 浮动面板可能遮挡消息底部。缓解：面板有折叠为摘要栏的能力，且高度可调
- **[ChatTabsBar 测试]** → 现有 `ChatTabsBar.test.tsx` 需要完全重写适配新 UI。缓解：新测试覆盖横排 Tab 的核心交互
