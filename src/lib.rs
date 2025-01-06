use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use crate::utils::windows::deserialisers::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CPUInfo {
    #[serde(rename = "Manufacturer")]
    pub vendor: String,

    #[serde(rename = "Description")]
    pub model: String,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "CurrentClockSpeed", deserialize_with = "format_frequency")]
    pub frequency: String,

    #[serde(rename = "Architecture", deserialize_with = "deserialize_architecture")]
    pub architecture: CPUArchitecture,

    #[serde(rename = "NumberOfCores", deserialize_with = "to_string")]
    pub cores: String,

    #[serde(rename = "NumberOfLogicalProcessors", deserialize_with = "to_string")]
    pub logical_cores: String,

    #[serde(flatten)]
    pub cache_size: CPUCacheSize,

    #[serde(rename = "VirtualizationFirmwareEnabled")]
    pub virtualisation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum CPUArchitecture {
    X86,          // The x86 processor architecture
    Arm,          // The ARM processor architecture
    X64,          // The x64 processor architecture
    Neutral,      // A neutral processor architecture
    Arm64,        // The Arm64 processor architecture
    X86OnArm64,   // The Arm64 processor architecture emulating the X86 architecture
    Unknown,      // An unknown processor architecture
}

#[derive(Debug, Deserialize)]
pub struct CPUCacheSize {
    #[serde(rename = "L1CacheSize", default, deserialize_with = "optional_to_string")]
    pub L1: String,

    #[serde(rename = "L2CacheSize", default, deserialize_with = "to_string")]
    pub L2: String,

    #[serde(rename = "L3CacheSize", default, deserialize_with = "to_string")]
    pub L3: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GPUInfo {
    #[serde(default)]
    index: u8,
    #[serde(rename = "AdapterCompatibility")]
    vendor: String,
    #[serde(rename = "Name")]
    model: String,
    #[serde(rename = "AdapterRAM")]
    memory: u128,
    #[serde(rename = "DeviceID")]
    device_id: String,
    #[serde(flatten)]
    refresh_rate: GPURefreshRate,
    #[serde(rename = "InstalledDisplayDrivers", deserialize_with = "deserialize_drivers")]
    display_drivers_location: Vec<String>,
    #[serde(rename = "DriverVersion")]
    driver_version: String,
    #[serde(rename = "VideoModeDescription", deserialize_with = "deserialize_video_modes")]
    video_mode_description: Vec<String>,
    #[serde(rename = "Status", deserialize_with = "deserialize_status")]
    status: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GPURefreshRate {
    #[serde(rename = "MinRefreshRate")]
    min: u32,
    #[serde(rename = "MaxRefreshRate")]
    max: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OSInfo {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Caption")]
    short_name: String,
    #[serde(rename = "Version")]
    version: String,
    #[serde(rename = "OSArchitecture")]
    os_architecture: String,
    #[serde(rename = "Status")]
    status: String,
    #[serde(rename = "CSName")]
    computer_name: String,
    #[serde(rename = "LastBootUpTime", deserialize_with = "deserialize_last_boot_up_time")]
    uptime: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemInfo {
    #[serde(default)]
    index: u8,
    #[serde(rename = "Manufacturer", default)]
    vendor: String,
    #[serde(rename = "Model", default)]
    model: String,
    #[serde(rename = "DeviceLocator", default)]
    name: String,
    #[serde(rename = "SerialNumber", default)]
    serial_number: String,
    #[serde(rename = "PartNumber", default)]
    part_number: String,
    #[serde(rename = "Capacity", deserialize_with = "deserialize_capacity")]
    total_memory: u64,
    #[serde(default)]
    free_memory: u64, // This field doesn't exist in WMI, so we'll leave it as default (0).
}

#[cfg(target_os = "windows")]
pub mod windows {
    use std::collections::HashMap;
    use crate::{CPUInfo, GPUInfo, GPURefreshRate, MemInfo, OSInfo};
    use wmi::*;
    #[allow(missing_copy_implementations)]
    impl CPUInfo {
        #[cfg(target_os = "windows")]
        pub fn fetch() -> Vec<CPUInfo> {
            let wmi_con = WMIConnection::new(
                COMLibrary::new().unwrap_or_else(
                    |err|
                        panic!("An error occurred while accessing the COM Library: {}", err)))
                .unwrap_or_else(
                    |err|
                        panic!("An error occurred while connecting to the WMI: {}", err));
            let results: Vec<CPUInfo> = wmi_con.raw_query("SELECT * FROM Win32_Processor").unwrap();
            results
        }
    }

    impl GPUInfo {
        #[cfg(target_os = "windows")]
        pub fn fetch() -> Vec<GPUInfo> {
            // Initialize COM library
            let wmi_con = WMIConnection::new(
                COMLibrary::new().unwrap_or_else(
                    |err|
                        panic!("An error occurred while accessing the COM Library: {}", err)))
                .unwrap_or_else(
                    |err|
                        panic!("An error occurred while connecting to the WMI: {}", err));
            let mut results: Vec<GPUInfo> = wmi_con.raw_query("SELECT * FROM Win32_VideoController").unwrap();

            for (i, gpu) in results.iter_mut().enumerate() {
                gpu.index = i as u8;
            }

            results
        }
    }
    impl OSInfo {
        #[cfg(target_os = "windows")]
        pub fn fetch() -> Vec<OSInfo> {
            // Initialize COM library
            let wmi_con = WMIConnection::new(
                COMLibrary::new().unwrap_or_else(
                    |err|
                        panic!("An error occurred while accessing the COM Library: {}", err)))
                .unwrap_or_else(
                    |err|
                        panic!("An error occurred while connecting to the WMI: {}", err));
            let mut results: Vec<OSInfo> = wmi_con.raw_query("SELECT * FROM Win32_OperatingSystem").unwrap();

            results
        }
    }

    impl MemInfo {
        #[cfg(target_os = "windows")]
        pub fn fetch() -> Result<Vec<MemInfo>, String> {
            // Initialize COM library
            let wmi_con = WMIConnection::new(COMLibrary::new().map_err(|err| {
                format!("Failed to initialize COM Library: {}", err)
            })?)
                .map_err(|err| format!("Failed to connect to WMI: {}", err))?;

            // Query for physical memory
            let results: Vec<MemInfo> = wmi_con
                .raw_query("SELECT * FROM Win32_PhysicalMemory")
                .map_err(|err| format!("WMI query failed: {}", err))?;

            // Add indices to each memory module
            let mut results = results;
            for (i, mem) in results.iter_mut().enumerate() {
                mem.index = i as u8;
            }

            Ok(results)
        }
    }

}

pub mod linux {

}

pub mod macos {

}



pub mod utils {
    pub mod testing {
        use std::collections::HashMap;
        use wmi::{COMLibrary, Variant, WMIConnection};
        use crate::OSInfo;

        pub fn fetch_query_based(query: &str) {
            let wmi_con = WMIConnection::new(
                COMLibrary::new().unwrap_or_else(
                    |err|
                        panic!("An error occurred while accessing the COM Library: {}", err)))
                .unwrap_or_else(
                    |err|
                        panic!("An error occurred while connecting to the WMI: {}", err));
            let results: Vec<HashMap<String, Variant>> = wmi_con.raw_query(format!("SELECT * FROM {}", query)).unwrap();
            println!("{:#?}", results);
        }
    }
    pub mod windows {
        pub(crate) mod deserialisers {
            use std::time::Duration;
            use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
            use serde::{Deserialize, Deserializer};
            use serde_json::Value;
            use crate::CPUArchitecture;

            pub(crate) fn to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value: u32 = Deserialize::deserialize(deserializer)?;
                Ok(value.to_string())
            }

            pub(crate) fn optional_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value: Option<u32> = Option::deserialize(deserializer)?;
                Ok(value.map_or_else(|| "N/A".to_string(), |v| v.to_string()))
            }

            pub(crate) fn format_frequency<'de, D>(deserializer: D) -> Result<String, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value: u32 = Deserialize::deserialize(deserializer)?;
                Ok(format!("{} MHz", value))
            }

            pub(crate) fn deserialize_architecture<'de, D>(deserializer: D) -> Result<CPUArchitecture, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value: u16 = Deserialize::deserialize(deserializer)?;
                Ok(CPUArchitecture::from(value))
            }

            // Custom deserializer for driver paths
            pub(crate) fn deserialize_drivers<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let drivers_str: String = String::deserialize(deserializer)?;
                Ok(drivers_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect())
            }

            // Custom deserializer for video modes
            pub(crate) fn deserialize_video_modes<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let mode_str: String = String::deserialize(deserializer)?;
                Ok(vec![mode_str]) // Put single mode in a Vec to match struct
            }

