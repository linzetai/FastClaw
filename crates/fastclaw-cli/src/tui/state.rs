use std::time::Instant;

use serde_json::Value;
use unicode_width::UnicodeWidthChar;

// ── Slash commands ──────────────────────────────────────────────────

/// Grouped slash commands for display in /help and tab completion.
/// Format: (command, description). Aliases are handled in commands.rs.
pub(crate) const SLASH_COMMANDS: &[(&str, &str)] = &[
    // ── Session ──
    ("/clear", "Clear conversation history and free up context"),
    ("/resume", "Resume a previous conversation"),
    ("/branch", "Create a branch of the current conversation"),
    ("/rename", "Rename the current conversation"),
    ("/export", "Export the current conversation to a file"),
    // ── Model & Agent ──
    ("/model", "Set the AI model"),
    ("/models", "List all available models"),
    ("/agent", "Switch agent: /agent <id>"),
    ("/agents", "List available agents"),
    // ── Context ──
    ("/context", "Show context window usage"),
    ("/compact", "Summarize and compact context"),
    ("/files", "List all files currently in context"),
    // ── Development ──
    ("/diff", "View uncommitted changes and per-turn diffs"),
    ("/undo", "Undo last edit (revert file)"),
    ("/plan", "Toggle Plan/Agent mode"),
    ("/todo", "Show current todo list"),
    ("/skills", "List available skills"),
    ("/hooks", "View hook configurations"),
    // ── System ──
    ("/doctor", "Diagnose installation and settings"),
    ("/mcp", "Manage MCP servers"),
    ("/permissions", "Manage tool permission rules"),
    ("/status", "Show connection and agent status"),
    ("/config", "Show current configuration"),
    ("/cost", "Show the total cost and duration of the session"),
    ("/stats", "Show session token/time stats"),
    // ── Other ──
    ("/help", "Show help and available commands"),
    ("/ping", "Ping gateway for latency"),
    ("/copy", "Copy last response to clipboard"),
    ("/memory", "Search agent memory: /memory <query>"),
    ("/bug", "Send feedback or report a bug"),
    ("/cancel", "Cancel current streaming"),
    ("/add-dir", "Add a new working directory"),
    ("/exit", "Exit the REPL"),
];

/// Alias mappings: (alias, canonical_command)
pub(crate) const COMMAND_ALIASES: &[(&str, &str)] = &[
    ("/quit", "/exit"),
    ("/reset", "/clear"),
    ("/new", "/clear"),
    ("/continue", "/resume"),
    ("/settings", "/config"),
    ("/feedback", "/bug"),
    ("/sessions", "/resume"),
    ("/rewind", "/undo"),
];

// ── Spinner ──────────────────────────────────────────────────────────

pub(crate) const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

const DOT_FRAMES: &[&str] = &[".", "..", "...", ".."];

pub(crate) struct SpinnerState {
    pub(crate) frame: usize,
    pub(crate) verb: String,
    pub(crate) started_at: Instant,
    pub(crate) tool_name: Option<String>,
}

impl SpinnerState {
    pub(crate) fn new() -> Self {
        Self {
            frame: 0,
            verb: "thinking".into(),
            started_at: Instant::now(),
            tool_name: None,
        }
    }

    pub(crate) fn tick(&mut self) {
        self.frame = (self.frame + 1) % SPINNER_FRAMES.len();
    }

    pub(crate) fn display(&self) -> String {
        let dots_idx = (self.frame / 3) % DOT_FRAMES.len();
        let dots = DOT_FRAMES[dots_idx];
        let elapsed = self.started_at.elapsed().as_secs();
        let time = if elapsed >= 3 {
            format!(" {elapsed}s")
        } else {
            String::new()
        };
        let verb = if let Some(ref tool) = self.tool_name {
            tool.as_str()
        } else {
            self.verb.as_str()
        };
        format!("{verb}{dots}{time}")
    }

    pub(crate) fn set_thinking(&mut self) {
        self.verb = "thinking".into();
        self.tool_name = None;
        self.started_at = Instant::now();
    }

