use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use fastclaw_core::tool::{Tool, ToolKind, ToolParameterSchema, ToolResult};
use fastclaw_core::types::ExecutionMode;

const MODE_AGENT: u8 = 0;
const MODE_PLAN: u8 = 1;

fn mode_from_u8(v: u8) -> ExecutionMode {
    if v == MODE_PLAN {
        ExecutionMode::Plan
    } else {
        ExecutionMode::Agent
    }
}

fn mode_to_u8(m: ExecutionMode) -> u8 {
    match m {
        ExecutionMode::Agent => MODE_AGENT,
        ExecutionMode::Plan => MODE_PLAN,
    }
}

/// Shared execution mode state. Thread-safe via AtomicU8.
///
/// The runtime and tools share this via `Arc`. The tool executor checks
/// `current_mode()` before executing write/edit/execute tools; if the
/// mode is `Plan`, those tools are blocked with a friendly message.
#[derive(Debug, Clone)]
pub struct ExecutionModeState {
    state: Arc<AtomicU8>,
}

impl Default for ExecutionModeState {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionModeState {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicU8::new(MODE_AGENT)),
        }
    }

    pub fn current_mode(&self) -> ExecutionMode {
        mode_from_u8(self.state.load(Ordering::Acquire))
    }

    /// Try to transition to the given mode. Returns `(from, to)`.
    /// If already in the target mode, `from == to`.
    pub fn transition(&self, target: ExecutionMode) -> (ExecutionMode, ExecutionMode) {
        let new_val = mode_to_u8(target);
        let old_val = self.state.swap(new_val, Ordering::AcqRel);
        (mode_from_u8(old_val), target)
    }

    /// Whether the current mode blocks the given tool kind.
    pub fn is_blocked(&self, kind: ToolKind) -> bool {
        if self.current_mode() != ExecutionMode::Plan {
            return false;
        }
        matches!(kind, ToolKind::Edit | ToolKind::Execute)
    }

    /// Human-readable message when a tool is blocked by plan mode.
    pub fn blocked_message(tool_name: &str) -> String {
        format!(
            "Tool '{tool_name}' is blocked in Plan mode (read-only). \
             Use exit_plan_mode to return to Agent mode before making changes."
        )
    }
}

// ─── EnterPlanModeTool ───────────────────────────────────────────────

/// Switches the agent to plan mode (read-only exploration).
/// Write/edit/execute tools are blocked until `exit_plan_mode` is called.
pub struct EnterPlanModeTool {
    mode_state: ExecutionModeState,
}

impl EnterPlanModeTool {
    pub fn new(mode_state: ExecutionModeState) -> Self {
        Self { mode_state }
    }
}

#[async_trait]
impl Tool for EnterPlanModeTool {
    fn kind(&self) -> ToolKind {
        ToolKind::Think
    }

    fn name(&self) -> &str {
        "enter_plan_mode"
    }

    fn description(&self) -> &str {
        "Switch to plan mode for read-only exploration and design. \
         Write/edit/execute tools are blocked until exit_plan_mode is called."
    }

    fn search_hint(&self) -> &str {
        "switch to plan mode design approach before coding"
    }

    fn is_deferred(&self) -> bool {
        true
    }

    fn parameters_schema(&self) -> ToolParameterSchema {
        ToolParameterSchema {
            schema_type: "object".to_string(),
            properties: HashMap::new(),
            required: vec![],
        }
    }

    async fn execute(&self, _arguments: &str) -> ToolResult {
        let (from, _to) = self.mode_state.transition(ExecutionMode::Plan);

        if from == ExecutionMode::Plan {
            return ToolResult::ok("Already in plan mode.");
        }

        ToolResult::ok(format!(
            "Entered plan mode (was: {from}).\n\n\
             In plan mode:\n\
             1. Explore the codebase with read/search tools\n\
             2. Identify patterns and approaches\n\
             3. Design an implementation strategy\n\
             4. Use exit_plan_mode when ready to start coding\n\n\
             DO NOT write or edit any files. This is a read-only phase."
        ))
    }
}

// ─── ExitPlanModeTool ────────────────────────────────────────────────

/// Exits plan mode, restoring full tool access (Agent mode).
pub struct ExitPlanModeTool {
    mode_state: ExecutionModeState,
}

impl ExitPlanModeTool {
    pub fn new(mode_state: ExecutionModeState) -> Self {
        Self { mode_state }
    }
}

#[async_trait]
impl Tool for ExitPlanModeTool {
    fn kind(&self) -> ToolKind {
        ToolKind::Think
    }

    fn name(&self) -> &str {
        "exit_plan_mode"
    }

    fn description(&self) -> &str {
        "Exit plan mode and return to agent mode with full tool access. \
         Call this after designing your approach to start implementation."
    }

    fn search_hint(&self) -> &str {
        "exit plan mode start coding implementation"
    }

    fn is_deferred(&self) -> bool {
        true
    }

