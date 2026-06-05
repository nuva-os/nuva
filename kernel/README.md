# Kernel — Kernel Layer (L1)

## Overview

The Kernel layer (Layer 1) is the core of Nuva OS, designed with a **three-level microkernel architecture**. It depends only on the HAL (L0) layer and provides core subsystems including scheduling, memory management, IPC, driver framework, and security.

## Three-Level Privilege Architecture

| Level | Name | Components | HW Mapping |
|-------|------|------------|------------|
| EL2 | Minimal Kernel | Scheduler, IPC, MM, CapMgr, IRQ, Timer | ARM64 EL2 / x64 Ring 0 / RISC-V M / LA64 PLV0 |
| EL1 | Equipment Mode | Filesystem, Network, Drivers, Display | ARM64 EL1 / x64 Ring 1 / RISC-V S / LA64 PLV1 |
| EL0 | User Mode | Applications | ARM64 EL0 / x64 Ring 3 / RISC-V U / LA64 PLV3 |

- **NvSupervisorCall**: The only controlled EL1→EL2 interface (capability-gated)
- **NvCrossLevelAccessEnforcement**: Direct cross-level memory access always denied
- **NvEquipmentFaultDomain**: Independent fault domain per EL1 service, crash never affects kernel

## Submodules

| Submodule | Description |
|-----------|-------------|
| arch/ | Architecture-specific code (arm64, x64, loongarch64, riscv64): boot, context switch, MMU, interrupts, linker scripts, exception vectors |
| platform | Platform detection (PlatformInfo, BootInfoType: Fdt/Acpi/Multiboot2/LoongArchFw) |
| mm/ | Memory management: Buddy allocator, SLAB, page cache, VMA, mmap, munmap, mprotect, msync, COW, NUMA, huge pages, OOM killer |
| mempool | Memory pool management |
| cache | Kernel cache system |
| block | Block device subsystem |
| sched/ | Scheduler: CFS, EAS (Energy-Aware Scheduling), RT real-time scheduling, AI scheduler integration, red-black tree, scheduling domains, load balancing, NvScheduler (AI intelligent), NvBalancer (heterogeneous HW), NvPowerMgr cooperation |
| process | Process management: fork, execve, wait4, signal handling, complete creation/destruction lifecycle |
| workqueue | Work queue |
| sync/ | Synchronization primitives: spinlock, mutex, atomic operations |
| irq | IRQ interrupt request management |
| interrupt/ | Interrupt management: generic interrupt handling, GIC |
| trap | Trap/exception handling |
| ipc/ | Inter-process communication: NuvaIPC (fast path, zero-copy), shared memory, L4 IPC, quantum-secure IPC |
| net/ | Network stack: TCP/UDP/ICMP/IPv6/ARP/Ethernet, routing, firewall, Socket, Netlink, NFS/SMB network client |
| tcpip | TCP/IP stack initialization |
| socket | Socket API |
| fs/ | Kernel file system: VFS (open/close/read/write/lseek/mkdir/unlink), buffer, directory cache, page cache, io_uring, WAL, COW, Snapshot |
| journal | Journaling system |
| driver/ | Driver framework: device model, bus, IRQ (auto-detect GIC/APIC/EIOINTC), DMA, GPIO, I2C, SPI, clock, regulator |
| device_model | Device model |
| driver_plugin | Driver plugin system |
| plugin/ | Plugin system: ELF loader full implementation, manager, registry, sandbox, SHA-256 verification, core plugins |
| module | Kernel module loader |
| feature_plugin | Feature plugin system |
| elf | ELF parser |
| security/ | Security subsystem: LSM, ASLR, sandbox, stack canary |
| defense | Defense system |
| scanner | Virus scanner |
| quantum/ | Quantum computing support: quantum manager, quantum scheduler |
| debug/ | Kernel debugging: printk macros (pr_err!/pr_info!/pr_warn!, etc.) |
| kdebug | Kernel debugger |
| log | Kernel logging system |
| perf/ | Performance monitoring: event counting, performance monitor |
| perf_tune | Performance tuning |
| syscall/ | System call interface |
| timer/ | Timer subsystem |
| time | Time subsystem |
| cpu | CPU management |
| hotplug | Hot-plug |
| power | Power management (ACPI) |
| pm | Power manager |
| config | Kernel configuration |
| cmdline | Kernel command line |
| random | Random number generation |
| resource | Resource manager |
| signal | Signal handling |
| stats | Statistics |
| notifier | Notifier chain |
| apic_ops | APIC operations |
| vmx | Virtualization support |
| posix | Kernel POSIX compatibility |
| capability/ | Nuva native capability model: NvCapability tokens, NvCapabilityManager, NvRightsSet (replaces uid/gid + LSM) |
| nv_process/ | Nuva native process model: nv_process_spawn, nv_process_execute (replaces fork/execve) |
| nv_event/ | Nuva native event notification: NvEvent + NvNotificationPort (replaces POSIX signals) |
| equipment/ | Equipment mode fault domain: NvEquipmentFaultDomain, NvEquipmentMonitor, NvEquipmentRecovery (7-step auto-restart) |
| core/privilege.rs | Three-level privilege: NvPrivilegeLevel, NvArchPrivilegeMapping (ARM64/X64/RISC-V/LA64) |
| core/supervisor_call.rs | NvSupervisorCall: EL1→EL2 capability-gated interface (14 operations) |
| core/cross_level.rs | NvCrossLevelAccessEnforcement: cross-level memory access isolation |
| sched/nv_policy.rs | Nuva native scheduling policies: NvSchedPolicy (7 types including NvDeadline, NvEnergyAware, NvEquipment) |
| mm/region.rs | Nuva native memory regions: NvMemoryRegion with capability-controlled access |

## Dependencies

- **Lower dependencies**: hal (L0)
- **Depended by**: syslib (L2), services (L3)

## Build Configuration

The kernel supports multiple architectures via conditional compilation:

- `--features arm64`: ARM64 architecture (with Kirin/Snapdragon SoC support)
- `--features x64`: x86_64 architecture
- `--features loongarch64`: LoongArch64 architecture
- `--features riscv64`: RISC-V 64 architecture (boot/SBI, trap, MMU, PLIC, timer, context)
- `--features smp`: Symmetric multiprocessing support
- `--features debug`: Debug mode

## Public Interface

The kernel exposes interfaces to upper layers through system calls (syscall) and kernel API. The main entry point is the `kernel_main(boot_info: *const u8) -> !` function, which receives boot information and detects platform info via `detect_platform_info()`.
