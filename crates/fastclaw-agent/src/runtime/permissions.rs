use std::path::Path;

use serde::{Deserialize, Serialize};

/// Rule effect: whether a matching rule allows or denies access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleEffect {
    Allow,
    Deny,
}

/// Rule scope: determines the priority/lifetime of a rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleScope {
    /// Session-level rule (higher priority, expires with session).
    Session,
    /// Global rule (lower priority, persists across sessions).
    Global,
}

/// Matcher for determining which tools/resources a rule applies to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleMatcher {
    /// Exact tool name match.
    Exact { tool: String },
    /// Prefix match (e.g. "git:" matches "git:push", "git:pull").
    Prefix { prefix: String },
    /// Wildcard/glob pattern (e.g. "file_*").
    Wildcard { pattern: String },
}

impl RuleMatcher {
    pub fn matches(&self, tool_name: &str) -> bool {
        match self {
            Self::Exact { tool } => tool == tool_name,
            Self::Prefix { prefix } => tool_name.starts_with(prefix),
            Self::Wildcard { pattern } => wildcard_match(pattern, tool_name),
        }
    }
}

/// A single permission rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    pub matcher: RuleMatcher,
    pub effect: RuleEffect,
    pub scope: RuleScope,
    #[serde(default)]
    pub reason: Option<String>,
}

/// Result of evaluating permissions for a tool call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionDecision {
    /// Access is allowed.
    Allowed,
    /// Access is denied with an optional reason.
    Denied(Option<String>),
    /// No matching rule found — falls through to default behavior.
    NoMatch,
}

/// Engine that evaluates a chain of permission rules.
#[derive(Debug, Clone, Default)]
pub struct PermissionRuleEngine {
    session_rules: Vec<PermissionRule>,
    global_rules: Vec<PermissionRule>,
}

impl PermissionRuleEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a rule. Automatically placed into session or global bucket.
    pub fn add_rule(&mut self, rule: PermissionRule) {
        match rule.scope {
            RuleScope::Session => self.session_rules.push(rule),
            RuleScope::Global => self.global_rules.push(rule),
        }
    }

    /// Add multiple rules.
    pub fn add_rules(&mut self, rules: Vec<PermissionRule>) {
        for rule in rules {
            self.add_rule(rule);
        }
    }

    /// Evaluate whether a tool call is permitted.
    /// Priority order:
    /// 1. Session deny rules (highest priority)
    /// 2. Session allow rules
    /// 3. Global deny rules
    /// 4. Global allow rules
    /// 5. NoMatch (fallback)
    ///
    /// Within the same scope, deny takes priority over allow.
    pub fn evaluate(&self, tool_name: &str) -> PermissionDecision {
        // Check session rules first (higher priority)
        if let Some(decision) = self.evaluate_scope(&self.session_rules, tool_name) {
            return decision;
        }

        // Then global rules
        if let Some(decision) = self.evaluate_scope(&self.global_rules, tool_name) {
            return decision;
        }

        PermissionDecision::NoMatch
    }

    fn evaluate_scope(
        &self,
        rules: &[PermissionRule],
        tool_name: &str,
    ) -> Option<PermissionDecision> {
        let mut has_allow = false;
        let mut allow_reason: Option<String> = None;

        for rule in rules {
            if !rule.matcher.matches(tool_name) {
                continue;
            }
            match rule.effect {
                RuleEffect::Deny => {
                    return Some(PermissionDecision::Denied(rule.reason.clone()));
                }
                RuleEffect::Allow => {
                    has_allow = true;
                    if allow_reason.is_none() {
                        allow_reason = rule.reason.clone();
                    }
                }
            }
        }

        if has_allow {
            Some(PermissionDecision::Allowed)
        } else {
            None
        }
    }

    /// Load rules from a JSON file in the given directory.
    /// Looks for `settings.json5` or `settings.json`.
    pub fn load_from_dir(dir: &Path) -> anyhow::Result<Self> {
        let json5_path = dir.join("settings.json5");
        let json_path = dir.join("settings.json");

        let content = if json5_path.exists() {
            std::fs::read_to_string(&json5_path)?
        } else if json_path.exists() {
            std::fs::read_to_string(&json_path)?
        } else {
            return Ok(Self::new());
        };

        let config: PermissionConfig = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("failed to parse permission config: {e}"))?;

        let mut engine = Self::new();
        engine.add_rules(config.permissions);
        Ok(engine)
    }

    /// Clear all session-scoped rules (e.g. when session ends).
    pub fn clear_session_rules(&mut self) {
        self.session_rules.clear();
    }

    /// Number of rules in the engine.
    pub fn rule_count(&self) -> usize {
        self.session_rules.len() + self.global_rules.len()
    }

    /// Get the reason why a tool is denied, if applicable.
    pub fn permission_explain(&self, tool_name: &str) -> String {
        match self.evaluate(tool_name) {
            PermissionDecision::Allowed => format!("'{tool_name}' is allowed"),
            PermissionDecision::Denied(reason) => {
                let r = reason.unwrap_or_else(|| "no reason given".into());
                format!("'{tool_name}' is denied: {r}")
            }
            PermissionDecision::NoMatch => {
                format!("'{tool_name}' has no matching rule (default behavior applies)")
            }
        }
    }
}

