/*
 * Nuva OS - Kernel - Platform Info
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

//! Platform information abstraction for multi-arch boot

/// Boot information type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootInfoType {
    /// Flattened Device Tree (ARM64)
    Fdt,
    /// ACPI (x86_64)
    Acpi,
    /// Multiboot2 (x86_64)
    Multiboot2,
    /// LoongArch UEFI firmware info
    LoongArchFw,
    /// No boot info available
    None,
}

/// Platform information parsed from boot info
#[derive(Debug, Clone, Copy)]
pub struct PlatformInfo {
    /// Physical memory base address
    pub memory_base: u64,
    /// Physical memory size in bytes
    pub memory_size: u64,
    /// Number of CPUs
    pub cpu_count: u32,
    /// Raw boot info pointer
    pub boot_info: *const u8,
    /// Boot info type
    pub boot_info_type: BootInfoType,
}

impl Default for PlatformInfo {
    fn default() -> Self {
        Self {
            memory_base: 0,
            memory_size: 128 * 1024 * 1024,
            cpu_count: 1,
            boot_info: core::ptr::null(),
            boot_info_type: BootInfoType::None,
        }
    }
}

/// Detect platform info from boot info pointer
/// Dispatches to architecture-specific parsers based on compile-time features.
/// Returns default PlatformInfo if parsing fails.
pub fn detect_platform_info(boot_info: *const u8) -> PlatformInfo {
    if boot_info.is_null() {
        return PlatformInfo::default();
    }

    #[cfg(feature = "arm64")]
    {
        crate::kernel::arch::arm64::boot::fdt::extract_platform_info(boot_info)
            .unwrap_or_default()
    }

    #[cfg(feature = "x64")]
    {
        crate::kernel::arch::x64::boot::multiboot2::parse_multiboot2_info(boot_info)
    }

    #[cfg(feature = "loongarch64")]
    {
        PlatformInfo {
            boot_info,
            boot_info_type: BootInfoType::LoongArchFw,
            ..PlatformInfo::default()
        }
    }

    #[cfg(not(any(feature = "arm64", feature = "x64", feature = "loongarch64")))]
    {
        let _ = boot_info;
        PlatformInfo::default()
    }
}
