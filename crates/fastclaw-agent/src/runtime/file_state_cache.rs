use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use dashmap::DashMap;

#[derive(Debug, Clone)]
struct FileState {
    content_hash: u64,
    modified_at: SystemTime,
    content_preview: String,
}

/// Cache that tracks file content hashes and modification times to avoid
/// redundant re-reads of unchanged files during an agent session.
#[derive(Debug, Clone)]
pub struct FileStateCache {
    cache: DashMap<PathBuf, FileState>,
}

impl Default for FileStateCache {
    fn default() -> Self {
        Self::new()
    }
}

impl FileStateCache {
    pub fn new() -> Self {
        Self {
            cache: DashMap::new(),
        }
    }

    /// Check whether the file at `path` is unchanged since last update.
    /// Returns `true` if the file's modification time matches our cached value.
    /// Returns `false` if the file has been modified, is missing from cache,
    /// or if we cannot stat the file.
    pub fn is_unchanged(&self, path: &Path) -> bool {
        let entry = match self.cache.get(path) {
            Some(e) => e,
            None => return false,
        };

        let current_mtime = match std::fs::metadata(path).and_then(|m| m.modified()) {
            Ok(t) => t,
            Err(_) => return false,
        };

        entry.modified_at == current_mtime
    }

    /// Record the current state of a file's content.
    /// Stores the content hash, modification time, and a preview (first 200 lines).
    pub fn update(&self, path: &Path, content: &str) {
        let mtime = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        let hash = compute_hash(content);
        let preview = content
            .lines()
            .take(200)
            .collect::<Vec<_>>()
            .join("\n");

        self.cache.insert(
            path.to_path_buf(),
            FileState {
                content_hash: hash,
                modified_at: mtime,
                content_preview: preview,
            },
        );
    }

    /// Remove a single path from the cache (e.g. after a write/edit operation).
    pub fn invalidate(&self, path: &Path) {
        self.cache.remove(path);
    }

    /// Clear the entire cache.
    pub fn invalidate_all(&self) {
        self.cache.clear();
    }

    /// Get the cached content preview for a path, if available and unchanged.
    pub fn get_preview(&self, path: &Path) -> Option<String> {
        if self.is_unchanged(path) {
            self.cache.get(path).map(|e| e.content_preview.clone())
        } else {
            None
        }
    }

    /// Get the content hash for a path, if cached.
    pub fn content_hash(&self, path: &Path) -> Option<u64> {
        self.cache.get(path).map(|e| e.content_hash)
    }

    /// Number of entries currently cached.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

fn compute_hash(content: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn is_unchanged_returns_true_for_unmodified_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "hello world").unwrap();

        let cache = FileStateCache::new();
        cache.update(&file_path, "hello world");

        assert!(cache.is_unchanged(&file_path));
    }

    #[test]
    fn is_unchanged_returns_false_after_modification() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "hello").unwrap();

        let cache = FileStateCache::new();
        cache.update(&file_path, "hello");
        assert!(cache.is_unchanged(&file_path));

        std::thread::sleep(std::time::Duration::from_millis(50));
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&file_path)
            .unwrap();
        f.write_all(b"modified content").unwrap();
        f.flush().unwrap();
        drop(f);

        assert!(!cache.is_unchanged(&file_path));
    }

    #[test]
    fn invalidate_removes_single_path() {
        let dir = tempfile::tempdir().unwrap();
        let path_a = dir.path().join("a.txt");
        let path_b = dir.path().join("b.txt");
        std::fs::write(&path_a, "aaa").unwrap();
        std::fs::write(&path_b, "bbb").unwrap();

        let cache = FileStateCache::new();
        cache.update(&path_a, "aaa");
        cache.update(&path_b, "bbb");
        assert_eq!(cache.len(), 2);

        cache.invalidate(&path_a);
        assert_eq!(cache.len(), 1);
        assert!(!cache.is_unchanged(&path_a));
        assert!(cache.is_unchanged(&path_b));
    }

    #[test]
    fn invalidate_all_clears_cache() {
        let dir = tempfile::tempdir().unwrap();
        let path_a = dir.path().join("a.txt");
        let path_b = dir.path().join("b.txt");
        std::fs::write(&path_a, "aaa").unwrap();
        std::fs::write(&path_b, "bbb").unwrap();

        let cache = FileStateCache::new();
        cache.update(&path_a, "aaa");
        cache.update(&path_b, "bbb");
        assert_eq!(cache.len(), 2);

        cache.invalidate_all();
        assert!(cache.is_empty());
        assert!(!cache.is_unchanged(&path_a));
        assert!(!cache.is_unchanged(&path_b));
    }

    #[test]
    fn get_preview_returns_first_200_lines() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("long.txt");

        let content: String = (0..300).map(|i| format!("line {i}")).collect::<Vec<_>>().join("\n");
        std::fs::write(&file_path, &content).unwrap();

        let cache = FileStateCache::new();
        cache.update(&file_path, &content);

        let preview = cache.get_preview(&file_path).unwrap();
        let preview_lines: Vec<_> = preview.lines().collect();
        assert_eq!(preview_lines.len(), 200);
        assert_eq!(preview_lines[0], "line 0");
        assert_eq!(preview_lines[199], "line 199");
    }
}
