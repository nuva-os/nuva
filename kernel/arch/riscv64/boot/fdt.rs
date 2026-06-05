/*
 * Nuva OS - Kernel - RISC-V 64 FDT Support
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

//! Flattened Device Tree (FDT) parsing support for RISC-V.
//! Stores the FDT physical address passed by firmware and validates the magic number.

use core::sync::atomic::{AtomicU64, Ordering};

/// FDT magic number (0xd00dfeed big-endian).
const FDT_MAGIC: u32 = 0xd00dfeed;

static FDT_ADDR: AtomicU64 = AtomicU64::new(0);

/// Save FDT physical address and validate magic number.
pub fn init_fdt(fdt_addr: *const u8) {
    if fdt_addr.is_null() {
        log_warn!("RISC-V: FDT address is null, using QEMU virt defaults");
        FDT_ADDR.store(0, Ordering::SeqCst);
        return;
    }

    // Validate FDT magic number
    let magic = unsafe { *(fdt_addr as *const u32) };
    if u32::from_be(magic) != FDT_MAGIC {
        log_warn!("RISC-V: Invalid FDT magic: {:#x}, using defaults", magic);
        FDT_ADDR.store(0, Ordering::SeqCst);
        return;
    }

    FDT_ADDR.store(fdt_addr as u64, Ordering::SeqCst);
    log_info!("RISC-V: FDT at {:#x}", fdt_addr as u64);
}

/// Get the stored FDT physical address.
pub fn get_fdt() -> *const u8 {
    FDT_ADDR.load(Ordering::SeqCst) as *const u8
}