    pub(crate) fn set_tool(&mut self, name: &str) {
        let verb = match name {
            "file_read" | "read_file" => "reading",
            "file_write" | "write_file" | "edit_file" | "multi_edit" => "editing",
            "file_search" | "search_in_files" | "glob" | "list_directory" => "searching",
            "shell_exec" | "shell" => "running command",
            "web_search" => "searching web",
            "web_fetch" | "http_fetch" => "fetching",
            "todo_write" => "updating tasks",
            "memory_search" | "memory_store" => "accessing memory",
            "enter_plan_mode" | "exit_plan_mode" => "switching mode",
            _ => "running tool",
        };
        self.verb = verb.into();
        self.tool_name = Some(name.to_string());
        self.started_at = Instant::now();
    }
}

// ── Data types ──────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ChatMsg {
    pub(crate) role: String,
    pub(crate) content: String,
    pub(crate) timestamp: String,
}

#[derive(Debug)]
pub(crate) struct AgentInfo {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) model: String,
}

pub struct TuiApp {
    pub(crate) input: String,
    pub(crate) cursor_pos: usize,
    pub(crate) messages: Vec<ChatMsg>,
    pub(crate) scroll_offset: u16,
    pub(crate) status: String,
    pub(crate) connected: bool,
    pub(crate) streaming: bool,
    pub(crate) session_id: Option<String>,
    pub(crate) agent_id: String,
    pub(crate) agents: Vec<AgentInfo>,
    pub(crate) ws_url: String,
    pub(crate) api_key: Option<String>,
    pub(crate) should_quit: bool,
    pub(crate) req_counter: u64,
    pub(crate) input_history: Vec<String>,
    pub(crate) history_index: Option<usize>,
    pub(crate) history_stash: String,
    pub(crate) tab_completions: Vec<String>,
    pub(crate) tab_index: usize,
    pub(crate) tab_prefix: String,
    pub(crate) show_popup: Option<PopupKind>,

    pub(crate) chat_start_time: Option<Instant>,
    pub(crate) last_elapsed_ms: Option<u64>,
    pub(crate) last_input_tokens: Option<u64>,
    pub(crate) last_output_tokens: Option<u64>,
    pub(crate) total_input_tokens: u64,
    pub(crate) total_output_tokens: u64,
    pub(crate) total_messages: u32,
    pub(crate) total_elapsed_ms: u64,

    pub(crate) work_dir: Option<String>,

    pub(crate) last_request_id: Option<String>,

    pub(crate) config_mode: fastclaw_core::config::ConfigMode,

    pub(crate) execution_mode: String,

    pub(crate) plan_file_path: Option<String>,
    pub(crate) plan_file_exists: bool,

    pub(crate) spinner: SpinnerState,

    pub(crate) ctx_used_tokens: u32,
    pub(crate) ctx_limit_tokens: u32,

    pub(crate) last_esc_at: Option<Instant>,

    pub(crate) stashed_input: Option<(String, usize)>,

    pub(crate) timeout_warned: bool,

    pub current_model: String,
    pub(crate) model_override: String,
    pub(crate) models_cache: Vec<(String, String)>,

    // History search state (Ctrl+R)
    pub(crate) history_search_active: bool,
    pub(crate) history_search_query: String,
    pub(crate) history_search_index: Option<usize>,

    // Thinking block state
    pub(crate) thinking_content: String,
    pub(crate) thinking_collapsed: bool,

    // Interactive select (model picker, etc.)
    pub(crate) select_state: Option<SelectState>,

    // Fulltext search (Ctrl+O)
    pub(crate) search_active: bool,
    pub(crate) search_query: String,
    pub(crate) search_matches: Vec<usize>,
    pub(crate) search_current: usize,
}

#[derive(Clone, Debug)]
pub(crate) enum PopupKind {
    Help,
    Agents,
    Sessions(Vec<Value>),
    AskQuestion {
        request_id: String,
        question: String,
        options: Vec<(String, String)>,
    },
    ModelPicker,
}

/// Generic interactive select state
pub(crate) struct SelectState {
    pub(crate) items: Vec<SelectItem>,
    pub(crate) selected: usize,
    pub(crate) filter: String,
    pub(crate) filtered_indices: Vec<usize>,
}

