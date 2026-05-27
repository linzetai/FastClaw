use std::collections::HashMap;

use fastclaw_protocol::ApprovalDecision;

/// Session-scoped approval cache.
///
/// Stores `ApprovedForSession` decisions keyed by a canonical string
/// derived from tool-specific approval keys. The orchestrator checks this
/// cache before prompting the user — if all keys match a prior approval,
/// the tool call is silently allowed.
#[derive(Debug, Default)]
pub struct ApprovalCache {
    decisions: HashMap<String, ApprovalDecision>,
}

impl ApprovalCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if all provided keys have a cached `ApprovedForSession` decision.
    /// Returns `Some(ApprovedForSession)` only if every key is present and approved.
    pub fn check(&self, keys: &[String]) -> Option<ApprovalDecision> {
        if keys.is_empty() {
            return None;
        }
        let all_approved = keys.iter().all(|k| {
            matches!(
                self.decisions.get(k),
                Some(ApprovalDecision::ApprovedForSession)
            )
        });
        if all_approved {
            Some(ApprovalDecision::ApprovedForSession)
        } else {
            None
        }
    }

    /// Store a decision for the given keys. Only `ApprovedForSession` is cached;
    /// other decisions are not stored (they're one-shot).
    pub fn store(&mut self, keys: &[String], decision: ApprovalDecision) {
        if decision == ApprovalDecision::ApprovedForSession {
            for key in keys {
                self.decisions.insert(key.clone(), decision.clone());
            }
        }
    }

    /// Clear all cached decisions (e.g. on session end).
    pub fn clear(&mut self) {
        self.decisions.clear();
    }

    /// Number of cached approval entries.
    pub fn len(&self) -> usize {
        self.decisions.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.decisions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_keys_returns_none() {
        let cache = ApprovalCache::new();
        assert_eq!(cache.check(&[]), None);
    }

    #[test]
    fn store_and_check_approved_for_session() {
        let mut cache = ApprovalCache::new();
        let keys = vec!["shell:ls:/tmp".to_string()];
        cache.store(&keys, ApprovalDecision::ApprovedForSession);
        assert_eq!(cache.check(&keys), Some(ApprovalDecision::ApprovedForSession));
    }

    #[test]
    fn non_session_approval_not_cached() {
        let mut cache = ApprovalCache::new();
        let keys = vec!["shell:rm:/tmp".to_string()];
        cache.store(&keys, ApprovalDecision::Approved);
        assert_eq!(cache.check(&keys), None);
    }

    #[test]
    fn partial_key_match_returns_none() {
        let mut cache = ApprovalCache::new();
        let keys = vec!["cmd:a".to_string(), "cmd:b".to_string()];
        cache.store(&["cmd:a".to_string()], ApprovalDecision::ApprovedForSession);
        assert_eq!(cache.check(&keys), None);
    }

    #[test]
    fn clear_removes_all() {
        let mut cache = ApprovalCache::new();
        cache.store(
            &["key1".to_string()],
            ApprovalDecision::ApprovedForSession,
        );
        assert_eq!(cache.len(), 1);
        cache.clear();
        assert!(cache.is_empty());
    }
}