/// Configuration file structure for permissions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PermissionConfig {
    #[serde(default)]
    permissions: Vec<PermissionRule>,
}

/// Simple wildcard matching (supports `*` and `?`).
fn wildcard_match(pattern: &str, text: &str) -> bool {
    super::hook_executor::glob_match(pattern, text)
}

// ── Convenience constructors ─────────────────────────────────────────

impl PermissionRule {
    pub fn allow_exact(tool: &str) -> Self {
        Self {
            matcher: RuleMatcher::Exact { tool: tool.into() },
            effect: RuleEffect::Allow,
            scope: RuleScope::Global,
            reason: None,
        }
    }

    pub fn deny_exact(tool: &str) -> Self {
        Self {
            matcher: RuleMatcher::Exact { tool: tool.into() },
            effect: RuleEffect::Deny,
            scope: RuleScope::Global,
            reason: None,
        }
    }

    pub fn allow_prefix(prefix: &str) -> Self {
        Self {
            matcher: RuleMatcher::Prefix { prefix: prefix.into() },
            effect: RuleEffect::Allow,
            scope: RuleScope::Global,
            reason: None,
        }
    }

    pub fn deny_prefix(prefix: &str) -> Self {
        Self {
            matcher: RuleMatcher::Prefix { prefix: prefix.into() },
            effect: RuleEffect::Deny,
            scope: RuleScope::Global,
            reason: None,
        }
    }

    pub fn with_scope(mut self, scope: RuleScope) -> Self {
        self.scope = scope;
        self
    }

    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn exact_match_works() {
        let matcher = RuleMatcher::Exact { tool: "shell_exec".into() };
        assert!(matcher.matches("shell_exec"));
        assert!(!matcher.matches("shell_exec_v2"));
        assert!(!matcher.matches("file_read"));
    }

    #[test]
    fn prefix_match_works() {
        let matcher = RuleMatcher::Prefix { prefix: "git:".into() };
        assert!(matcher.matches("git:push"));
        assert!(matcher.matches("git:pull"));
        assert!(matcher.matches("git:"));
        assert!(!matcher.matches("github_search"));
    }

    #[test]
    fn wildcard_match_works() {
        let matcher = RuleMatcher::Wildcard { pattern: "file_*".into() };
        assert!(matcher.matches("file_read"));
        assert!(matcher.matches("file_write"));
        assert!(!matcher.matches("shell_exec"));
    }

    #[test]
    fn deny_takes_priority_over_allow_in_same_scope() {
        let mut engine = PermissionRuleEngine::new();
        engine.add_rule(PermissionRule::allow_exact("shell_exec"));
        engine.add_rule(PermissionRule::deny_exact("shell_exec").with_reason("dangerous"));

        assert_eq!(
            engine.evaluate("shell_exec"),
            PermissionDecision::Denied(Some("dangerous".into()))
        );
    }

    #[test]
    fn session_rules_override_global() {
        let mut engine = PermissionRuleEngine::new();
        engine.add_rule(PermissionRule::allow_exact("shell_exec"));
        engine.add_rule(
            PermissionRule::deny_exact("shell_exec")
                .with_scope(RuleScope::Session)
                .with_reason("session blocked"),
        );

        assert_eq!(
            engine.evaluate("shell_exec"),
            PermissionDecision::Denied(Some("session blocked".into()))
        );
    }

    #[test]
    fn session_allow_overrides_global_deny() {
        let mut engine = PermissionRuleEngine::new();
        engine.add_rule(PermissionRule::deny_exact("shell_exec"));
        engine.add_rule(PermissionRule::allow_exact("shell_exec").with_scope(RuleScope::Session));

        assert_eq!(engine.evaluate("shell_exec"), PermissionDecision::Allowed);
    }

    #[test]
    fn no_match_returns_nomatch() {
        let engine = PermissionRuleEngine::new();
        assert_eq!(engine.evaluate("any_tool"), PermissionDecision::NoMatch);
    }

