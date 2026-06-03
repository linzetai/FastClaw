## 1. 侧边栏拖拽调整宽度 (resizable-sidebar)

- [x] 1.1 在 `useUIStore` 中新增 `sidebarWidth: number` 状态字段（默认 240），添加 `setSidebarWidth` action，接入 persistence
- [x] 1.2 在 `SessionList.tsx` 右侧渲染 ResizeHandle 组件：4px 透明区域 + hover 时显示分隔线 + `cursor: col-resize`
- [x] 1.3 实现拖拽逻辑：`onPointerDown` 捕获 → 全局 `pointermove` 计算新宽度 → clamp(180, 400) → `pointerup` 释放
- [x] 1.4 将 SessionList 的 `width: 240px` 硬编码替换为 `width: ${sidebarWidth}px`
- [x] 1.5 修改折叠行为：移除 `opacity: 0`，使用 `width: 0` + `overflow: hidden`，移除折叠状态下的展开按钮渲染
- [x] 1.6 实现双击手柄重置宽度到 240px 的交互
- [x] 1.7 验证：拖拽流畅无卡顿、折叠/展开动画正常、宽度在应用重启后保持

## 2. 合并 Header 与恢复横排 Tab (unified-header-bar)

- [x] 2.1 删除 `AppLayout.tsx` 中的 `ContentHeader` 组件定义和渲染
- [x] 2.2 重写 `ChatTabsBar.tsx`：从下拉菜单改为横向可滚动 Tab 栏，高度 36px
- [x] 2.3 实现 Tab 滚动：Tab 容器 `overflow-x: auto; scrollbar-width: none`，两端显示滚动箭头按钮
- [x] 2.4 实现 Tab 溢出指示器：超过 8 个 Tab 时末尾显示 `+N` 按钮，点击展开下拉列表
- [x] 2.5 将侧边栏展开按钮（`PanelLeftOpen`）移入 ChatTabsBar 左侧，仅在 `sidebarCollapsed` 时显示
- [x] 2.6 将搜索按钮移入 ChatTabsBar 右侧
- [x] 2.7 实现 Tab 拖拽排序：HTML5 drag-and-drop API，拖拽时显示放置指示线
- [x] 2.8 实现 Tab 上的 streaming 脉冲圆点和 attention 橙色圆点指示器
- [x] 2.9 实现 Tab 双击重命名（复用现有 rename 逻辑）
- [x] 2.10 更新 `ChatTabsBar.test.tsx` 适配新的横排 Tab UI
- [x] 2.11 验证：Tab 切换、关闭、新建、重命名、拖拽排序全部正常

## 3. SubAgentMonitor 改为浮动面板 (floating-subagent-panel)

- [x] 3.1 修改 `SubAgentMonitor.tsx`：从固定右侧面板改为 `position: absolute` 底部抽屉
- [x] 3.2 实现抽屉进入/退出动画：底部滑入（`slide-up`），退出滑出
- [x] 3.3 实现折叠/展开切换：折叠为 36px 摘要栏显示 `N 个子智能体运行中`，展开显示完整列表
- [x] 3.4 实现抽屉顶部拖拽调整高度：clamp(120, 400)，默认 240px
- [x] 3.5 实现自动收起：所有子智能体完成后 3 秒延迟自动折叠为摘要栏
- [x] 3.6 修改 `AppLayout.tsx`：从 `<SubAgentMonitor />` 在 flex 布局中移除，改为在 MessageStream 内部 absolute 定位
- [x] 3.7 验证：面板不影响消息区宽度、折叠/展开/拖拽高度正常、自动收起正常

## 4. 响应式布局 (responsive-layout)

- [x] 4.1 在 `AppLayout.tsx` 中用 `ResizeObserver` 监听主内容区宽度，计算 `layoutTier: 'compact' | 'standard' | 'wide'`
- [x] 4.2 在 `useUIStore` 中新增 `layoutTier` 状态（只读，由 ResizeObserver 驱动）
- [x] 4.3 实现紧凑模式 NavRail 折叠：`layoutTier === 'compact'` 时 NavRail 不渲染，在 TitleBar 或 header 左侧显示 hamburger 按钮
- [x] 4.4 实现 hamburger 菜单：overlay 形式显示导航项列表，点击后关闭
- [x] 4.5 实现紧凑模式 SessionList overlay：`layoutTier === 'compact'` 时 SessionList 以 overlay + 遮罩呈现，选中会话后自动关闭
- [x] 4.6 修改 `MessageRenderer.tsx` 中 `maxWidth: 720px`：改为根据 `layoutTier` 动态计算 —— compact: `100%-32px`, standard: `720px`, wide: `min(860px, 容器宽度-40px)`
- [x] 4.7 在 `index.css` 中新增响应式相关 CSS 变量：`--content-max-w` 等
- [x] 4.8 验证：拖拽窗口大小时布局自动切换、三个模式的 NavRail/SessionList/消息区表现正确

## 5. 代码块行号 (code-line-numbers)

- [x] 5.1 在 `useConfigStore.display` 中新增 `showLineNumbers: boolean` 配置字段，默认 `true`
- [x] 5.2 修改 `MarkdownContent.tsx` 中的 `PreBlock` 组件：将代码内容按行分割为 `<span class="code-line">` 元素
- [x] 5.3 在 `index.css` 中添加行号样式：CSS counter + `::before` 伪元素 + `user-select: none`
- [x] 5.4 实现行号列宽自适应：根据总行数计算列宽（1-9 行: 32px, 10-99 行: 40px, 100+ 行: 48px）
- [x] 5.5 在设置面板中添加"显示代码行号"开关，绑定 `showLineNumbers`
- [x] 5.6 确保 `StreamingMarkdown` 的代码块也正确显示行号
- [x] 5.7 验证：行号渲染正确、复制代码不包含行号、设置开关生效、streaming 时行号递增正常

## 6. 集成验证

- [x] 6.1 运行 `tsc --noEmit` 确认无类型错误
- [x] 6.2 运行 `vite build` 确认构建成功、检查 bundle 大小无明显增长
- [x] 6.3 运行 `vitest run` 确认所有测试通过（transport.test.ts 预已存在的失败除外）
- [x] 6.4 启动 `cargo tauri dev`，手动验证各功能：侧边栏拖拽、Tab 横排、SubAgent 面板、响应式切换、代码行号
- [x] 6.5 测试暗色主题和各 accent 主题下的视觉表现
