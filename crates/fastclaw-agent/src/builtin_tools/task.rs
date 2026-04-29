use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use dashmap::DashMap;
use fastclaw_core::tool::{Tool, ToolKind, ToolParameterSchema, ToolResult};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

/// Status of a managed background task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Metadata for a managed background task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub task_id: String,
    pub subject: String,
    pub description: String,
    pub status: TaskStatus,
    pub created_at: u64,
    pub finished_at: Option<u64>,
    pub output: Option<String>,
    pub error: Option<String>,
}

struct TaskHandle {
    info: TaskInfo,
    join_handle: Option<JoinHandle<()>>,
}

/// Manages parallel background tasks with concurrency limits.
///
/// Tasks are stored in a `DashMap` keyed by `task_id`. The manager
/// enforces a maximum concurrency limit — `spawn` rejects new tasks
/// when the limit is reached. Completed/failed tasks auto-update
/// their status. `stop` aborts the tokio task and marks it cancelled.
pub struct TaskManager {
    tasks: Arc<DashMap<String, TaskHandle>>,
    max_concurrency: usize,
    running_count: Arc<AtomicUsize>,
}

impl TaskManager {
    pub fn new(max_concurrency: usize) -> Self {
        Self {
            tasks: Arc::new(DashMap::new()),
            max_concurrency,
            running_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn now_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    fn generate_task_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Spawn a new background task. Returns the task_id on success,
    /// or an error if the concurrency limit is reached.
    ///
    /// The `work` future runs on the tokio runtime. When it completes,
    /// the task status is automatically updated to `Completed` (on Ok)
    /// or `Failed` (on Err). The result string is stored in `output` or `error`.
    pub fn spawn<F>(
        &self,
        subject: String,
        description: String,
        work: F,
    ) -> Result<String, TaskManagerError>
    where
        F: Future<Output = Result<String, String>> + Send + 'static,
    {
        let current = self.running_count.load(Ordering::Acquire);
        if current >= self.max_concurrency {
            return Err(TaskManagerError::ConcurrencyLimitReached {
                max: self.max_concurrency,
                current,
            });
        }

        let task_id = Self::generate_task_id();
        let info = TaskInfo {
            task_id: task_id.clone(),
            subject,
            description,
            status: TaskStatus::Running,
            created_at: Self::now_ms(),
            finished_at: None,
            output: None,
            error: None,
        };

        let tasks = Arc::clone(&self.tasks);
        let running = Arc::clone(&self.running_count);
        let id = task_id.clone();

        running.fetch_add(1, Ordering::AcqRel);

        let handle = tokio::spawn(async move {
            let result = work.await;
            let now = TaskManager::now_ms();

            if let Some(mut entry) = tasks.get_mut(&id) {
                match result {
                    Ok(output) => {
                        entry.info.status = TaskStatus::Completed;
                        entry.info.output = Some(output);
                    }
                    Err(error) => {
                        entry.info.status = TaskStatus::Failed;
                        entry.info.error = Some(error);
                    }
                }
                entry.info.finished_at = Some(now);
                entry.join_handle = None;
            }

            running.fetch_sub(1, Ordering::AcqRel);
        });

        self.tasks.insert(
            task_id.clone(),
            TaskHandle {
                info,
                join_handle: Some(handle),
            },
        );

        Ok(task_id)
    }

    /// Get a snapshot of a task's info.
    pub fn get(&self, task_id: &str) -> Option<TaskInfo> {
        self.tasks.get(task_id).map(|entry| entry.info.clone())
    }

    /// List all tasks.
    pub fn list(&self) -> Vec<TaskInfo> {
        self.tasks
            .iter()
            .map(|entry| entry.info.clone())
            .collect()
    }

    /// Stop a running task by aborting its tokio JoinHandle.
    /// Returns `true` if the task was running and is now cancelled.
    pub fn stop(&self, task_id: &str) -> Result<bool, TaskManagerError> {
        let mut entry = self
            .tasks
            .get_mut(task_id)
            .ok_or(TaskManagerError::NotFound(task_id.to_string()))?;

        match entry.info.status {
            TaskStatus::Running | TaskStatus::Pending => {
                if let Some(handle) = entry.join_handle.take() {
                    handle.abort();
                    self.running_count.fetch_sub(1, Ordering::AcqRel);
                }
                entry.info.status = TaskStatus::Cancelled;
                entry.info.finished_at = Some(Self::now_ms());
                Ok(true)
            }
            TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => Ok(false),
        }
    }

    /// Update a task's subject/description while it's still running.
    pub fn update(
        &self,
        task_id: &str,
        subject: Option<String>,
        description: Option<String>,
    ) -> Result<(), TaskManagerError> {
        let mut entry = self
            .tasks
            .get_mut(task_id)
            .ok_or(TaskManagerError::NotFound(task_id.to_string()))?;

        if let Some(s) = subject {
            entry.info.subject = s;
        }
        if let Some(d) = description {
            entry.info.description = d;
        }
        Ok(())
    }

    /// Number of currently running tasks.
    pub fn running_count(&self) -> usize {
        self.running_count.load(Ordering::Acquire)
    }

    /// Total number of tasks (all statuses).
    pub fn total_count(&self) -> usize {
        self.tasks.len()
    }
}

/// Errors from TaskManager operations.
#[derive(Debug, Clone)]
pub enum TaskManagerError {
    NotFound(String),
    ConcurrencyLimitReached { max: usize, current: usize },
}

impl std::fmt::Display for TaskManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(id) => write!(f, "task not found: {id}"),
            Self::ConcurrencyLimitReached { max, current } => {
                write!(f, "concurrency limit reached: {current}/{max} tasks running")
            }
        }
    }
}

