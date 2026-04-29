use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub const DEFAULT_MAX_RESULT_SIZE_CHARS: usize = 50_000;

pub const MAX_TOOL_RESULTS_PER_MESSAGE_CHARS: usize = 200_000;

pub const BYTES_PER_TOKEN: usize = 4;

pub const PREVIEW_SIZE_BYTES: usize = 2000;

pub const PERSISTED_OUTPUT_TAG: &str = "<persisted-output>";
pub const PERSISTED_OUTPUT_CLOSING_TAG: &str = "</persisted-output>";

pub const TOOL_RESULT_CLEARED_MESSAGE: &str = "[Old tool result content cleared]";

const TOOL_RESULTS_SUBDIR: &str = "tool-results";

/// Resolve the effective persistence threshold for a tool.
///
/// - `usize::MAX` = hard opt-out (e.g. ReadFile — persisting its output then
///   reading it back with ReadFile would be circular).
/// - Otherwise: `min(declared, DEFAULT_MAX_RESULT_SIZE_CHARS)`.
pub fn get_persistence_threshold(declared_max_result_size_chars: usize) -> usize {
    if declared_max_result_size_chars == usize::MAX {
        return usize::MAX;
    }
    declared_max_result_size_chars.min(DEFAULT_MAX_RESULT_SIZE_CHARS)
}

pub struct PersistedToolResult {
    pub filepath: PathBuf,
    pub original_size: usize,
    pub preview: String,
    pub has_more: bool,
}

pub struct ToolResultStorage {
    session_dir: PathBuf,
}

impl ToolResultStorage {
    pub fn new(session_dir: PathBuf) -> Self {
        Self { session_dir }
    }

    fn tool_results_dir(&self) -> PathBuf {
        self.session_dir.join(TOOL_RESULTS_SUBDIR)
    }

    fn tool_result_path(&self, tool_use_id: &str) -> PathBuf {
        self.tool_results_dir().join(format!("{tool_use_id}.txt"))
    }

    /// Process a tool result: persist large results to disk, return empty-result
    /// markers, and pass through small results unchanged.
    ///
    /// Returns `Ok(None)` if the content is small enough and should be used as-is.
    /// Returns `Ok(Some(replacement))` with the persisted-output XML message.
    /// Returns `Err` only on unexpected I/O failures (caller should fallback to
    /// using original content).
    pub fn process_result(
        &self,
        tool_name: &str,
        tool_use_id: &str,
        content: &str,
        persistence_threshold: usize,
    ) -> Result<Option<String>, String> {
        if content.trim().is_empty() {
            return Ok(Some(format!("({tool_name} completed with no output)")));
        }

        if content.starts_with(PERSISTED_OUTPUT_TAG) {
            return Ok(None);
        }

        if content.len() <= persistence_threshold {
            return Ok(None);
        }

        let result = self.persist_tool_result(content, tool_use_id)?;
        let message = build_large_tool_result_message(&result);
        Ok(Some(message))
    }

    fn persist_tool_result(
        &self,
        content: &str,
        tool_use_id: &str,
    ) -> Result<PersistedToolResult, String> {
        let dir = self.tool_results_dir();
        fs::create_dir_all(&dir).map_err(|e| {
            format!("Failed to create tool-results directory {}: {e}", dir.display())
        })?;

        let filepath = self.tool_result_path(tool_use_id);

        // O_EXCL equivalent: create_new(true) fails if file already exists.
        // This makes replays safe — we never overwrite a prior persist.
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&filepath)
        {
            Ok(mut f) => {
                f.write_all(content.as_bytes()).map_err(|e| {
                    format!("Failed to write tool result to {}: {e}", filepath.display())
                })?;
                tracing::debug!(
                    path = %filepath.display(),
                    size = content.len(),
                    "Persisted tool result to disk"
                );
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // Already persisted on a prior turn — fall through to preview.
            }
            Err(e) => {
                return Err(format!(
                    "Failed to create tool result file {}: {e}",
                    filepath.display()
                ));
            }
        }

        let (preview, has_more) = generate_preview(content, PREVIEW_SIZE_BYTES);

        Ok(PersistedToolResult {
            filepath,
            original_size: content.len(),
            preview,
            has_more,
        })
    }
}

/// Build the XML-wrapped message that replaces a large tool result in the
/// conversation. The model sees the preview + file path and can use `read_file`
/// to access the full content.
pub fn build_large_tool_result_message(result: &PersistedToolResult) -> String {
    let size = format_file_size(result.original_size);
    let preview_size = format_file_size(PREVIEW_SIZE_BYTES);
    let trail = if result.has_more { "\n...\n" } else { "\n" };
    format!(
        "{PERSISTED_OUTPUT_TAG}\n\
         Output too large ({size}). Full output saved to: {path}\n\n\
         Preview (first {preview_size}):\n\
         {preview}{trail}\
         {PERSISTED_OUTPUT_CLOSING_TAG}",
        path = result.filepath.display(),
        preview = result.preview,
    )
}

