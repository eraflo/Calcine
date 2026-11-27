//! The battery, from WMI (`Win32_Battery`). Desktops have none.

use calcine_core::hardware::PowerSource;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename = "Win32_Battery", rename_all = "PascalCase")]
struct Battery {
    /// 1: discharging; 2: on mains; 3 and up: charging, full, low…
    battery_status: Option<u16>,
    estimated_charge_remaining: Option<u16>,
}

pub(crate) fn power_source() -> wmi::WMIResult<Option<PowerSource>> {
    let connection = wmi::WMIConnection::new()?;
    let batteries: Vec<Battery> = connection
        .raw_query("SELECT BatteryStatus, EstimatedChargeRemaining FROM Win32_Battery")?;
    Ok(batteries.first().map(|battery| PowerSource {
        on_battery: battery.battery_status == Some(1),
        battery_percent: battery
            .estimated_charge_remaining
            .and_then(|percent| u8::try_from(percent.min(100)).ok()),
    }))
}
