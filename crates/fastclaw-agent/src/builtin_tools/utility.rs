use std::collections::HashMap;

use async_trait::async_trait;
use fastclaw_core::tool::{Tool, ToolKind, ToolParameterSchema, ToolResult};

/// Returns current UTC time. Useful for agents that need to reason about time.
pub struct CurrentTimeTool;

#[async_trait]
impl Tool for CurrentTimeTool {
    fn kind(&self) -> ToolKind { ToolKind::Think }
    fn name(&self) -> &str {
        "get_current_time"
    }

    fn description(&self) -> &str {
        "Get current time in UTC (RFC3339). Returns JSON {\"utc\": \"...\"}. No parameters needed."
    }

    fn parameters_schema(&self) -> ToolParameterSchema {
        ToolParameterSchema {
            schema_type: "object".to_string(),
            properties: HashMap::new(),
            required: Vec::new(),
        }
    }

    async fn execute(&self, _arguments: &str) -> ToolResult {
        let now = chrono::Utc::now();
        ToolResult::ok(format!("{{\"utc\": \"{}\"}}", now.to_rfc3339()))
    }
}

/// Simple calculator for basic arithmetic.
pub struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn kind(&self) -> ToolKind { ToolKind::Think }
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Evaluate a simple arithmetic expression (+ - * /). \
         Supports decimal literals with standard operator precedence. \
         No parentheses, functions, or variables — use shell_exec with python for complex math."
    }

    fn parameters_schema(&self) -> ToolParameterSchema {
        let mut props = HashMap::new();
        props.insert(
            "expression".to_string(),
            serde_json::json!({
                "type": "string",
                "description": "Arithmetic expression, e.g. '100 / 4 + 2'."
            }),
        );
        ToolParameterSchema {
            schema_type: "object".to_string(),
            properties: props,
            required: vec!["expression".to_string()],
        }
    }

    async fn execute(&self, arguments: &str) -> ToolResult {
        let args: serde_json::Value = match serde_json::from_str(arguments) {
            Ok(v) => v,
            Err(e) => return ToolResult::err(format!(
                "calculator arguments are not valid JSON: {e}. \
                 Pass exactly {{\"expression\": \"1 + 2 * 3\"}} with a string value, then retry."
            )),
        };

        let expr = match args.get("expression").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return ToolResult::err(
                "calculator is missing required string field 'expression'. \
                 Example: {\"expression\": \"100 / 4 + 2\"}."
                    .to_string(),
            ),
        };

        match eval_simple_expr(expr) {
            Some(result) => ToolResult::ok(format!("{{\"result\": {result}}}")),
            None => ToolResult::err(format!(
                "calculator could not evaluate '{expr}'. \
                 What went wrong: the parser only accepts digits, at most one '.' per number, whitespace, and binary operators + - * / in a flat left-to-right expression—division by zero also yields this error. \
                 What to do next: remove parentheses, letters, commas, underscores, scientific notation (1e6), or unsupported symbols; split into smaller calculator calls; for sqrt/mod/log use shell_exec with python -c only if policy allows."
            )),
        }
    }
}

fn eval_simple_expr(expr: &str) -> Option<f64> {
    let expr = expr.trim();
    let mut result: f64 = 0.0;
    let mut current_op = '+';
    let mut num_str = String::new();
    let mut term_result: f64 = 0.0;

    let chars: Vec<char> = format!("{expr}+").chars().collect();

    for ch in chars {
        if ch.is_ascii_digit() || ch == '.' {
            num_str.push(ch);
        } else if ch == '+' || ch == '-' || ch == '*' || ch == '/' {
            let num: f64 = num_str.trim().parse().ok()?;
            num_str.clear();

            match current_op {
                '+' => {
                    result += term_result;
                    term_result = num;
                }
                '-' => {
                    result += term_result;
                    term_result = -num;
                }
                '*' => term_result *= num,
                '/' => {
                    if num == 0.0 {
                        return None;
                    }
                    term_result /= num;
                }
                _ => return None,
            }
            current_op = ch;
        } else if !ch.is_whitespace() {
            return None;
        }
    }

    Some(result + term_result)
}
