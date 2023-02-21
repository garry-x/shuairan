// Copyright 2022 Garry Xu
// SPDX-License-Identifier: Apache-2.0

use utils::{json::Json, log::LogLevel};
#[allow(unused_imports)]
use std::str::FromStr;
use super::error::{Result, Error};

// When kernel is configured with MAXSMP on, 8192 cpus are allowed.
// So we use this value.
const MAX_VCPU_DEFAULT: u32 = 8192;

macro_rules! required {
    ($object:ident, $func:ident, $prefix:expr, $str:expr) => {
        match $object.$func($str) {
            Some(value) => Ok(value),
            None => Err(Error::MissingConfig(format!("{}.{}", $prefix, $str))),
        }?
    }
}

/// CPU configurations for a virtual machine.
#[derive(Debug, PartialEq, Clone)]
pub struct CpuConfig {
    /// The number of vcpus.
    pub count: u32,
}

impl CpuConfig {
    /// Construct CpuConfig from a JSON object.
    fn from(mut json: Json) -> Result<Self> {
        let count = required!(json, take_number, "cpu", "count") as u32;
        if count == 0 || count > MAX_VCPU_DEFAULT {
            return Err(Error::IllegalConfig(format!("cpu.count={}", count)));
        }
        Ok(CpuConfig { count })
    }
}

/// Memory configurations for a virtual machine.
#[derive(Debug, PartialEq, Clone)]
pub struct MemoryConfig {
    /// The total size of VM's memory in MB.
    pub size_mib: u32,
}

impl MemoryConfig {
    /// Construct MemoryConfig form a JSON object.
    fn from(mut json: Json) -> Result<Self> {
        let size_mib = required!(
            json, 
            take_number, 
            "memory", 
            "size_mib"
        ) as u32;
        if size_mib == 0 {
            return Err(Error::IllegalConfig("memory.size_mib".to_string()));
        }
        Ok(MemoryConfig { size_mib })
    }
}

/// Configurations of a virtual device for a VM.
#[derive(Debug, PartialEq, Clone)]
pub struct DeviceConfig {
    /// The driver related to this device.
    pub driver: String,
    /// The physical source device or file related to this device.
    pub source: Option<String>,
}

impl DeviceConfig {
    /// Construct DeviceConfig from a JSON object.
    fn from(mut json: Json) -> Result<Self> {
        Ok(DeviceConfig {
            driver: required!(json, take_string, "device", "driver"),
            source: json.take_string("source"),
        })
    }
}

/// Configurations related to the operating system.
#[derive(Debug, PartialEq, Clone)]
pub struct OsConfig {
    /// Path to the kernel bzImage.
    pub kernel: Option<String>,
    /// Path to the kernel initrd.
    pub initrd: Option<String>,
    /// Path to the root file system.
    pub rootfs: Option<String>,
    /// Command line arguments for linux kernel.
    pub cmdline: Option<String>,
}

impl OsConfig {
    /// Construct OsConfig from a JSON object.
    fn from(mut json: Json) -> Result<Self> {
        Ok(OsConfig{
            kernel: json.take_string("kernel"),
            initrd: json.take_string("initrd"),
            rootfs: json.take_string("rootfs"),
            cmdline: json.take_string("cmdline")
        })
    }
}

/// Configurations related to the logger.
#[derive(Debug, PartialEq, Clone)]
pub struct LogConfig {
    /// `LogLevel` for the logger.
    pub level: Option<LogLevel>,
    /// File path for the file logger.
    pub path: Option<String>
}

impl LogConfig {
    /// Construct LogConfig from a JSON object.
    pub fn from(mut json: Json) -> Result<Self> {
        Ok(
            LogConfig {
                level: json.take_string("level").map(
                    |l| match &l[..] {
                        "Debug" | "debug" => LogLevel::Debug,
                        "Info" | "info" => LogLevel::Info,
                        "Warn" | "warn" => LogLevel::Warn,
                        "Error" | "error" => LogLevel::Error,
                        // Unrecognized config will be amended to `Debug` level.
                        _ => LogLevel::Debug
                    }
                ),
                path: json.take_string("path")
            }
        )
    }
}

/// Configurations related to the hypervisor.
#[derive(Debug, PartialEq, Clone)]
pub struct VmmConfig {
    /// Configurations for the logger.
    pub log: Option<LogConfig> 
}

impl VmmConfig {
    /// Construct VmmConfig from a JSON object.
    pub fn from(mut json: Json) -> Result<Self> {
        Ok(
            VmmConfig {
                log: match json.take_object("log") {
                    Some(obj) => Some(LogConfig::from(obj)?),
                    _ => None
                }
            }
        )
    }
}

/// Overall configurations for a virtual machine.
#[derive(Debug, PartialEq, Clone)]
pub struct VmConfig {
    /// CPU configurations for a VM.
    pub cpu: CpuConfig,
    /// Memory configurations for a VM.
    pub memory: MemoryConfig,
    /// Device configurations for a VM.
    pub device: Vec<DeviceConfig>,
    /// OS configurations for a VM.
    pub os: OsConfig,
    /// Optional VMM configurations for a VM.
    pub vmm: Option<VmmConfig>
}

