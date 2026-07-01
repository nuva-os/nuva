/*
 * Nuva OS - Nuva OS
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


#![no_std]
#![allow(unused_features)]
#![feature(abi_x86_interrupt)]
#![feature(stmt_expr_attributes)]
#![feature(asm_experimental_arch)]
#![allow(deref_into_dyn_supertrait)]
#![allow(invalid_reference_casting)]
#![allow(overflowing_literals)]
#![allow(dangerous_implicit_autorefs)]

extern crate alloc;

// Print macros - defined at crate root before modules for maximum availability
#[macro_export]
macro_rules! log_emerg {
    ($($arg:tt)*) => { $crate::kernel::debug::printk::printk_emerg(format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! log_alert {
    ($($arg:tt)*) => { $crate::kernel::debug::printk::printk_alert(format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! log_crit {
    ($($arg:tt)*) => { $crate::kernel::debug::printk::printk_crit(format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => { $crate::kernel::debug::printk::printk_err(format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => { $crate::kernel::debug::printk::printk_warn(format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! log_notice {
    ($($arg:tt)*) => { $crate::kernel::debug::printk::printk_notice(format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => { $crate::kernel::debug::printk::printk_info(format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => { $crate::kernel::debug::printk::printk_debug(format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! pr_emerg {
    ($($arg:tt)*) => { $crate::kernel::debug::printk::printk_emerg(format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! pr_alert {
    ($($arg:tt)*) => { $crate::kernel::debug::printk::printk_alert(format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! pr_crit {
    ($($arg:tt)*) => { $crate::kernel::debug::printk::printk_crit(format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! pr_err {
    ($($arg:tt)*) => { $crate::kernel::debug::printk::printk_err(format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! pr_warn {
    ($($arg:tt)*) => { $crate::kernel::debug::printk::printk_warn(format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! pr_notice {
    ($($arg:tt)*) => { $crate::kernel::debug::printk::printk_notice(format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! pr_info {
    ($($arg:tt)*) => { $crate::kernel::debug::printk::printk_info(format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! pr_debug {
    ($($arg:tt)*) => { $crate::kernel::debug::printk::printk_debug(format_args!($($arg)*)) };
}
#[macro_export]
macro_rules! log_err {
    ($($arg:tt)*) => { $crate::kernel::debug::printk::printk_err(format_args!($($arg)*)) };
}

// Synchronization primitives
pub mod sync_oncelock;

// Kernel module
pub mod kernel;

// Hardware Abstraction Layer
pub mod hal;

// System library
pub mod syslib;

// System services
pub mod services;

// Application framework
pub mod application;

// POSIX interface (optional, only for POSIX compatibility)
#[cfg(feature = "posix")]
pub mod posix;

// Test module
#[cfg(test)]
pub mod tests;

/// Kernel entry point
/// boot_info: platform-specific boot information pointer (FDT/Multiboot2/UEFI)
pub fn kernel_main(boot_info: *const u8) -> ! {
    // Detect platform info from boot info
    let platform_info = kernel::platform::detect_platform_info(boot_info);

    // Initialize kernel with platform info
    kernel_init(&platform_info);
    
    // Start scheduler (never returns)
    start_scheduler()
}

/// Kernel initialization
fn kernel_init(info: &kernel::platform::PlatformInfo) {
    // Initialize platform detection (must be first)
    #[cfg(feature = "arm64")]
    {
        #[cfg(feature = "kirin9020")]
        kernel::arch::platform::init_platform();
        
        #[cfg(feature = "snapdragon8gen4")]
        kernel::arch::platform::init_platform();
    }
    
    #[cfg(feature = "x64")]
    {
        kernel::arch::platform::init_platform();
    }

    #[cfg(feature = "loongarch64")]
    {
        kernel::arch::platform::init_platform();
    }
    
    // Initialize memory management with platform-detected sizes
    kernel::mm::init_phys_mem(info.memory_size);
    kernel::mm::init_buddy((info.memory_size / 4096) as u32);

    // Initialize mmap subsystem
    kernel::mm::mmap::init_mmap();

    // Initialize OOM killer
    kernel::mm::oom::init_oom();
    
    // Initialize interrupts
    kernel::interrupt::init_interrupts();
    
    // Initialize scheduler with detected CPU count
    kernel::sched::init_scheduler(info.cpu_count);
    
    // Initialize file system
    kernel::fs::filesystem::init_filesystem();
    
    // Initialize POSIX (optional, only when POSIX compat enabled)
    #[cfg(feature = "posix")]
    posix::init_posix();
    
    // Initialize HAL (includes platform detection, DT/ACPI, and HAL drivers)
    hal::init_hal();
    
    // Initialize services (includes form factor manager)
    services::init_services();

    // Initialize application framework (includes adaptive layout engine)
    application::init_application_framework();
    
    log_info!("Nuva OS initialized successfully");
}

/// Start scheduler
fn start_scheduler() -> ! {
    use kernel::arch::{current_arch, CpuContext};
    use kernel::sched::{init_scheduler, get_current_task};

    loop {
        let sched = init_scheduler();
        sched.schedule();

        let task = get_current_task();
        if !task.is_null() {
            // SAFETY: task pointer from scheduler is valid
            unsafe {
                let ctx = &(*task).context;
                if ctx.pc != 0 {
                    let mut cpu_ctx = CpuContext::new();
                    cpu_ctx.regs[..31].copy_from_slice(&ctx.regs[..31]);
                    cpu_ctx.sp = ctx.sp;
                    cpu_ctx.pc = ctx.pc;
                    cpu_ctx.pstate = ctx.pstate;
                    cpu_ctx.tls_base = ctx.tpidr;
                    cpu_ctx.tls_base_ro = ctx.tpidrro;

                    let arch = current_arch();
                    arch.context().restore_context(&cpu_ctx);
                }
            }
        } else {
            let arch = current_arch();
            arch.power().cpu_idle();
        }
    }
}

// Print macros are defined in kernel/debug/printk.rs
// Removed duplicate definitions - use the ones from kernel/debug/printk.rs
