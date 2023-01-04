# Design    

## Structures  and Operations  

Structures and operations are divied into three levels:  
| | Scope |  
| - | - |  
| Hypervisor Level |  State outside of a VM  |  
| VM Level | State  inside of a VM |  
| Device Level | State of a specific device|  

## Device Level

### struct VcpuManager & Vcpu
VcpuManager manages all the virtual vcpus owned by a VM, while Vcpu stands for a specific virtual cpu.  Other differences:  
- `struct VcpuManger`: owned by the control thread.  
- `struct Vcpu`: owned by a specific vcpu thread.  

## Thread Model

We have main thread and a vcpu for each vcpu.  Main thread is used to 
