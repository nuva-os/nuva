/*
 * Nuva OS - Kernel - Mod.Rs
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */



// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};

// Existing subdirectory modules (unchanged)
pub mod arch;
pub mod debug;
pub mod driver;
pub mod fs;
pub mod interrupt;
pub mod ipc;
pub mod media;
pub mod mm;
pub mod net;
pub mod perf;
pub mod plugin;
pub mod process;
pub mod quantum;
pub mod sched;
pub mod security;
pub mod service;
pub mod sync;
pub mod syscall;
pub mod tests;
pub mod timer;
pub mod tombstone;
pub mod user;
pub mod bsd;

// Root-level modules (not moved)
pub mod error;
pub mod panic;

// Functional domain subdirectory modules
pub mod core;
pub mod device;
pub mod diag;
pub mod init;
pub mod irq_mgmt;
pub mod net_stack;
pub mod power_mgmt;
pub mod storage;
pub mod virt;

// Backward-compatible re-exports for moved modules
pub use init::cmdline;
pub use init::config;
pub use init::elf;
pub use init::platform;
pub use init::resource;

pub use diag::journal;
pub use diag::kdebug;
pub use diag::log;
pub use diag::scanner;
pub use diag::stats;

pub use irq_mgmt::apic_ops;
pub use irq_mgmt::irq;
pub use irq_mgmt::trap;

pub use net_stack::socket;
pub use net_stack::tcpip;

pub use storage::block;

pub use device::device_model;
pub use device::driver_plugin;
pub use device::feature_plugin;
pub use device::module;
pub use device::notifier;

pub use power_mgmt::hotplug;
pub use power_mgmt::pm;
pub use power_mgmt::power;

pub use virt::vmx;

pub use core::cache;
pub use core::cpu;
pub use core::defense;
pub use core::kernel_thread;
pub use core::mempool;
pub use core::perf_tune;
pub use core::posix;
pub use core::random;
pub use core::signal;
pub use core::time;
pub use core::wait;
pub use core::workqueue;

// Re-export unified error type
pub use error::{KernelError, KernelResult};

// Re-export main modules
pub use ipc::nuvaipc;
pub use quantum::{QuantumManager, QuantumRng, QkdSession, PqcContext, init_quantum};

/// Kernel version
pub const KERNEL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Kernel name
pub const KERNEL_NAME: &str = "Nuva";

/// Kernel initialization
pub fn init() {
    log_info!("{} kernel v{}", KERNEL_NAME, KERNEL_VERSION);
    
    // Initialize subsystems
    init_subsystems();
}

/// Initialize all kernel subsystems in dependency order.
///
/// Subsystems are organized into phases:
/// Phase 1 — Bootstrap: cmdline, config, logging, CPU, debug
/// Phase 2 — Memory & IRQ: memory pool, resource, random, IRQ, time
/// Phase 3 — Device & Plugin: device model, plugins, drivers, modules
/// Phase 4 — Infrastructure: stats, hotplug, PM, perf, timer, workqueue
/// Phase 5 — Core Kernel: process, scheduler, signals, security
/// Phase 6 — Resilience: tombstone, defense, scanner, cache, perf tuning
/// Phase 7 — I/O & Net: block devices, TCP/IP, sockets, networking
/// Phase 8 — Platform: APIC, virtualization, ACPI, debugger, journaling
fn init_subsystems() {
    // Phase 1: Bootstrap — no dependencies
    log_info!("Kernel: initializing bootstrap subsystems...");
    cmdline::init_cmdline();
    config::init_config();
    log::init_log();
    cpu::init_cpu();
    debug::init_debug();

    // Phase 2: Memory management and interrupts
    log_info!("Kernel: initializing memory and IRQ...");
    mempool::init_mempool();
    resource::init_resource();
    random::init_random();
    irq::init_irq();
    time::init_time();

    // Phase 3: Device model, plugins, and driver framework
    log_info!("Kernel: initializing device and plugin subsystems...");
    device_model::init_device_model();
    plugin::init_plugin();
    driver_plugin::init_driver_plugin();
    feature_plugin::init_feature_plugin();
    module::init_module();
    notifier::init_notifier();

    // Phase 4: System infrastructure
    log_info!("Kernel: initializing infrastructure subsystems...");
    stats::init_stats();
    hotplug::init_hotplug();
    pm::init_pm();
    perf::init_perf();
    timer::init_timer();
    workqueue::init_workqueue();

    // Phase 5: Core kernel services
    log_info!("Kernel: initializing core kernel services...");
    process::init_process();
    sched::init_scheduler(1);
    signal::init_signal();
    security::init_security();

    // Phase 6: Resilience and performance
    log_info!("Kernel: initializing resilience subsystems...");
    tombstone::init_tombstone();
    defense::init_defense();
    scanner::init_virus_scanner();
    cache::init_cache();
    perf_tune::init_perf_tune();

    // Phase 7: I/O and networking
    log_info!("Kernel: initializing I/O and network subsystems...");
    block::init_block_device();
    tcpip::init_tcpip();
    socket::init_socket_api();
    net::init_net();

    // Phase 8: Platform-specific and diagnostics
    log_info!("Kernel: initializing platform and diagnostic subsystems...");
    apic_ops::init_apic_ops();
    vmx::init_vmx();
    power::init_acpi();
    kdebug::init_kdebug();
    journal::init_journal();

    log_info!("Kernel subsystems initialized successfully");
}