    fn parameters_schema(&self) -> ToolParameterSchema {
        ToolParameterSchema {
            schema_type: "object".to_string(),
            properties: HashMap::new(),
            required: vec![],
        }
    }

    async fn execute(&self, _arguments: &str) -> ToolResult {
        let (from, _to) = self.mode_state.transition(ExecutionMode::Agent);

        if from == ExecutionMode::Agent {
            return ToolResult::ok("Already in agent mode.");
        }

        ToolResult::ok(
            "Exited plan mode → agent mode. All tools are now available.\n\
             You can proceed with implementation."
                .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_state_default_is_agent() {
        let state = ExecutionModeState::new();
        assert_eq!(state.current_mode(), ExecutionMode::Agent);
    }

    #[test]
    fn mode_state_transition_to_plan() {
        let state = ExecutionModeState::new();
        let (from, to) = state.transition(ExecutionMode::Plan);
        assert_eq!(from, ExecutionMode::Agent);
        assert_eq!(to, ExecutionMode::Plan);
        assert_eq!(state.current_mode(), ExecutionMode::Plan);
    }

    #[test]
    fn mode_state_transition_back_to_agent() {
        let state = ExecutionModeState::new();
        state.transition(ExecutionMode::Plan);
        let (from, to) = state.transition(ExecutionMode::Agent);
        assert_eq!(from, ExecutionMode::Plan);
        assert_eq!(to, ExecutionMode::Agent);
    }

    #[test]
    fn mode_state_idempotent_transition() {
        let state = ExecutionModeState::new();
        let (from, to) = state.transition(ExecutionMode::Agent);
        assert_eq!(from, ExecutionMode::Agent);
        assert_eq!(to, ExecutionMode::Agent);
    }

    #[test]
    fn is_blocked_in_plan_mode() {
        let state = ExecutionModeState::new();
        state.transition(ExecutionMode::Plan);

        assert!(state.is_blocked(ToolKind::Edit));
        assert!(state.is_blocked(ToolKind::Execute));
        assert!(!state.is_blocked(ToolKind::Read));
        assert!(!state.is_blocked(ToolKind::Search));
        assert!(!state.is_blocked(ToolKind::Fetch));
        assert!(!state.is_blocked(ToolKind::Think));
    }

    #[test]
    fn is_not_blocked_in_agent_mode() {
        let state = ExecutionModeState::new();
        assert!(!state.is_blocked(ToolKind::Edit));
        assert!(!state.is_blocked(ToolKind::Execute));
    }

    #[tokio::test]
    async fn enter_plan_mode_tool() {
        let state = ExecutionModeState::new();
        let tool = EnterPlanModeTool::new(state.clone());

        let result = tool.execute("{}").await;
        assert!(result.success);
        assert!(result.output.contains("Entered plan mode"));
        assert!(result.output.contains("read-only"));
        assert_eq!(state.current_mode(), ExecutionMode::Plan);
    }

    #[tokio::test]
    async fn enter_plan_mode_already_in_plan() {
        let state = ExecutionModeState::new();
        state.transition(ExecutionMode::Plan);
        let tool = EnterPlanModeTool::new(state.clone());

        let result = tool.execute("{}").await;
        assert!(result.success);
        assert!(result.output.contains("Already in plan mode"));
    }

    #[tokio::test]
    async fn exit_plan_mode_tool() {
        let state = ExecutionModeState::new();
        state.transition(ExecutionMode::Plan);
        let tool = ExitPlanModeTool::new(state.clone());

        let result = tool.execute("{}").await;
        assert!(result.success);
        assert!(result.output.contains("agent mode"));
        assert!(result.output.contains("All tools are now available"));
        assert_eq!(state.current_mode(), ExecutionMode::Agent);
    }

    #[tokio::test]
    async fn exit_plan_mode_already_in_agent() {
        let state = ExecutionModeState::new();
        let tool = ExitPlanModeTool::new(state.clone());

        let result = tool.execute("{}").await;
        assert!(result.success);
        assert!(result.output.contains("Already in agent mode"));
    }

    #[tokio::test]
    async fn roundtrip_enter_exit() {
        let state = ExecutionModeState::new();
        let enter = EnterPlanModeTool::new(state.clone());
        let exit = ExitPlanModeTool::new(state.clone());

        assert_eq!(state.current_mode(), ExecutionMode::Agent);

        enter.execute("{}").await;
        assert_eq!(state.current_mode(), ExecutionMode::Plan);
        assert!(state.is_blocked(ToolKind::Edit));

        exit.execute("{}").await;
        assert_eq!(state.current_mode(), ExecutionMode::Agent);
        assert!(!state.is_blocked(ToolKind::Edit));
    }

    #[test]
    fn blocked_message_format() {
        let msg = ExecutionModeState::blocked_message("write_file");
        assert!(msg.contains("write_file"));
        assert!(msg.contains("Plan mode"));
        assert!(msg.contains("exit_plan_mode"));
    }
}
