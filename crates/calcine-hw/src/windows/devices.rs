//! Windows driver database (WMI): the Hexagon NPU is a `ComputeAccelerator`
//! device, the Adreno GPU a `Display` device.

use calcine_core::hardware::Accelerator;
use calcine_core::models::ComputeUnit;
use serde::Deserialize;

use crate::snapshot::PlatformDevices;

#[derive(Debug, Deserialize)]
#[serde(rename = "Win32_PnPSignedDriver", rename_all = "PascalCase")]
struct SignedDriver {
    device_name: Option<String>,
    device_class: Option<String>,
    driver_version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "Win32_Processor", rename_all = "PascalCase")]
struct Processor {
    name: Option<String>,
}

pub(crate) fn devices() -> wmi::WMIResult<PlatformDevices> {
    let connection = wmi::WMIConnection::new()?;
    let drivers: Vec<SignedDriver> = connection.raw_query(
        "SELECT DeviceName, DeviceClass, DriverVersion FROM Win32_PnPSignedDriver \
         WHERE DeviceClass = 'COMPUTEACCELERATOR' OR DeviceClass = 'DISPLAY'",
    )?;
    let processors: Vec<Processor> = connection.raw_query("SELECT Name FROM Win32_Processor")?;
    Ok(PlatformDevices {
        accelerators: drivers.into_iter().filter_map(to_accelerator).collect(),
        cpu_name: processors
            .into_iter()
            .find_map(|processor| processor.name)
            .map(|name| name.trim().to_owned())
            .filter(|name| !name.is_empty()),
    })
}

fn to_accelerator(driver: SignedDriver) -> Option<Accelerator> {
    let name = driver.device_name?.trim().to_owned();
    let unit = match driver.device_class?.to_ascii_uppercase().as_str() {
        "COMPUTEACCELERATOR" => ComputeUnit::Npu,
        // Skip virtual adapters (remote desktop, basic display driver).
        "DISPLAY" if !name.starts_with("Microsoft") => ComputeUnit::Gpu,
        _ => return None,
    };
    Some(Accelerator {
        unit,
        name,
        driver_version: driver.driver_version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn driver(name: &str, class: &str) -> SignedDriver {
        SignedDriver {
            device_name: Some(name.into()),
            device_class: Some(class.into()),
            driver_version: Some("1.0".into()),
        }
    }

    #[test]
    fn classifies_npu_and_gpu_and_skips_virtual_adapters() {
        let npu = to_accelerator(driver("Qualcomm(R) Hexagon(TM) NPU", "COMPUTEACCELERATOR"));
        assert_eq!(npu.map(|a| a.unit), Some(ComputeUnit::Npu));
        let gpu = to_accelerator(driver("Qualcomm(R) Adreno(TM) X1-85 GPU", "DISPLAY"));
        assert_eq!(gpu.map(|a| a.unit), Some(ComputeUnit::Gpu));
        assert!(to_accelerator(driver("Microsoft Basic Display Adapter", "DISPLAY")).is_none());
    }
}
