# Kernel Initialization Subsystem

## Overview

The `kernel/init/` module handles early kernel initialization, including command line parsing, platform configuration, ELF loading, and resource management.

## Module Structure

| File | Description |
|------|-------------|
| `mod.rs` | Module entry point and initialization orchestration |
| `cmdline.rs` | Kernel command line parsing |
| `config.rs` | Kernel configuration management |
| `elf.rs` | ELF binary loader for kernel modules |
| `platform.rs` | Platform-specific initialization |
| `resource.rs` | Resource management and allocation |

## Initialization Order

The init module provides the bootstrap components that kick off kernel initialization. The full kernel boot sequence is organized into 8 phases, orchestrated by `kernel::init_subsystems()`:

**Phase 1 — Bootstrap** (no dependencies):
1. Command line parsing (`cmdline::init_cmdline`)
2. Configuration setup (`config::init_config`)
3. Logging (`log::init_log`)
4. CPU management (`cpu::init_cpu`)
5. Debug subsystem (`debug::init_debug`)

**Phase 2 — Memory & IRQ** (depends on Phase 1):
6. Memory pool (`mempool::init_mempool`)
7. Resource manager (`resource::init_resource`)
8. Random number generator (`random::init_random`)
9. IRQ management (`irq::init_irq`)
10. Time keeping (`time::init_time`)

**Phase 3 — Device & Plugin** (depends on Phase 2):
11. Device model (`device_model::init_device_model`)
12. Plugin system (`plugin::init_plugin`)
13. Driver plugins (`driver_plugin::init_driver_plugin`)
14. Feature plugins (`feature_plugin::init_feature_plugin`)
15. Module loader (`module::init_module`)
16. Notifier chain (`notifier::init_notifier`)

**Phase 4 — Infrastructure** (depends on Phase 3):
17. Statistics (`stats::init_stats`)
18. Hot-plug (`hotplug::init_hotplug`)
19. Power management (`pm::init_pm`)
20. Performance monitoring (`perf::init_perf`)
21. Timer subsystem (`timer::init_timer`)
22. Workqueue (`workqueue::init_workqueue`)

**Phase 5 — Core Kernel Services** (depends on Phase 4):
23. Process management (`process::init_process`)
24. Scheduler (`sched::init_scheduler`)
25. Signal handling (`signal::init_signal`)
26. Security subsystem (`security::init_security`)

**Phase 6 — Resilience & Perf** (depends on Phase 5):
27. Tombstone (`tombstone::init_tombstone`)
28. Defense mechanisms (`defense::init_defense`)
29. Virus scanner (`scanner::init_virus_scanner`)
30. Kernel cache (`cache::init_cache`)
31. Performance tuning (`perf_tune::init_perf_tune`)

**Phase 7 — I/O & Networking** (depends on Phase 6):
32. Block device (`block::init_block_device`)
33. TCP/IP stack (`tcpip::init_tcpip`)
34. Socket API (`socket::init_socket_api`)
35. Network subsystem (`net::init_net`)

**Phase 8 — Platform & Diagnostics** (depends on Phase 7):
36. APIC operations (`apic_ops::init_apic_ops`)
37. Virtualization (`vmx::init_vmx`)
38. ACPI power (`power::init_acpi`)
39. Kernel debugger (`kdebug::init_kdebug`)
40. Journaling (`journal::init_journal`)

## Dependencies

- **Lower dependencies**: `kernel/core` (CPU, mempool), HAL (L0)
- **Depended by**: All other kernel subsystems — init provides the foundational boot configuration and resource management that every kernel component relies on

## Public Interface

- `cmdline` module: Kernel command line parsing (`init_cmdline()`, `get_cmdline()`, `get_boot_arg()`)
- `config` module: Kernel configuration management (`init_config()`, `get_config()`, `set_config()`)
- `elf` module: ELF binary loader for parsing and loading kernel modules and plugins
- `platform` module: Platform-specific initialization and detection (`detect_platform_info()`)
- `resource` module: Resource allocation and management (`init_resource()`, `allocate_resource()`, `free_resource()`)

## Backward-Compatible Re-exports

The following items are re-exported from `kernel/` for backward compatibility:
- `cmdline`, `config`, `elf`, `platform`, `resource`

---

*Last updated: 2026-05-20 | Nuva OS v1.0.0*
