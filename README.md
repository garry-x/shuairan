# ShuaiRan (率然)

ShuaiRan (率然) is a hypervisor designed for bare-metal servers.  
![](https://www.garryx.com/images/shuairan.png)


### Background  

Traditional bare-metal servers have following drawbacks:  
- **Storage Flexibility:** The number and the capacity of disks are limited by the physical resources. Therefore, when your workloads change, it will be difficult to optimize your storage plans.  
- **Data Reliability:** The reliability of your written data is determined by the reliablility of the physical disk. When some disk crashes, you may face the risk of losing data.  
- **Error Recovery:** When a physical server encounters some fatal errors, it's hard to immediately recover your services on other server because of the lack of needing data.

To address the above issues, we put a lightweight hypervisor (ShuaiRan) between the user operating system and the underlying physical devices. The designing goals of shuairan are listed as follows:  
- Performance of the VM is comparable with the original physical server.   
- The resource costs of ShuaiRan hypervisor is unnoticeable.  
- Elastic Block Storage is suported with the help of DPUs or Smartnics.   
- Live migration is supported.  