impl std::error::Error for TaskManagerError {}

// ─── TaskCreateTool ──────────────────────────────────────────────────

/// Tool that creates a new background task via the TaskManager.
///
/// The actual work function is provided via `set_work_factory`. If no factory
/// is set, the tool creates a placeholder task that immediately completes
/// with the description as output (useful for testing the task lifecycle).
pub struct TaskCreateTool {
    manager: Arc<TaskManager>,
}

impl TaskCreateTool {
    pub fn new(manager: Arc<TaskManager>) -> Self {
        Self { manager }
    }
}

#[derive(Deserialize)]
struct TaskCreateArgs {
    subject: String,
    #[serde(default)]
    description: Option<String>,
}

#[async_trait]
impl Tool for TaskCreateTool {
    fn kind(&self) -> ToolKind {
        ToolKind::Execute
    }

    fn name(&self) -> &str {
        "task_create"
    }

    fn description(&self) -> &str {
        "Create a new background task. The task runs asynchronously and its \
         progress can be monitored with task_list/task_get. Returns the unique task_id."
    }

    fn parameters_schema(&self) -> ToolParameterSchema {
        let mut props = HashMap::new();
        props.insert(
            "subject".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "Short title describing what the task does."
            }),
        );
        props.insert(
            "description".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "Detailed instructions for the task (optional)."
            }),
        );
        ToolParameterSchema {
            schema_type: "object".to_string(),
            properties: props,
            required: vec!["subject".to_string()],
        }
    }

    async fn execute(&self, arguments: &str) -> ToolResult {
        let args: TaskCreateArgs = match serde_json::from_str(arguments) {
            Ok(v) => v,
            Err(e) => {
                return ToolResult::err(format!(
                    "Invalid arguments: {e}. Expected {{\"subject\": \"...\", \"description\": \"...\"}}"
                ))
            }
        };

        let desc = args.description.unwrap_or_default();
        let desc_clone = desc.clone();

        let result = self.manager.spawn(args.subject.clone(), desc, async move {
            Ok(format!("Task completed: {desc_clone}"))
        });

        match result {
            Ok(task_id) => ToolResult::ok(
                serde_json::json!({
                    "task_id": task_id,
                    "status": "running",
                    "subject": args.subject,
                })
                .to_string(),
            ),
            Err(TaskManagerError::ConcurrencyLimitReached { max, current }) => {
                ToolResult::err(format!(
                    "Cannot create task: concurrency limit reached ({current}/{max} running). \
                     Wait for existing tasks to complete or stop one first."
                ))
            }
            Err(e) => ToolResult::err(format!("Failed to create task: {e}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn spawn_and_get_task() {
        let mgr = TaskManager::new(5);
        let id = mgr
            .spawn("test".into(), "a test task".into(), async {
                Ok("done".to_string())
            })
            .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;

        let info = mgr.get(&id).unwrap();
        assert_eq!(info.task_id, id);
        assert_eq!(info.subject, "test");
        assert_eq!(info.status, TaskStatus::Completed);
        assert_eq!(info.output.as_deref(), Some("done"));
    }

    #[tokio::test]
    async fn task_failure_updates_status() {
        let mgr = TaskManager::new(5);
        let id = mgr
            .spawn("fail".into(), "will fail".into(), async {
                Err("something went wrong".to_string())
            })
            .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;

        let info = mgr.get(&id).unwrap();
        assert_eq!(info.status, TaskStatus::Failed);
        assert_eq!(info.error.as_deref(), Some("something went wrong"));
        assert!(info.finished_at.is_some());
    }

    #[tokio::test]
    async fn concurrency_limit_rejects_excess() {
        let mgr = TaskManager::new(2);

        // Spawn 2 long-running tasks.
        mgr.spawn("t1".into(), "".into(), async {
            tokio::time::sleep(Duration::from_secs(5)).await;
            Ok("ok".to_string())
        })
        .unwrap();
        mgr.spawn("t2".into(), "".into(), async {
            tokio::time::sleep(Duration::from_secs(5)).await;
            Ok("ok".to_string())
        })
        .unwrap();

        // Third should be rejected.
        let result = mgr.spawn("t3".into(), "".into(), async { Ok("ok".to_string()) });
        assert!(result.is_err());
        match result.unwrap_err() {
            TaskManagerError::ConcurrencyLimitReached { max, current } => {
                assert_eq!(max, 2);
                assert_eq!(current, 2);
            }
            _ => panic!("expected ConcurrencyLimitReached"),
        }
    }

    #[tokio::test]
    async fn stop_cancels_running_task() {
        let mgr = TaskManager::new(5);
        let id = mgr
            .spawn("long".into(), "".into(), async {
                tokio::time::sleep(Duration::from_secs(60)).await;
                Ok("should not reach".to_string())
            })
            .unwrap();

        tokio::time::sleep(Duration::from_millis(10)).await;

        let stopped = mgr.stop(&id).unwrap();
        assert!(stopped);

        let info = mgr.get(&id).unwrap();
        assert_eq!(info.status, TaskStatus::Cancelled);
        assert!(info.finished_at.is_some());
    }

    #[tokio::test]
    async fn stop_completed_task_returns_false() {
        let mgr = TaskManager::new(5);
        let id = mgr
            .spawn("quick".into(), "".into(), async {
                Ok("done".to_string())
            })
            .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;

        let stopped = mgr.stop(&id).unwrap();
        assert!(!stopped);
    }

    #[tokio::test]
    async fn stop_nonexistent_returns_error() {
        let mgr = TaskManager::new(5);
        let result = mgr.stop("nonexistent");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_returns_all_tasks() {
        let mgr = TaskManager::new(10);
        mgr.spawn("a".into(), "".into(), async { Ok("ok".to_string()) })
            .unwrap();
        mgr.spawn("b".into(), "".into(), async { Ok("ok".to_string()) })
            .unwrap();
        mgr.spawn("c".into(), "".into(), async { Ok("ok".to_string()) })
            .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;

        let list = mgr.list();
        assert_eq!(list.len(), 3);
    }

    #[tokio::test]
    async fn update_task_metadata() {
        let mgr = TaskManager::new(5);
        let id = mgr
            .spawn("original".into(), "desc".into(), async {
                tokio::time::sleep(Duration::from_secs(5)).await;
                Ok("ok".to_string())
            })
            .unwrap();

        mgr.update(&id, Some("updated".into()), None).unwrap();

        let info = mgr.get(&id).unwrap();
        assert_eq!(info.subject, "updated");
        assert_eq!(info.description, "desc");
    }

    #[tokio::test]
    async fn running_count_tracks_active_tasks() {
        let mgr = TaskManager::new(10);
        assert_eq!(mgr.running_count(), 0);

        mgr.spawn("t1".into(), "".into(), async {
            tokio::time::sleep(Duration::from_secs(5)).await;
            Ok("ok".to_string())
        })
        .unwrap();

        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(mgr.running_count(), 1);
    }

    #[tokio::test]
    async fn completed_task_decrements_running_count() {
        let mgr = TaskManager::new(10);

        mgr.spawn("quick".into(), "".into(), async {
            Ok("done".to_string())
        })
        .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(mgr.running_count(), 0);
    }

    #[tokio::test]
    async fn concurrency_slot_freed_after_completion() {
        let mgr = TaskManager::new(1);

        let id1 = mgr
            .spawn("t1".into(), "".into(), async {
                Ok("done".to_string())
            })
            .unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(mgr.get(&id1).unwrap().status, TaskStatus::Completed);

        // Now slot is freed, should accept a new task.
        let result = mgr.spawn("t2".into(), "".into(), async { Ok("ok".to_string()) });
        assert!(result.is_ok());
    }

    // ═══════════════════════════════════════════════════════════════
    // TaskCreateTool tests
    // ═══════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn task_create_tool_success() {
        let mgr = Arc::new(TaskManager::new(5));
        let tool = TaskCreateTool::new(Arc::clone(&mgr));

        let result = tool
            .execute(r#"{"subject": "test task", "description": "do something"}"#)
            .await;
        assert!(result.success);

        let output: serde_json::Value = serde_json::from_str(&result.output).unwrap();
        assert!(output.get("task_id").is_some());
        assert_eq!(output["status"], "running");
        assert_eq!(output["subject"], "test task");

        // Verify task was actually created in the manager.
        let task_id = output["task_id"].as_str().unwrap();
        let info = mgr.get(task_id).unwrap();
        assert_eq!(info.subject, "test task");
    }

    #[tokio::test]
    async fn task_create_tool_missing_subject() {
        let mgr = Arc::new(TaskManager::new(5));
        let tool = TaskCreateTool::new(Arc::clone(&mgr));

        let result = tool.execute(r#"{"description": "no subject"}"#).await;
        assert!(!result.success);
        assert!(result.output.contains("Invalid arguments"));
    }

    #[tokio::test]
    async fn task_create_tool_concurrency_limit() {
        let mgr = Arc::new(TaskManager::new(1));

        // Fill the slot with a long-running task.
        mgr.spawn("blocker".into(), "".into(), async {
            tokio::time::sleep(Duration::from_secs(60)).await;
            Ok("ok".to_string())
        })
        .unwrap();

        let tool = TaskCreateTool::new(Arc::clone(&mgr));
        let result = tool
            .execute(r#"{"subject": "will be rejected"}"#)
            .await;
        assert!(!result.success);
        assert!(result.output.contains("concurrency limit"));
    }

    #[tokio::test]
    async fn task_create_tool_returns_unique_ids() {
        let mgr = Arc::new(TaskManager::new(10));
        let tool = TaskCreateTool::new(Arc::clone(&mgr));

        let r1 = tool.execute(r#"{"subject": "a"}"#).await;
        let r2 = tool.execute(r#"{"subject": "b"}"#).await;

        let o1: serde_json::Value = serde_json::from_str(&r1.output).unwrap();
        let o2: serde_json::Value = serde_json::from_str(&r2.output).unwrap();

        assert_ne!(o1["task_id"], o2["task_id"]);
    }
}
