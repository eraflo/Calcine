//! Usage over time, for the Activity page.

use std::sync::Arc;

use calcine_gateway::Gateway;
use calcine_gateway::usage::{UsagePeriod, UsageSummary};
use tauri::State;

/// Requests, tokens, speed and energy for `period`, ending now. Days start
/// at local midnight: `utc_offset_minutes` is the webview's offset from UTC.
#[tauri::command]
#[specta::specta]
pub fn usage_summary(
    gateway: State<'_, Arc<Gateway>>,
    period: UsagePeriod,
    utc_offset_minutes: i32,
) -> UsageSummary {
    gateway.usage().summary(period, utc_offset_minutes)
}

/// Forget all recorded usage.
#[tauri::command]
#[specta::specta]
pub fn clear_usage(gateway: State<'_, Arc<Gateway>>) {
    gateway.usage().clear();
}
