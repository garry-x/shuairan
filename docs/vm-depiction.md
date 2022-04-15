# Virtual Machine Depiction

A virtual machine is the abstraction for a real physical machine. Its depiction consists of:  
- CPU Configrations  
- Memory Configrations  
- Devices Configratons  
- Operating System Related Configrations  
- Hyervisor Related configuratons  

The depiction for a VM is implemented through a JSON file. Here is a example:  
```
{
    "cpu": { "count": 2 },
    "memory": { "size_mib": 1024 },
    "device": [
        {
            "driver": "virtio-blk",
            "specific": { "path": "focal-server-cloudimg-amd64.raw" }
        },
        {
            "driver": "virtio-net",
            "specific": { "mac": "fa:16:3e:21:c0:c0" }
        },
        {
            "driver": "vfio",
            "specific": { "source": "02:00.0" } 
        },
        {
            "driver": "console",
            "specific": { "type": "tty" } 
        }
    ],
    "os": {
        "type": "Linux",
        "specific": {
            "kernel_path": "/tmp/test-vm/vmlinux.bin",
            "initrd_path": null,
            "rootfs_path": "/tmp/test-vm/bionic.rootfs.ext4", 
            "cmd_args": "console=ttyS0 reboot=k panic=1 pci=off"
        }
    },
    "vmm": {}
}
```

> For now, we only support a really simple and crude depiction. More options will be added soon.