/// Generate a preview of content, truncating at a newline boundary when possible.
pub fn generate_preview(content: &str, max_bytes: usize) -> (String, bool) {
    if content.len() <= max_bytes {
        return (content.to_string(), false);
    }

    let truncated = &content[..max_bytes.min(content.len())];
    let last_newline = truncated.rfind('\n');

    let cut_point = match last_newline {
        Some(pos) if pos > max_bytes / 2 => pos + 1,
        _ => max_bytes.min(content.len()),
    };

    (content[..cut_point].to_string(), true)
}

fn format_file_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_storage() -> (ToolResultStorage, TempDir) {
        let tmp = TempDir::new().unwrap();
        let storage = ToolResultStorage::new(tmp.path().to_path_buf());
        (storage, tmp)
    }

    #[test]
    fn empty_result_returns_marker() {
        let (storage, _tmp) = make_storage();
        let result = storage
            .process_result("shell_exec", "id1", "", 50_000)
            .unwrap();
        assert_eq!(
            result,
            Some("(shell_exec completed with no output)".to_string())
        );
    }

    #[test]
    fn whitespace_only_result_returns_marker() {
        let (storage, _tmp) = make_storage();
        let result = storage
            .process_result("shell_exec", "id2", "   \n\t  ", 50_000)
            .unwrap();
        assert_eq!(
            result,
            Some("(shell_exec completed with no output)".to_string())
        );
    }

    #[test]
    fn already_persisted_returns_none() {
        let (storage, _tmp) = make_storage();
        let content = format!("{PERSISTED_OUTPUT_TAG}\nsome preview\n{PERSISTED_OUTPUT_CLOSING_TAG}");
        let result = storage
            .process_result("read_file", "id3", &content, 50_000)
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn small_result_passes_through() {
        let (storage, _tmp) = make_storage();
        let result = storage
            .process_result("shell_exec", "id4", "hello world", 50_000)
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn large_result_persists_to_disk() {
        let (storage, _tmp) = make_storage();
        let content = "x".repeat(60_000);
        let result = storage
            .process_result("shell_exec", "id5", &content, 50_000)
            .unwrap();
        assert!(result.is_some());
        let msg = result.unwrap();
        assert!(msg.starts_with(PERSISTED_OUTPUT_TAG));
        assert!(msg.contains("Output too large"));
        assert!(msg.ends_with(PERSISTED_OUTPUT_CLOSING_TAG));

        let persisted_path = storage.tool_result_path("id5");
        assert!(persisted_path.exists());
        assert_eq!(fs::read_to_string(&persisted_path).unwrap().len(), 60_000);
    }

    #[test]
    fn duplicate_persist_is_idempotent() {
        let (storage, _tmp) = make_storage();
        let content = "y".repeat(60_000);
        let r1 = storage
            .process_result("shell_exec", "id6", &content, 50_000)
            .unwrap();
        let r2 = storage
            .process_result("shell_exec", "id6", &content, 50_000)
            .unwrap();
        assert!(r1.is_some());
        assert!(r2.is_some());
    }

    #[test]
    fn get_persistence_threshold_infinity_opt_out() {
        assert_eq!(get_persistence_threshold(usize::MAX), usize::MAX);
    }

    #[test]
    fn get_persistence_threshold_below_default() {
        assert_eq!(get_persistence_threshold(30_000), 30_000);
    }

    #[test]
    fn get_persistence_threshold_above_default() {
        assert_eq!(get_persistence_threshold(100_000), 50_000);
    }

    #[test]
    fn generate_preview_short_content() {
        let (preview, has_more) = generate_preview("hello", 2000);
        assert_eq!(preview, "hello");
        assert!(!has_more);
    }

    #[test]
    fn generate_preview_truncates_at_newline() {
        let mut content = String::new();
        for i in 0..200 {
            content.push_str(&format!("line {i}\n"));
        }
        let (preview, has_more) = generate_preview(&content, 100);
        assert!(has_more);
        assert!(preview.len() <= 100);
        assert!(preview.ends_with('\n'));
    }

    #[test]
    fn build_large_tool_result_message_format() {
        let result = PersistedToolResult {
            filepath: PathBuf::from("/tmp/test.txt"),
            original_size: 60_000,
            preview: "hello world".to_string(),
            has_more: true,
        };
        let msg = build_large_tool_result_message(&result);
        assert!(msg.starts_with(PERSISTED_OUTPUT_TAG));
        assert!(msg.contains("58.6 KB"));
        assert!(msg.contains("/tmp/test.txt"));
        assert!(msg.contains("hello world"));
        assert!(msg.contains("..."));
        assert!(msg.ends_with(PERSISTED_OUTPUT_CLOSING_TAG));
    }

    #[test]
    fn format_file_size_various() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(50_000), "48.8 KB");
        assert_eq!(format_file_size(1_048_576), "1.0 MB");
    }
}
