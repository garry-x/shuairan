// Copyright 2022 Garry Xu
// SPDX-License-Identifier: Apache-2.0

use std::{
    thread::{self, JoinHandle},
    sync::mpsc::{Receiver, Sender, channel},
};
use kvm_ioctls::{VcpuFd, VmFd};
use super::config::CpuConfig;
use super::error::Result;

/// Status for the current vcpu. 
/// 
/// The lifecycle of a normal vcpu will be: epoch -> paused -> running ->| exit  
pub enum VcpuStatus {
    /// Vcpu is created and needs further initialization.  
    Epoch,
    /// Vcpu is ready to run.  
    Paused,
    /// Vcpu is now running. 
    Running
}

/// Input & output message for manipulating or querying information for the vcpu.
/// 
///  Note: messages can only be consumed by vcpus before VM enter or after VM exit. 
pub enum VcpuMsg {
    /// Tell vcpu to run. If the vcpu is not in 'epoch' or 'paused' status, 
    /// this message will be ignored.  
    Run,
    /// Reply for 'Run' message, current status of the vcpu is returned. 
    RunReply(VcpuStatus),
    /// Tell vcpu to exit.  
    Exit
}

/// VcpuManager contains operations and metadata for all the vcpus for a vm.  
/// 
/// Instance of VcpuManager should be owned by the control thread.  
pub struct VcpuManager {
    /// Configuration for VM's vcpus.
    config: CpuConfig,
    /// List of vcpu thread handles.  
    threads: Vec<JoinHandle<()>>,
    /// Vcpus' input channels.
    chs_in_send: Vec<Sender<VcpuMsg>>,
    /// Vcpus' output channels.
    chs_out_recv: Vec<Receiver<VcpuMsg>>,
}

impl VcpuManager {
    /// Create a new manager for VM's all vcpus.
    /// 
    /// # Arguments
    /// 
    /// * `fd` - File discriptor for VM ioctls.
    /// * `config` - Configuration for VM's vcpus.
    pub fn new(fd: VmFd, config: CpuConfig) -> Result<Self> {
        let mut threads = Vec::new();
        let mut chs_in_send = Vec::new();
        let mut chs_out_recv = Vec::new();
        for i in 0..config.count {
            let fd = fd.create_vcpu(i as u64)?;
            let (ch_in_send, ch_in_recv) = channel();
            let (vcpu, ch_out_recv) = Vcpu::new(i, fd, ch_in_recv);
            threads.push(thread::spawn(move || {
                Vcpu::run(vcpu);
            }));
            chs_in_send.push(ch_in_send);
            chs_out_recv.push(ch_out_recv);
        }
        Ok(
            VcpuManager {
                config,
                threads,
                chs_in_send,
                chs_out_recv,
            }
        )
    }
}

/// Vcpu contains operations and metadata for a specific vcpu.
/// Vcpu should be owned by the vcpu thread.
/// 
/// Each vcpu thread runs continuously except for following reasons:
/// - After having been properly initialized, the vcpu waits for a 'Run' message.
/// - A 'Pause' or 'Exit' message  is received.
/// 
/// Vcpu communicates with the outer world through two channels:
/// - Receive channel: receive control / query messages from main thread.
/// - Send channel: send reply / info messages to the main thread.
pub struct Vcpu {
    /// ID for the vcpu, should be unique.
    id: u32,
    /// File descriptor for vcpu ioctls.
    fd: VcpuFd,

    /// Receiver for input channel.
    ch_in_recv: Receiver<VcpuMsg>,
    /// Sender for the output channel.
    ch_out_send: Sender<VcpuMsg>
}

impl Vcpu {
    /// Create a vcpu instance.
    /// 
    /// # Arguments
    /// - `id` - Vcpu id.
    /// - `fd` - File descriptor for vcpu ioctls.
    /// - `ch_in_recv` - The receiver for the input channel.
    fn new(id: u32, fd: VcpuFd, ch_in_recv: Receiver<VcpuMsg>) -> (Self, Receiver<VcpuMsg>) {
        let (ch_out_send, ch_out_recv) = channel();
        (
            Vcpu {
                id,
                fd,
                ch_in_recv,
                ch_out_send
            },
            ch_out_recv
        )
        
    }

    /// Vcpu's main loop.
    fn run(vcpu: Vcpu) {
        todo!()
    }
}
