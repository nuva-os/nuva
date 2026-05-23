/*
 * Nuva OS - Kernel - Arch - x64 - GDT
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

//! x86_64 Global Descriptor Table (GDT)

use core::arch::asm;

/// GDT entry (8 bytes)
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct GdtEntry {
    pub limit_low: u16,
    pub base_low: u16,
    pub base_middle: u8,
    pub access: u8,
    pub flags_limit_high: u8,
    pub base_high: u8,
}

impl GdtEntry {
    pub const fn new(base: u32, limit: u32, access: u8, flags: u8) -> Self {
        Self {
            limit_low: (limit & 0xFFFF) as u16,
            base_low: (base & 0xFFFF) as u16,
            base_middle: ((base >> 16) & 0xFF) as u8,
            access,
            flags_limit_high: ((flags & 0x0F) << 4) | ((limit >> 16) & 0x0F) as u8,
            base_high: ((base >> 24) & 0xFF) as u8,
        }
    }

    pub const fn null() -> Self {
        Self {
            limit_low: 0,
            base_low: 0,
            base_middle: 0,
            access: 0,
            flags_limit_high: 0,
            base_high: 0,
        }
    }
}

/// Segment selectors
pub const KERNEL_CODE: u16 = 0x08;
pub const KERNEL_DATA: u16 = 0x10;
pub const USER_CODE: u16 = 0x18;
pub const USER_DATA: u16 = 0x20;
pub const TSS: u16 = 0x28;

/// GDT structure
pub struct Gdt {
    entries: [GdtEntry; 5],
}

impl Gdt {
    pub const fn new() -> Self {
        Self {
            entries: [
                GdtEntry::null(),
                GdtEntry::new(0, 0xFFFFF, 0x9A, 0x0A),
                GdtEntry::new(0, 0xFFFFF, 0x92, 0x0C),
                GdtEntry::new(0, 0xFFFFF, 0xFA, 0x0A),
                GdtEntry::new(0, 0xFFFFF, 0xF2, 0x0C),
            ],
        }
    }
}

static GDT: core::sync::OnceLock<Gdt> = core::sync::OnceLock::new();

/// Initialize and load the GDT
pub fn init_gdt() {
    // SAFETY: Loading the GDT via lgdt is required for x86_64 segmentation.
    // The GDT entries are properly initialized with valid segment descriptors.
    // This must be called early in boot before any segment register access.
    unsafe {
        let gdt_ptr = &GDT as *const Gdt;
        let limit = (core::mem::size_of::<Gdt>() - 1) as u16;
        let base = gdt_ptr as u64;

        let mut gdtr: [u8; 10] = [0; 10];
        gdtr[0..2].copy_from_slice(&limit.to_le_bytes());
        gdtr[2..10].copy_from_slice(&base.to_le_bytes());

        asm!(
            "lgdt [{}]",
            "mov ax, {sel:kdata}",
            "mov ds, ax",
            "mov es, ax",
            "mov fs, ax",
            "mov gs, ax",
            "mov ss, ax",
            in(reg) gdtr.as_ptr(),
            sel:kdata = const KERNEL_DATA,
            out("ax") _,
            options(nostack, preserves_flags)
        );

        // SAFETY: LGDT loads the GDT register with the specified base and limit.
        // Segment registers are reloaded with the kernel data selector.
    }
}
