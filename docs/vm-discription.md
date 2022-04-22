# Virtual Machine Discription

A virtual machine is the abstraction for a real physical machine. Its description consists of:  
- CPU Configrations  
- Memory Configrations  
- Devices Configratons  
- Operating System Related Configrations  
- Hyervisor Related configuratons  

The discription for a VM is implemented through a JSON file. Here is a example:  
```
{
    "cpu": { "count": 2 },
    "memory": { "size_mib": 1024 },
    "device": [
        {
            "driver": "virtio-blk",
            "source": "focal-server-cloudimg-amd64.raw"
        },
        {
            "driver": "virtio-net",
            "mac": "fa:16:3e:21:c0:c0"
        },
        {
            "driver": "vfio",
            "source": "02:00.0" 
        }
    ],
    "os": {
        "kernel": "/tmp/test-vm/vmlinux.bin",
        "initrd": null,
        "rootfs": "/tmp/test-vm/bionic.rootfs.ext4", 
        "cmdline": "console=ttyS0 reboot=k panic=1 pci=off"
    },
    "vmm": {}
}
```

> For now, we only support a really simple and crude discription. More options will be added soon.










