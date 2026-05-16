use crate::embedded::GatewayInfo;
use crate::AppData;

/// Get gateway connection info for the frontend.
///
/// Waits up to 30 seconds for the gateway to become ready.
/// The frontend uses this to establish WebSocket connection to the Gateway.
/// All business logic goes through WebSocket, not IPC.
#[tauri::command]
pub async fn get_gateway_info(state: tauri::State<'_, AppData>) -> Result<GatewayInfo, String> {
    const TIMEOUT_SECS: u64 = 30;
    const CHECK_INTERVAL_MS: u64 = 500;

    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(TIMEOUT_SECS);
    let interval = std::time::Duration::from_millis(CHECK_INTERVAL_MS);

    loop {
        let gw = state.gateway.lock().await;
        if let Some(g) = gw.as_ref() {
            return Ok(g.info().clone());
        }
        drop(gw); // Release lock before sleeping

        if start.elapsed() >= timeout {
            return Err(format!(
                "gateway not started after {}s. Check logs for errors.",
                TIMEOUT_SECS
            ));
        }

        tokio::time::sleep(interval).await;
    }
}