            // Custom deserializer for status
            pub(crate) fn deserialize_status<'de, D>(deserializer: D) -> Result<bool, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let status: String = String::deserialize(deserializer)?;
                Ok(status == "OK")
            }

            pub(crate) fn deserialize_name<'de, D>(deserializer: D) -> Result<String, D::Error>
            where
                D: Deserializer<'de>,
            {
                let raw_name: String = String::deserialize(deserializer)?;
                // Extract just the OS name part before the first '|' if it exists
                Ok(raw_name.split('|').next().unwrap_or(&raw_name).trim().to_string())
            }

            pub(crate) fn deserialize_capacity<'de, D>(deserializer: D) -> Result<u64, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                match Value::deserialize(deserializer)? {
                    Value::Number(num) => num.as_u64().ok_or_else(|| {
                        serde::de::Error::custom("Failed to convert capacity to u64")
                    }),
                    Value::String(s) => s.parse::<u64>().map_err(|_| {
                        serde::de::Error::custom(format!("Failed to parse capacity from string: {}", s))
                    }),
                    v => Err(serde::de::Error::custom(format!(
                        "Unexpected value for capacity: {:?}",
                        v
                    ))),
                }
            }

            pub(crate) fn default_endian() -> String {
                if cfg!(target_endian = "little") {
                    String::from("little")
                } else if cfg!(target_endian = "big") {
                    String::from("big")
                } else {
                    String::from("Unknown")
                }
            }

            pub(crate) fn deserialize_short_name<'de, D>(deserializer: D) -> Result<String, D::Error>
            where
                D: Deserializer<'de>,
            {
                Ok("windows".to_string())
            }

            // Custom deserializer for last boot up time
            pub(crate) fn deserialize_last_boot_up_time<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let time_str: String = String::deserialize(deserializer)?;
                let (date, _) = time_str.split_once('.').unwrap();
                NaiveDateTime::parse_from_str(date, "%Y%m%d%H%M%S").map_err(serde::de::Error::custom)
            }

            impl From<u16> for CPUArchitecture {
                fn from(value: u16) -> Self {
                    match value {
                        0 => CPUArchitecture::X86,
                        5 => CPUArchitecture::Arm,
                        9 => CPUArchitecture::X64,
                        11 => CPUArchitecture::Neutral,
                        12 => CPUArchitecture::Arm64,
                        14 => CPUArchitecture::X86OnArm64,
                        65535 => CPUArchitecture::Unknown,
                        _ => CPUArchitecture::Unknown,
                    }
                }
            }
        }
    }
}