impl VmConfig {
    /// Construct VmConfig form a JSON object.
    pub fn from(mut json: Json) -> Result<Self> {
        Ok(VmConfig { 
            cpu: CpuConfig::from(required!(json, take_object, "", "cpu"))?, 
            memory: MemoryConfig::from(required!(json, take_object, "", "memory"))?, 
            device: {
                let mut device = Vec::new();
                for dev in required!(json, take_array, "", "device") {
                    device.push(DeviceConfig::from(dev)?);
                }
                device
            }, 
            os: OsConfig::from(required!(json, take_object, "", "os"))?,
            vmm: match json.take_object("vmm") {
                Some(obj) => Some(VmmConfig::from(obj)?),
                _ => None
            }
        }) 
    }
    /// Construct VmConfig from loading a config file
    pub fn from_file(path: &str) -> Result<Self> {
        Self::from(Json::from_file(path)?)
    }
}

#[test]
fn test_cpu_config() {
    assert_eq!(
        CpuConfig::from(Json::from_str(r#"{ "count": 4 }"#).unwrap()),
        Ok(CpuConfig { count: 4 })
    );
    assert_eq!(
        CpuConfig::from(Json::from_str("{}").unwrap()), 
        Err(Error::MissingConfig("cpu.count".to_string()))
    );
    assert_eq!(
        CpuConfig::from(Json::from_str(r#"{ "count": 8197 }"#).unwrap()), 
        Err(Error::IllegalConfig("cpu.count=8197".to_string()))
    );
}

#[test]
fn test_memconfig() {
    assert_eq!(
        MemoryConfig::from(Json::from_str(r#"{"size_mib":1024}"#).unwrap()),
        Ok(MemoryConfig { size_mib: 1024 })
    );
    assert_eq!(
        MemoryConfig::from(Json::from_str(r#"{"size_mib": 0}"#).unwrap()),
        Err(Error::IllegalConfig("memory.size_mib".to_string()))
    );
    assert_eq!(
        MemoryConfig::from(Json::from_str(r#"{"size_mib": null}"#).unwrap()),
        Err(Error::MissingConfig("memory.size_mib".to_string()))
    );
    assert_eq!(
        MemoryConfig::from(Json::from_str(r#"{}"#).unwrap()),
        Err(Error::MissingConfig("memory.size_mib".to_string()))
    );
}

#[test]
fn test_devconfig() {
    assert_eq!(
        DeviceConfig::from(Json::from_str(
            r#"{"driver":"virtio-blk","source":"/xxx/disk.raw"}"#
        ).unwrap()),
        Ok(DeviceConfig { 
            driver: "virtio-blk".to_string(),
            source: Some("/xxx/disk.raw".to_string())
        })
    );
    assert_eq!(
        DeviceConfig::from(Json::from_str(
            r#"{ "driver":"virtio-blk" }"#
        ).unwrap()),
        Ok(DeviceConfig { 
            driver: "virtio-blk".to_string(),
            source: None
        })
    );
    assert_eq!(
        DeviceConfig::from(Json::from_str(
            r#"{}"#
        ).unwrap()),
        Err(Error::MissingConfig("device.driver".to_string()))
    );
}

#[test]
fn test_os_config() {
    assert_eq!(
        OsConfig::from(Json::from_str(
            concat!(
                r#"{ "kernel":"/xx/vmlinuz", "initrd":"/xx/initrd.img","#,
                r#""rootfs":"/xx/xxx.raw", "#,
                r#""cmdline":"console=ttyS0 reboot=k panic=1 pci=off" }"#
            ),
        ).unwrap()),
        Ok(OsConfig {
            kernel: Some("/xx/vmlinuz".to_string()),
            initrd: Some("/xx/initrd.img".to_string()),
            rootfs: Some("/xx/xxx.raw".to_string()),
            cmdline: Some("console=ttyS0 reboot=k panic=1 pci=off".to_string())
        })

    );
    assert_eq!(
        OsConfig::from(Json::from_str("{}").unwrap()),
        Ok(OsConfig {
            kernel: None, 
            initrd: None, 
            rootfs: None, 
            cmdline: None 
        })
    );
}

#[test]
fn test_vm_config() {
    assert_eq!(
        VmConfig::from(Json::from_str(
            concat!(
                r#"{"cpu":{"count":4},"memory":{"size_mib":1024},"#,
                r#""device":[{"driver":"virtio-blk","source":"/xxx/disk.raw"}],"#,
                r#""os":{"kernel":"/xx/vmlinuz", "#,
                r#""cmdline":"console=ttyS0 pci=off"},"#,
                r#""vmm":{"log":{"level":"Info","path":"/var/log/shuairan.log"}}}"#
            )
        ).unwrap()),
        Ok(VmConfig {
            cpu: CpuConfig { count: 4 },
            memory: MemoryConfig { size_mib: 1024 },
            device: vec![
                DeviceConfig {
                    driver: "virtio-blk".to_string(),
                    source: Some("/xxx/disk.raw".to_string())
                }
            ],            
            os: OsConfig {
                kernel: Some("/xx/vmlinuz".to_string()),
                initrd: None,
                rootfs: None,
                cmdline: Some("console=ttyS0 pci=off".to_string())
            },
            vmm: Some(VmmConfig {
                log: Some(LogConfig{
                    level: Some(LogLevel::Info),
                    path: Some("/var/log/shuairan.log".to_string())
                })
            })
        })
    )
}