    #[test]
    fn prefix_deny_blocks_subcommands() {
        let mut engine = PermissionRuleEngine::new();
        engine.add_rule(PermissionRule::deny_prefix("rm:").with_reason("destructive"));

        assert_eq!(
            engine.evaluate("rm:-rf"),
            PermissionDecision::Denied(Some("destructive".into()))
        );
        assert_eq!(
            engine.evaluate("rm:file.txt"),
            PermissionDecision::Denied(Some("destructive".into()))
        );
        assert_eq!(engine.evaluate("ls"), PermissionDecision::NoMatch);
    }

    #[test]
    fn wildcard_allow_permits_matching() {
        let mut engine = PermissionRuleEngine::new();
        engine.add_rule(PermissionRule {
            matcher: RuleMatcher::Wildcard { pattern: "file_*".into() },
            effect: RuleEffect::Allow,
            scope: RuleScope::Global,
            reason: None,
        });

        assert_eq!(engine.evaluate("file_read"), PermissionDecision::Allowed);
        assert_eq!(engine.evaluate("file_write"), PermissionDecision::Allowed);
        assert_eq!(engine.evaluate("shell_exec"), PermissionDecision::NoMatch);
    }

    #[test]
    fn clear_session_rules_removes_session_only() {
        let mut engine = PermissionRuleEngine::new();
        engine.add_rule(PermissionRule::allow_exact("tool_a").with_scope(RuleScope::Session));
        engine.add_rule(PermissionRule::allow_exact("tool_b"));

        assert_eq!(engine.rule_count(), 2);
        engine.clear_session_rules();
        assert_eq!(engine.rule_count(), 1);
        assert_eq!(engine.evaluate("tool_a"), PermissionDecision::NoMatch);
        assert_eq!(engine.evaluate("tool_b"), PermissionDecision::Allowed);
    }

    #[test]
    fn load_from_dir_parses_json() {
        let dir = tempfile::tempdir().unwrap();
        let config = PermissionConfig {
            permissions: vec![
                PermissionRule::allow_exact("file_read"),
                PermissionRule::deny_prefix("rm:").with_reason("no deletion"),
            ],
        };
        let json = serde_json::to_string_pretty(&config).unwrap();
        let mut f = std::fs::File::create(dir.path().join("settings.json")).unwrap();
        f.write_all(json.as_bytes()).unwrap();

        let engine = PermissionRuleEngine::load_from_dir(dir.path()).unwrap();
        assert_eq!(engine.rule_count(), 2);
        assert_eq!(engine.evaluate("file_read"), PermissionDecision::Allowed);
        assert_eq!(
            engine.evaluate("rm:-rf"),
            PermissionDecision::Denied(Some("no deletion".into()))
        );
    }

    #[test]
    fn load_from_dir_returns_empty_when_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let engine = PermissionRuleEngine::load_from_dir(dir.path()).unwrap();
        assert_eq!(engine.rule_count(), 0);
    }

    #[test]
    fn permission_explain_messages() {
        let mut engine = PermissionRuleEngine::new();
        engine.add_rule(PermissionRule::allow_exact("safe_tool"));
        engine.add_rule(PermissionRule::deny_exact("bad_tool").with_reason("unsafe"));

        assert!(engine.permission_explain("safe_tool").contains("allowed"));
        assert!(engine.permission_explain("bad_tool").contains("denied"));
        assert!(engine.permission_explain("bad_tool").contains("unsafe"));
        assert!(engine.permission_explain("unknown").contains("no matching rule"));
    }

    #[test]
    fn multiple_rules_first_deny_wins() {
        let mut engine = PermissionRuleEngine::new();
        engine.add_rule(PermissionRule {
            matcher: RuleMatcher::Wildcard { pattern: "*".into() },
            effect: RuleEffect::Allow,
            scope: RuleScope::Global,
            reason: Some("default allow".into()),
        });
        engine.add_rule(PermissionRule::deny_exact("dangerous").with_reason("blocked"));

        assert_eq!(
            engine.evaluate("dangerous"),
            PermissionDecision::Denied(Some("blocked".into()))
        );
        assert_eq!(engine.evaluate("safe"), PermissionDecision::Allowed);
    }

    #[test]
    fn serialization_roundtrip() {
        let rule = PermissionRule::deny_prefix("git:force-")
            .with_scope(RuleScope::Session)
            .with_reason("no force push");
        let json = serde_json::to_string(&rule).unwrap();
        let deserialized: PermissionRule = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.effect, RuleEffect::Deny);
        assert_eq!(deserialized.scope, RuleScope::Session);
        assert!(deserialized.matcher.matches("git:force-push"));
        assert!(!deserialized.matcher.matches("git:pull"));
    }
}
