# Direct Kernel Boot

We support directly booting a Linux kernel to demonstrate Shuairan'a basic functions.   

### Virtual Machine Discription

```
{
    "cpu": { "count": 2 },
    "memory": { "size_mib": 1024 },
    "device": [],
    "os": {
        "kernel_path": "/tmp/test-vm/vmlinux.bin",
        "initrd_path": null,
        "rootfs_path": "/tmp/test-vm/bionic.rootfs.ext4", 
        "cmd_args": "console=ttyS0 reboot=k panic=1 pci=off"
    },
    "vmm": {}
}
```