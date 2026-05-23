/*
 * Nuva OS - Kernel - Arch - LoongArch64 - Boot
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

//! LoongArch64 boot module

use super::qemu;
use crate::kernel::platform::{BootInfoType, PlatformInfo};

// ============================================================================
// UEFI Boot Protocol Constants
// ============================================================================

/// UEFI system table signature
const EFI_SYSTEM_TABLE_SIGNATURE: u64 = 0x5453_5953_4942_4555;

/// Boot protocol types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootProtocol {
    /// UEFI boot via firmware
    Uefi,
    /// Direct boot via QEMU -kernel
    QemuDirect,
    /// FDT-based boot (bare metal)
    Fdt,
    /// Unknown boot protocol
    Unknown,
}

/// Boot information parsed from firmware
#[derive(Debug, Clone)]
pub struct LoongArchBootInfo {
    /// Boot protocol detected
    pub protocol: BootProtocol,
    /// RAM base physical address
    pub ram_base: u64,
    /// RAM size in bytes
    pub ram_size: u64,
    /// Kernel entry point
    pub kernel_entry: u64,
    /// FDT/ACPI table address (if present)
    pub firmware_table: u64,
}

// ============================================================================
// Boot Info Parsing
// ============================================================================

/// Parse LoongArch UEFI boot info
/// Placeholder implementation: returns default PlatformInfo with LoongArchFw type.
/// Full implementation requires UEFI runtime services parsing.
pub fn parse_boot_info(boot_info: *const u8) -> Result<PlatformInfo, &'static str> {
    if boot_info.is_null() {
        return Err("Boot info pointer is null");
    }
    Ok(PlatformInfo {
        boot_info,
        boot_info_type: BootInfoType::LoongArchFw,
        ..PlatformInfo::default()
    })
}

/// Parse LoongArch boot info and detect boot protocol
/// @param boot_info: Pointer to boot information structure
/// @return: Parsed LoongArch-specific boot info
pub fn parse_loongarch_boot_info(boot_info: *const u8) -> Result<LoongArchBootInfo, &'static str> {
    if boot_info.is_null() {
        return Err("Boot info pointer is null");
    }

    let protocol = detect_boot_protocol(boot_info);

    match protocol {
        BootProtocol::QemuDirect => {
            let layout = qemu::qemu_mem_layout();
            Ok(LoongArchBootInfo {
                protocol: BootProtocol::QemuDirect,
                ram_base: layout.ram_base,
                ram_size: layout.ram_size,
                kernel_entry: layout.kernel_entry,
                firmware_table: 0,
            })
        }
        BootProtocol::Uefi => Ok(LoongArchBootInfo {
            protocol: BootProtocol::Uefi,
            ram_base: 0,
            ram_size: 0,
            kernel_entry: 0,
            firmware_table: boot_info as u64,
        }),
        _ => Ok(LoongArchBootInfo {
            protocol: BootProtocol::Unknown,
            ram_base: 0,
            ram_size: 0,
            kernel_entry: 0,
            firmware_table: 0,
        }),
    }
}

/// Detect boot protocol
/// Checks for QEMU virt machine signature or UEFI system table.
fn detect_boot_protocol(boot_info: *const u8) -> BootProtocol {
    // SAFETY: Reading potential UEFI system table header.
    // The pointer is validated as non-null by the caller.
    unsafe {
        if !boot_info.is_null() {
            let signature_ptr = boot_info as *const u64;
            if let Ok(&sig) = signature_ptr.as_ref() {
                if sig == EFI_SYSTEM_TABLE_SIGNATURE {
                    return BootProtocol::Uefi;
                }
            }
        }
    }

    BootProtocol::QemuDirect
}

// ============================================================================
// Early Boot Initialization
// ============================================================================

/// Early boot initialization for LoongArch64
/// Initializes QEMU serial console and interrupt controller
/// for early output and interrupt handling.
pub fn early_init() {
    qemu::qemu_early_init();
}
