# Design    

## Structures  and Operations  

Structures and operations are divied into three levels as below:

| | Scope | Structures |
| - | - | - |
| Hypervisor Level |  State and Ops outside of a VM  | Vmm |
| VM Level | State and Ops inside of a VM | Vm |
| Device Level | State and Ops of a specific device | VcpuManager Vcpu |


## Device Level

### struct VcpuManager & Vcpu
VcpuManager manages all the virtual vcpus owned by a VM, while Vcpu stands for a specific virtual cpu.  Other differences:  
- `struct VcpuManger`: owned by the control thread.
- `struct Vcpu`: owned by a specific vcpu thread.

#### Vcpu Status
![](/docs/images/shuairan-vcpu-state-machine.png)

## Thread Model




We have main thread and a vcpu for each vcpu.  Main thread is used to 
