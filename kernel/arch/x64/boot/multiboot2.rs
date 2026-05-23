/*
 * Nuva OS - Kernel - Arch - x64 - Boot - Multiboot2
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

//! Multiboot2 info parser for x86_64

use crate::kernel::platform::{PlatformInfo, BootInfoType};

/// Multiboot2 info header
#[repr(C)]
pub struct Multiboot2Info {
    /// Total size of multiboot2 info
    pub total_size: u32,
    /// Reserved, must be 0
    pub reserved: u32,
}

/// Multiboot2 tag header
#[repr(C)]
pub struct MbiTagHeader {
    /// Tag type
    pub tag_type: u32,
    /// Tag size in bytes
    pub tag_size: u32,
}

/// Multiboot2 tag types
pub const MBI_TAG_END: u32 = 0;
pub const MBI_TAG_CMDLINE: u32 = 1;
pub const MBI_TAG_BOOT_LOADER_NAME: u32 = 2;
pub const MBI_TAG_BASIC_MEM_INFO: u32 = 4;
pub const MBI_TAG_BOOTDEV: u32 = 5;
pub const MBI_TAG_MEMORY_MAP: u32 = 6;

/// Basic memory info tag (type 4)
#[repr(C)]
pub struct MbiBasicMemInfoTag {
    pub header: MbiTagHeader,
    /// Amount of lower memory in KB
    pub mem_lower: u32,
    /// Amount of upper memory in KB
    pub mem_upper: u32,
}

/// Memory map entry
#[repr(C)]
pub struct MbiMemMapEntry {
    /// Base address
    pub base_addr: u64,
    /// Length in bytes
    pub length: u64,
    /// Entry type: 1=available, 2=reserved, 3=ACPI reclaimable, 4=NVS, 5=unusable
    pub entry_type: u32,
    /// Reserved
    pub reserved: u32,
}

/// Memory map tag (type 6)
#[repr(C)]
pub struct MbiMemMapTag {
    pub header: MbiTagHeader,
    /// Size of each memory map entry
    pub entry_size: u32,
    /// Entry version
    pub entry_version: u32,
}

/// Command line tag (type 1)
#[repr(C)]
pub struct MbiCmdlineTag {
    pub header: MbiTagHeader,
    /// Command line string (null-terminated)
    pub cmdline: [u8; 0],
}

/// Parse Multiboot2 info and return PlatformInfo
pub fn parse_multiboot2_info(ptr: *const u8) -> PlatformInfo {
    if ptr.is_null() {
        return PlatformInfo::default();
    }

    let mut info = PlatformInfo {
        boot_info: ptr,
        boot_info_type: BootInfoType::Multiboot2,
        ..PlatformInfo::default()
    };

    // SAFETY: ptr is valid as passed by bootloader per Multiboot2 spec
    unsafe {
        let mbi = ptr as *const Multiboot2Info;
        let total_size = (*mbi).total_size as usize;
        let end = ptr.add(total_size);

        let mut tag_ptr = ptr.add(core::mem::size_of::<Multiboot2Info>());

        while tag_ptr < end {
            let header = tag_ptr as *const MbiTagHeader;
            let tag_type = (*header).tag_type;
            let tag_size = (*header).tag_size;

            if tag_type == MBI_TAG_END || tag_size == 0 {
                break;
            }

            match tag_type {
                MBI_TAG_BASIC_MEM_INFO => {
                    let basic = tag_ptr as *const MbiBasicMemInfoTag;
                    let mem_lower_kb = (*basic).mem_lower as u64 * 1024;
                    let mem_upper_kb = (*basic).mem_upper as u64 * 1024;
                    info.memory_size = mem_lower_kb + mem_upper_kb;
                    if info.memory_size == 0 {
                        info.memory_size = 128 * 1024 * 1024;
                    }
                }
                MBI_TAG_MEMORY_MAP => {
                    let mmap = tag_ptr as *const MbiMemMapTag;
                    let entry_size = (*mmap).entry_size as usize;
                    let entries_start = tag_ptr.add(core::mem::size_of::<MbiMemMapTag>());
                    let entries_end = tag_ptr.add(tag_size);
                    let mut total_available: u64 = 0;
                    let mut entry_ptr = entries_start;
                    while entry_ptr.add(entry_size) <= entries_end {
                        let entry = entry_ptr as *const MbiMemMapEntry;
                        if (*entry).entry_type == 1 {
                            total_available += (*entry).length;
                        }
                        entry_ptr = entry_ptr.add(entry_size);
                    }
                    if total_available > 0 {
                        info.memory_size = total_available;
                    }
                }
                _ => {}
            }

            let next = (tag_ptr as usize + tag_size + 7) & !7;
            tag_ptr = next as *const u8;
        }
    }

    info
}
