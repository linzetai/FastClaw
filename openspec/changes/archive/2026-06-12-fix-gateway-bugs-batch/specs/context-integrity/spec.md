## Context Integrity

### Bug 12: response_language 被硬编码 None

**现状**: `chat_pipeline.rs:216` 构造 `enriched_request` 时将 `response_language` 硬编码为 `None`，丢弃了用户通过 WS 传入的语言偏好。

**要求**:
- [ ] `enriched_request.response_language` 必须使用 `request.response_language.clone()`
- [ ] 用户设置的 response_language 应能传递到 agent runtime

### Bug 16: ensure_system_messages 只比数量不比内容

**现状**: `reactive.rs` 中 `ensure_system_messages` 仅检查 system 消息数量是否 ≥ 原始数量。如果压缩器生成了等量但内容不同的 system 消息，原始 system prompt 会丢失。

**要求**:
- [ ] 必须按 content 做相等性检查，逐条验证原始 system 消息是否存在
- [ ] 缺失的原始 system 消息需重新插入到 compacted 结果前端
- [ ] 性能：system 消息通常 ≤ 3 条，O(n*m) 比较可接受

### Bug 17: handleRecallLastMessage 不处理 multimodal content

**现状**: 前端 `StreamFooter.tsx` 中 `handleRecallLastMessage` 直接返回 `item.data.content`。当 content 为数组（multimodal 消息）时，显示为 `[object Object]`。

**要求**:
- [ ] 如果 content 是数组，提取第一个 `type === "text"` 条目的 `.text` 值
- [ ] 如果没有文本条目，返回空字符串
- [ ] 非数组 content 保持原有行为不变

### Bug 19: MCP prompt 中 `&name[4..]` 可能 panic

**现状**: `chat_pipeline.rs:657` 用 `&name[4..]` 裸切片，假设所有 MCP 工具名以 "mcp_" 开头且长度 ≥ 4。

**要求**:
- [ ] 使用 `name.strip_prefix("mcp_").unwrap_or(name)` 替代裸切片
- [ ] 不满足前缀假设时不应 panic，应优雅降级
