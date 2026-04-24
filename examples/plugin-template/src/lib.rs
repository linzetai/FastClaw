//! FastClaw WASM Plugin Template
//!
//! Build: `cargo build --target wasm32-wasi --release`
//! Output: `target/wasm32-wasi/release/fastclaw_plugin_template.wasm`
//!
//! Place the `.wasm` file alongside a `fastclaw.plugin.json` manifest
//! in your plugins directory.

use serde::{Deserialize, Serialize};

/// Plugin entry point — called by the FastClaw host for each invocation.
///
/// Input and output are JSON-encoded strings passed through the WASM ABI.
#[no_mangle]
pub extern "C" fn invoke(input_ptr: *const u8, input_len: usize) -> u64 {
    let input = unsafe { std::slice::from_raw_parts(input_ptr, input_len) };
    let input_str = std::str::from_utf8(input).unwrap_or("{}");

    let request: PluginRequest = serde_json::from_str(input_str).unwrap_or_default();
    let response = handle_request(&request);

    let output = serde_json::to_string(&response).unwrap_or_default();
    let bytes = output.into_bytes();
    let ptr = bytes.as_ptr() as u64;
    let len = bytes.len() as u64;
    std::mem::forget(bytes);

    (ptr << 32) | len
}

#[derive(Debug, Deserialize, Default)]
struct PluginRequest {
    capability: String,
    input: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct PluginResponse {
    success: bool,
    output: serde_json::Value,
}

fn handle_request(req: &PluginRequest) -> PluginResponse {
    match req.capability.as_str() {
        "hello" => PluginResponse {
            success: true,
            output: serde_json::json!({
                "message": format!("Hello from plugin! Input: {}", req.input),
            }),
        },
        _ => PluginResponse {
            success: false,
            output: serde_json::json!({
                "error": format!("Unknown capability: {}", req.capability),
            }),
        },
    }
}
