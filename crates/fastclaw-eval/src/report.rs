//! Eval report generation (JSON and HTML).

use serde::Serialize;

use crate::EvalSuiteResult;

/// Serializable eval report suitable for JSON and HTML output.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalReport {
    pub title: String,
    pub run_at: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub pass_rate: f64,
    pub results: Vec<CaseReport>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaseReport {
    pub case_id: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub tool_calls: Vec<String>,
    pub total_turns: u32,
    pub final_response: Option<String>,
    pub failure_reasons: Vec<String>,
}

impl EvalReport {
    /// Build a report from a suite result.
    pub fn from_suite(suite: &EvalSuiteResult, title: impl Into<String>) -> Self {
        let results = suite
            .results
            .iter()
            .map(|r| CaseReport {
                case_id: r.case_id.clone(),
                passed: r.passed,
                duration_ms: r.duration_ms,
                tool_calls: r.tool_calls_made.clone(),
                total_turns: r.total_turns,
                final_response: r.final_response.clone(),
                failure_reasons: r.failure_reasons.clone(),
            })
            .collect();

        Self {
            title: title.into(),
            run_at: suite.run_at.clone(),
            total: suite.total,
            passed: suite.passed,
            failed: suite.failed,
            pass_rate: suite.pass_rate,
            results,
        }
    }

    /// Render the report as pretty-printed JSON.
    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Render the report as a self-contained HTML page.
    pub fn to_html(&self) -> String {
        let pass_pct = (self.pass_rate * 100.0).round();
        let bar_color = if pass_pct >= 80.0 {
            "#68D391"
        } else if pass_pct >= 50.0 {
            "#ED8936"
        } else {
            "#FC8181"
        };

        let mut rows = String::new();
        for r in &self.results {
            let status = if r.passed { "PASS" } else { "FAIL" };
            let status_color = if r.passed { "#276749" } else { "#C53030" };
            let reasons = if r.failure_reasons.is_empty() {
                "—".to_string()
            } else {
                r.failure_reasons.join("; ")
            };
            rows.push_str(&format!(
                "<tr>\
                    <td>{}</td>\
                    <td style=\"color:{status_color};font-weight:bold\">{status}</td>\
                    <td>{}ms</td>\
                    <td>{}</td>\
                    <td>{}</td>\
                </tr>",
                html_escape(&r.case_id),
                r.duration_ms,
                r.total_turns,
                html_escape(&reasons),
            ));
        }

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>{title} — FastClaw Eval</title>
<style>
body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; max-width: 900px; margin: 2rem auto; color: #2D3748; }}
h1 {{ color: #2D3748; }}
.summary {{ display: flex; gap: 2rem; margin: 1rem 0; }}
.stat {{ padding: 1rem; border-radius: 8px; background: #F7FAFC; border: 1px solid #E2E8F0; }}
.stat .num {{ font-size: 2rem; font-weight: bold; }}
.bar {{ height: 8px; border-radius: 4px; background: #EDF2F7; margin: 1rem 0; }}
.bar-fill {{ height: 100%; border-radius: 4px; }}
table {{ width: 100%; border-collapse: collapse; margin-top: 1rem; }}
th, td {{ text-align: left; padding: 0.5rem 0.75rem; border-bottom: 1px solid #E2E8F0; }}
th {{ background: #F7FAFC; }}
</style>
</head>
<body>
<h1>{title}</h1>
<p>Run at: {run_at}</p>
<div class="summary">
    <div class="stat"><div class="num">{total}</div>Total</div>
    <div class="stat"><div class="num" style="color:#276749">{passed}</div>Passed</div>
    <div class="stat"><div class="num" style="color:#C53030">{failed}</div>Failed</div>
    <div class="stat"><div class="num">{pass_pct}%</div>Pass Rate</div>
</div>
<div class="bar"><div class="bar-fill" style="width:{pass_pct}%;background:{bar_color}"></div></div>
<table>
<thead><tr><th>Case</th><th>Status</th><th>Latency</th><th>Turns</th><th>Failures</th></tr></thead>
<tbody>{rows}</tbody>
</table>
</body>
</html>"#,
            title = html_escape(&self.title),
            run_at = html_escape(&self.run_at),
            total = self.total,
            passed = self.passed,
            failed = self.failed,
            pass_pct = pass_pct,
            bar_color = bar_color,
            rows = rows,
        )
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EvalResult, EvalSuiteResult};

    fn suite(pass: usize, fail: usize) -> EvalSuiteResult {
        let total = pass + fail;
        let mut results = Vec::new();
        for i in 0..pass {
            results.push(EvalResult {
                case_id: format!("pass-{i}"),
                passed: true,
                tool_calls_made: vec!["tool_a".into()],
                total_turns: 1,
                final_response: Some("ok".into()),
                duration_ms: 100,
                failure_reasons: vec![],
            });
        }
        for i in 0..fail {
            results.push(EvalResult {
                case_id: format!("fail-{i}"),
                passed: false,
                tool_calls_made: vec![],
                total_turns: 1,
                final_response: None,
                duration_ms: 200,
                failure_reasons: vec!["missing tool".into()],
            });
        }
        let pass_rate = if total > 0 {
            pass as f64 / total as f64
        } else {
            0.0
        };
        EvalSuiteResult {
            total,
            passed: pass,
            failed: fail,
            results,
            pass_rate,
            run_at: "2026-04-25T00:00:00Z".into(),
        }
    }

    #[test]
    fn from_suite_counts() {
        let s = suite(3, 1);
        let report = EvalReport::from_suite(&s, "test");
        assert_eq!(report.total, 4);
        assert_eq!(report.passed, 3);
        assert_eq!(report.failed, 1);
        assert!((report.pass_rate - 0.75).abs() < f64::EPSILON);
        assert_eq!(report.results.len(), 4);
    }

    #[test]
    fn to_json_roundtrip() {
        let s = suite(2, 1);
        let report = EvalReport::from_suite(&s, "roundtrip");
        let json_str = report.to_json().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["title"], "roundtrip");
        assert_eq!(parsed["total"], 3);
        assert_eq!(parsed["passed"], 2);
        assert_eq!(parsed["results"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn to_html_contains_structure() {
        let s = suite(1, 1);
        let report = EvalReport::from_suite(&s, "html-test");
        let html = report.to_html();
        assert!(html.contains("<title>html-test"));
        assert!(html.contains("pass-0"));
        assert!(html.contains("fail-0"));
        assert!(html.contains("PASS"));
        assert!(html.contains("FAIL"));
        assert!(html.contains("%"));
    }

    #[test]
    fn html_escapes_special_chars() {
        let mut s = suite(0, 1);
        s.results[0].case_id = "<script>alert(1)</script>".into();
        let report = EvalReport::from_suite(&s, "escape");
        let html = report.to_html();
        assert!(!html.contains("<script>alert"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn empty_suite() {
        let s = suite(0, 0);
        let report = EvalReport::from_suite(&s, "empty");
        assert_eq!(report.total, 0);
        assert!((report.pass_rate - 0.0).abs() < f64::EPSILON);
        let html = report.to_html();
        assert!(html.contains("<title>empty"));
    }
}
