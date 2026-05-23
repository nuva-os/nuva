/*
 * Nuva OS - HAL - Mod.Rs
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



// CPU HAL (generic)
// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
pub mod cpu;

// Input HAL
pub mod input;

// FFI (Foreign Function Interface)
pub mod ffi;

// GPU HAL
pub mod gpu;

// NPU HAL
pub mod npu;

// Power management HAL
pub mod power;

// Platform detection and identification
pub mod platform;

// Quantum technology HAL
pub mod quantum;

// Device Tree parser (ARM64)
#[cfg(feature = "arm64")]
pub mod dt;

// ACPI table parser (x86_64)
#[cfg(feature = "x64")]
pub mod acpi;

// ARM64 HAL
#[cfg(feature = "arm64")]
pub mod arm64;

// X64 HAL
#[cfg(feature = "x64")]
pub mod x64;

// LoongArch64 HAL
#[cfg(feature = "loongarch64")]
pub mod loongarch64;

// Snapdragon HAL
#[cfg(feature = "snapdragon8gen4")]
pub mod snapdragon;

/// Initialize HAL
pub fn init_hal() {
    // Step 1: Detect platform at runtime (architecture, SoC, form factor)
    platform::detect_platform();

    // Step 2: Initialize hardware discovery
    #[cfg(feature = "arm64")]
    {
        // Parse Device Tree for hardware discovery on ARM64
        dt::init_dt();
    }

    #[cfg(feature = "x64")]
    {
        // Parse ACPI tables for hardware discovery on x86_64
        acpi::init_acpi();
    }

    // Step 3: Initialize platform-specific HAL
    #[cfg(feature = "arm64")]
    {
        #[cfg(any(feature = "kirin", feature = "kirin9000", feature = "kirin9010", feature = "kirin9020"))]
        {
            // Kirin HAL initialize
            cpu::kirin::get_kirin_hal().init();
        }

        #[cfg(feature = "snapdragon8gen4")]
        {
            crate::hal::snapdragon::cpu::init_cpu_hal();
            crate::hal::snapdragon::gpu::init_gpu_hal();
            crate::hal::snapdragon::npu::init_npu_hal();
        }
    }

    #[cfg(feature = "x64")]
    {
        crate::hal::x64::cpu::init_cpu_hal();
        crate::hal::x64::apic::init_apic();
        crate::hal::x64::mmu::init_mmu();
    }

    #[cfg(feature = "loongarch64")]
    {
        #[cfg(any(feature = "loongson3a6000", feature = "loongson3c6000"))]
        {
            // Loongson HAL initialize
            cpu::loongson::get_loongson_hal().init();
        }
    }

    // Step 4: Log final platform summary
    let info = platform::get_platform_info();
    log_info!("HAL initialized: {:?} {:?} {:?}", info.arch, info.soc, info.form_factor);
}
