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
    ($object:ident, $func:ident, $str:expr) => {
        match $object.$func($str) {
            Some(value) => Ok(value),
            None => Err(Error::MissingConfig($str.to_string())),
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
        let count = required!(json, take_number, "count") as u32;
        if count == 0 || count > MAX_VCPU_DEFAULT {
            return Err(Error::IllegalConfig("cpu->count".to_string()));
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
        let size_mib = required!(json, take_number, "size_mib") as u32;
        if size_mib == 0 {
            return Err(Error::IllegalConfig("memory->size_mib".to_string()));
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
            driver: required!(json, take_string, "driver"),
            source: json.take_string("driver"),
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
    pub devices: Vec<DeviceConfig>,
    /// OS configurations for a VM
    pub os: OSConfig,
    /// Hypervisor configurations
    pub vmm: VMMConfig
}

impl VMConfig {
    /// Construct VMConfig from a JSON object.
    pub fn from_file(path: &str) -> Result<Self> {
        let mut json = Json::from_file(path)?;
        let cpu = CPUConfig::from(required!(json, take_object, "cpu"))?;
        let memory = MemoryConfig::from(required!(json, take_object, "memory"))?;
        let mut devices = Vec::new();
        for dev in required!(json, take_array, "device") {
            devices.push(DeviceConfig::from(dev)?);
        }
        let os = OSConfig::from(required!(json, take_object, "os"))?;
        let vmm = VMMConfig {};
        Ok(VMConfig { cpu, memory, devices, os, vmm }) 
    }
}