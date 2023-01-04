// Copyright 2022 Garry Xu
// SPDX-License-Identifier: Apache-2.0

pub mod config;
pub mod error;
pub mod vcpu;
pub mod vm;

use kvm_ioctls::Kvm;
use config::{VmConfig, VmmConfig};
use error::{Result};
use vm::Vm;

/// Contains operations and metadata needed for the hypervisor.
pub struct Vmm {
    /// VMM configurations.
    config: Option<VmmConfig>,
    /// Used for KVM system level ioctls.
    kvm: Kvm,
    /// Inside virtual machine.
    pub vm: Vm
}

impl Vmm {
    fn new(mut config: VmConfig) -> Result<Self> {
        let kvm = Kvm::new()?;
        let fd = kvm.create_vm()?;
        Ok(
            Vmm {
                config: config.vmm.take(),
                kvm,
                vm: Vm::new(fd, config)?
            }
        )
    }
}