#[derive(Clone, Debug)]
pub(crate) struct SelectItem {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) description: String,
    pub(crate) is_current: bool,
}

impl SelectState {
    pub(crate) fn new(items: Vec<SelectItem>) -> Self {
        let filtered_indices: Vec<usize> = (0..items.len()).collect();
        Self {
            items,
            selected: 0,
            filter: String::new(),
            filtered_indices,
        }
    }

    pub(crate) fn move_up(&mut self) {
        if !self.filtered_indices.is_empty() && self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub(crate) fn move_down(&mut self) {
        if !self.filtered_indices.is_empty() && self.selected < self.filtered_indices.len() - 1 {
            self.selected += 1;
        }
    }

    pub(crate) fn apply_filter(&mut self) {
        let q = self.filter.to_lowercase();
        self.filtered_indices = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                q.is_empty()
                    || item.label.to_lowercase().contains(&q)
                    || item.id.to_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .collect();
        if self.selected >= self.filtered_indices.len() {
            self.selected = 0;
        }
    }

    pub(crate) fn selected_item(&self) -> Option<&SelectItem> {
        self.filtered_indices
            .get(self.selected)
            .and_then(|&i| self.items.get(i))
    }
}

impl TuiApp {
    pub fn new(ws_url: String, api_key: Option<String>, session_id: Option<String>) -> Self {
        Self {
            input: String::new(),
            cursor_pos: 0,
            messages: Vec::new(),
            scroll_offset: 0,
            status: "Connecting...".into(),
            connected: false,
            streaming: false,
            session_id,
            agent_id: "default".into(),
            agents: Vec::new(),
            ws_url,
            api_key,
            should_quit: false,
            req_counter: 0,
            input_history: Vec::new(),
            history_index: None,
            history_stash: String::new(),
            tab_completions: Vec::new(),
            tab_index: 0,
            tab_prefix: String::new(),
            show_popup: None,
            chat_start_time: None,
            last_elapsed_ms: None,
            last_input_tokens: None,
            last_output_tokens: None,
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_messages: 0,
            total_elapsed_ms: 0,
            work_dir: None,
            last_request_id: None,
            config_mode: fastclaw_core::config::ConfigMode::Production,
            execution_mode: "agent".into(),
            plan_file_path: None,
            plan_file_exists: false,
            spinner: SpinnerState::new(),
            ctx_used_tokens: 0,
            ctx_limit_tokens: 0,
            last_esc_at: None,
            stashed_input: None,
            timeout_warned: false,
            current_model: String::new(),
            model_override: String::new(),
            models_cache: Vec::new(),
            history_search_active: false,
            history_search_query: String::new(),
            history_search_index: None,
            thinking_content: String::new(),
            thinking_collapsed: true,
            select_state: None,
            search_active: false,
            search_query: String::new(),
            search_matches: Vec::new(),
            search_current: 0,
        }
    }

    pub(crate) fn next_id(&mut self) -> String {
        self.req_counter += 1;
        format!("tui-{}", self.req_counter)
    }

    pub(crate) fn push_system(&mut self, content: String) {
        self.messages.push(ChatMsg {
            role: "system".into(),
            content,
            timestamp: now_hms(),
        });
    }

    pub(crate) fn push_history(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        if self.input_history.last().map(String::as_str) != Some(text) {
            self.input_history.push(text.to_string());
        }
        self.history_index = None;
    }

    pub(crate) fn reset_tab(&mut self) {
        self.tab_completions.clear();
        self.tab_index = 0;
        self.tab_prefix.clear();
    }
}

pub(crate) fn now_hms() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

pub(crate) fn format_elapsed(ms: u64) -> String {
    if ms < 1000 {
        format!("{ms}ms")
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        let secs = ms / 1000;
        format!("{}m{}s", secs / 60, secs % 60)
    }
}

/// Convert a char index to byte offset in a string.
pub(crate) fn char_to_byte(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(s.len())
}

/// Display width of the first `char_count` characters (CJK chars = 2 columns).
pub(crate) fn display_width_chars(s: &str, char_count: usize) -> usize {
    s.chars()
        .take(char_count)
        .map(|c| UnicodeWidthChar::width(c).unwrap_or(0))
        .sum()
}
