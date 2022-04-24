//
// Copyright 2022 Garry Xu
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt;
use utils::json;
use utils::json::Json;
#[allow(unused_imports)]
use std::str::FromStr;

const MAX_VCPU_DEFAULT: u32 = 512;

type Result<T> = std::result::Result<T, Error>;

/// Errors generated during parsing and validating VM configurations.
#[derive(Debug, PartialEq)]
pub enum Error {
    /// The required configuration is missing.
    MissingConfig(String),
    /// The configuration provided is illegal.
    IllegalConfig(String),
    /// Errors generated when paring configurations from JSON strings.
    ParsingError(String),
    /// Errors generated when doing file operations.
    IOError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;

        match self {
            MissingConfig(s) => write!(
                f, 
                "The required configuration for {} is missing.", 
                s
            ),
            IllegalConfig(s) => write!(
                f, 
                "The given configuration for {} is illegal.", 
                s
            ),
            ParsingError(s) => write!(f, "{}", s),
            IOError(s) => write!(f, "{}", s),
        }
    }
}

impl From<json::Error> for Error {
    // Convert a json::Error to config::Error
    fn from(e: json::Error) -> Self {
        match e {
            json::Error::ParsingError(_) => Error::ParsingError(e.to_string()),
            json::Error::IOError(s) => Error::IOError(s),
        }
    }
}

macro_rules! required {
    ($object:ident, $func:ident, $prefix:expr, $str:expr) => {
        match $object.$func($str) {
            Some(value) => Ok(value),
            None => Err(Error::MissingConfig(format!("{}.{}", $prefix, $str))),
        }?
    };
}

/// CPU configurations for a virtual machine.
#[derive(Debug, PartialEq)]
pub struct CPUConfig {
    /// The number of vcpus.
    pub count: u32,
}

impl CPUConfig {
    /// Construct CPUConfig from a JSON object.
    fn from(mut json: Json) -> Result<Self> {
        let count = required!(json, take_number, "cpu", "count") as u32;
        if count == 0 || count > MAX_VCPU_DEFAULT {
            return Err(Error::IllegalConfig("cpu.count".to_string()));
        }
        Ok(CPUConfig { count })
    }
}

/// Memory configurations for a virtual machine.
#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq)]
pub struct OSConfig {
    /// Path to the kernel bzImage.
    pub kernel: Option<String>,
    /// Path to the kernel initrd.
    pub initrd: Option<String>,
    /// Path to the root file system.
    pub rootfs: Option<String>,
    /// Command line arguments for linux kernel.
    pub cmdline: Option<String>,
}

impl OSConfig {
    /// Construct OSConfig from a JSON object.
    fn from(mut json: Json) -> Result<Self> {
        Ok(OSConfig{
            kernel: json.take_string("kernel"),
            initrd: json.take_string("initrd"),
            rootfs: json.take_string("rootfs"),
            cmdline: json.take_string("cmdline")
        })
    }
}

/// Configurations related to the hypervisor.
#[derive(Debug, PartialEq)]
pub struct VMMConfig {}

/// Overall configurations for a virtual machine.
#[derive(Debug, PartialEq)]
pub struct VMConfig {
    /// CPU configurations for a VM.
    pub cpu: CPUConfig,
    /// Memory configurations for a VM.
    pub memory: MemoryConfig,
    /// Device configurations for a VM.
    pub device: Vec<DeviceConfig>,
    /// OS configurations for a VM
    pub os: OSConfig,
    /// Hypervisor configurations
    pub vmm: VMMConfig
}

impl VMConfig {
    /// Construct VmConfig form a JSON object.
    pub fn from(mut json: Json) -> Result<Self> {
        let cpu = CPUConfig::from(required!(json, take_object, "", "cpu"))?;
        let memory = MemoryConfig::from(required!(json, take_object, "", "memory"))?;
        let mut device = Vec::new();
        for dev in required!(json, take_array, "", "device") {
            device.push(DeviceConfig::from(dev)?);
        }
        let os = OSConfig::from(required!(json, take_object, "", "os"))?;
        let vmm = VMMConfig {};
        Ok(VMConfig { cpu, memory, device, os, vmm }) 
    }
    /// Construct VMConfig from loading a config file
    pub fn from_file(path: &str) -> Result<Self> {
        Self::from(Json::from_file(path)?)
    }
}

#[test]
fn test_cpuconfig() {
    assert_eq!(
        CPUConfig::from(Json::from_str(r#"{ "count": 4 }"#).unwrap()),
        Ok(CPUConfig { count: 4 })
    );
    assert_eq!(
        CPUConfig::from(Json::from_str("{}").unwrap()), 
        Err(Error::MissingConfig("cpu.count".to_string()))
    );
    assert_eq!(
        CPUConfig::from(Json::from_str(r#"{ "count": 2048 }"#).unwrap()), 
        Err(Error::IllegalConfig("cpu.count".to_string()))
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
fn test_osconfig() {
    assert_eq!(
        OSConfig::from(Json::from_str(
            concat!(
                r#"{ "kernel":"/xx/vmlinuz", "initrd":"/xx/initrd.img","#,
                r#""rootfs":"/xx/xxx.raw", "#,
                r#""cmdline":"console=ttyS0 reboot=k panic=1 pci=off" }"#
            ),
        ).unwrap()),
        Ok(OSConfig {
            kernel: Some("/xx/vmlinuz".to_string()),
            initrd: Some("/xx/initrd.img".to_string()),
            rootfs: Some("/xx/xxx.raw".to_string()),
            cmdline: Some("console=ttyS0 reboot=k panic=1 pci=off".to_string())
        })

    );
    assert_eq!(
        OSConfig::from(Json::from_str("{}").unwrap()),
        Ok(OSConfig {
            kernel: None, 
            initrd: None, 
            rootfs: None, 
            cmdline: None 
        })
    );
}

#[test]
fn test_vmconfig() {
    assert_eq!(
        VMConfig::from(Json::from_str(
            concat!(
                r#"{"cpu":{"count":4},"memory":{"size_mib":1024},"#,
                r#""device":[{"driver":"virtio-blk","source":"/xxx/disk.raw"}],"#,
                r#""os":{"kernel":"/xx/vmlinuz", "#,
                r#""cmdline":"console=ttyS0 pci=off"}}"#
            )
        ).unwrap()),
        Ok(VMConfig {
            cpu: CPUConfig { count: 4 },
            memory: MemoryConfig { size_mib: 1024 },
            device: vec![
                DeviceConfig {
                    driver: "virtio-blk".to_string(),
                    source: Some("/xxx/disk.raw".to_string())
                }
            ],            
            os: OSConfig {
                kernel: Some("/xx/vmlinuz".to_string()),
                initrd: None,
                rootfs: None,
                cmdline: Some("console=ttyS0 pci=off".to_string())
            },
            vmm: VMMConfig {}
        })
    )
}