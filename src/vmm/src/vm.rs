// Copyright 2022 Garry Xu
// SPDX-License-Identifier: Apache-2.0

use kvm_ioctls::VmFd;
use vm_memory::mmap::GuestMemoryMmap;
use super::config::VmConfig;
use super::error::Result;

/// VmStatus represents the current status of a VM.
///
/// Lifecycle of a VM: Epoch -> Paused -> Running -> Paused / Exit.  
pub enum VmStatus {
    /// VM is created, but not yet fully initialized.  
    Epoch,
    /// VM is ready to run.  
    Paused,
    /// VM is running.  
    Running,
}

/// Contains operations and related metadata for a specific Vm.  
pub struct Vm {
    /// File desicriptor used by VM ioctl.
    fd: VmFd,
    /// Configrations for the VM and its devices.
    config: VmConfig,
    /// Use mmap as the memory backend for the VM.  
    memory: GuestMemoryMmap,
    /// Current status of the VM.  
    status: VmStatus,
}

impl Vm {
    /// Create a new VM instance with the given configuration.  
    ///
    /// ## Arguments
    /// * `fd` - File discriptor for vm ioctls, it will be owned by this VM.  
    /// * `config` - VM configuration object, it will be owned by this VM.  
    pub fn new(fd: VmFd, config: VmConfig) -> Result<Self> {
        Ok(Vm {
            fd,
            config,
            memory: GuestMemoryMmap::new(),
            status: VmStatus::Epoch,
        })
    }
}
