# 任务列表

## Phase 1: 模型选择器同步（高优 bug）

- [x] 1.1 config-store 新增 models 状态和 refreshModels action
- [x] 1.2 ModelSelector 改为从 store 读取模型列表
- [x] 1.3 ModelTab 保存/删除后调用 refreshModels
- [x] 1.4 ModelSelector 打开下拉时触发 refresh 兜底

## Phase 2: 侧边栏完全折叠

- [x] 2.1 SessionList collapsed 时 width=0 + overflow hidden
- [x] 2.2 ContentHeader 添加展开按钮（仅 collapsed 时显示）
- [x] 2.3 验证动画过渡和边界情况

## Phase 3: 消息交错渲染

- [x] 3.1 Chat 类型增加 segments 可选字段
- [x] 3.2 useMessageStreamChat stream 完成时存 segments 到 chat
- [x] 3.3 AiMessage 优先使用 segments 渲染交错视图
- [x] 3.4 StepGroup 非 streaming 时默认展开
- [x] 3.5 persistence 层适配 segments 序列化

## Phase 4: 风格统一

- [x] 4.1 创建 FormElements 共享组件（Input, Button）
- [x] 4.2 ModelTab 替换内联 input/button 样式为 shared 组件
- [x] 4.3 其他 Settings tabs 批量对齐样式
