## 1. SessionManager 自动 GC

- [x] 1.1 在 `SessionManager` 上添加 `gc_with_stats()` 方法，返回清理数量
- [x] 1.2 在 `AppState` 上添加 `gc_stale_resources()` 方法，统一清理 DashMap 孤立条目
- [x] 1.3 在 gateway `start_background_tasks()` 中 spawn GC loop（60s 间隔），调用上述两个方法
- [x] 1.4 GC loop 中添加 RSS 内存监控日志

## 2. Streaming 安全阀

- [x] 2.1 在 `ws/chat.rs` 的 `spawn_chat` streaming loop 中添加 `assistant_content` 长度检查（2MB 上限）
- [x] 2.2 添加 turn 最大执行时间限制（600s），超时后 cancel turn
- [x] 2.3 超限时 emit Error event 并保存已有内容

## 3. 内存监控

- [x] 3.1 实现 `get_process_rss_bytes()` 函数（macOS: mach_task_basic_info, Linux: /proc/self/status）
- [x] 3.2 在 GC loop 中调用并根据阈值输出日志
- [x] 3.3 在 /health endpoint 返回内存信息

## 4. 验证

- [x] 4.1 添加 SessionManager GC 单元测试
- [x] 4.2 `cargo check` 无编译错误
- [x] 4.3 手动验证：启动 gateway 后查看 GC 日志输出
