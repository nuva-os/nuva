/*
* Nuva OS - Kernel - Arch
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

use crate::kernel::platform::{BootInfoType, PlatformInfo};
use alloc::string::ToString;
use core::ptr;
use core::slice;

/// FDT magic number
const FDT_MAGIC: u32 = 0xD00DFEED;

/// FDT header
#[repr(C, packed)]
struct FdtHeader {
    magic: u32,
    totalsize: u32,
    off_dt_struct: u32,
    off_dt_strings: u32,
    off_mem_rsvmap: u32,
    version: u32,
    last_comp_version: u32,
    boot_cpuid_phys: u32,
    size_dt_strings: u32,
    size_dt_struct: u32,
}

/// FDT node markers
const FDT_BEGIN_NODE: u32 = 0x1;
const FDT_END_NODE: u32 = 0x2;
const FDT_PROP: u32 = 0x3;
const FDT_NOP: u32 = 0x4;
const FDT_END: u32 = 0x9;

/// Device tree parser
pub struct FdtParser {
    header: *const FdtHeader,
    struct_block: *const u8,
    strings_block: *const u8,
    mem_rsvmap: *const u8,
}

/// Memory reserved region
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub address: u64,
    pub size: u64,
}

/// Device tree property
#[derive(Debug, Clone, Copy)]
pub struct FdtProperty {
    pub name: &'static str,
    pub value: &'static [u8],
}

/// Device tree node
pub struct FdtNode {
    pub name: &'static str,
    pub properties: [FdtProperty; 16],
    pub property_count: usize,
}

impl FdtParser {
    /// Create new FDT parser
    pub fn new(fdt_ptr: *const u8) -> Option<Self> {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let header = fdt_ptr as *const FdtHeader;

            // Validate magic number
            if (*header).magic != FDT_MAGIC {
                return None;
            }

            let struct_block = fdt_ptr.add((*header).off_dt_struct as usize);
            let strings_block = fdt_ptr.add((*header).off_dt_strings as usize);
            let mem_rsvmap = fdt_ptr.add((*header).off_mem_rsvmap as usize);

            Some(FdtParser {
                header,
                struct_block,
                strings_block,
                mem_rsvmap,
            })
        }
    }

    /// Get FDT total size
    pub fn total_size(&self) -> usize {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { (*self.header).totalsize as usize }
    }

    /// Get FDT version
    pub fn version(&self) -> u32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { (*self.header).version }
    }

    /// Get memory reserved regions
    pub fn get_memory_reservations(&self) -> &'static [MemoryRegion] {
        static mut REGIONS: [MemoryRegion; 16] = [MemoryRegion {
            address: 0,
            size: 0,
        }; 16];
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let mut count = 0;

            let mut ptr = self.mem_rsvmap as *const MemoryRegion;

            loop {
                let region = *ptr;

                // Termination condition
                if region.address == 0 && region.size == 0 {
                    break;
                }

                if count < REGIONS.len() {
                    REGIONS[count] = region;
                    count += 1;
                }

                ptr = ptr.add(1);
            }

            &REGIONS[..count]
        }
    }

    /// Find node
    pub fn find_node(&self, path: &str) -> Option<FdtNode> {
        // Simplified implementation: only supports root node and first-level child nodes
        let mut node = FdtNode {
            name: "",
            properties: [FdtProperty {
                name: "",
                value: &[],
            }; 16],
            property_count: 0,
        };

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let mut ptr = self.struct_block as *const u32;
            let mut in_node = false;
            let mut current_depth = 0;

            loop {
                let token = *ptr;
                ptr = ptr.add(1);

                match token {
                    FDT_BEGIN_NODE => {
                        // Read node name
                        let name_ptr = ptr as *const i8;
                        let name_len = Self::strlen(name_ptr);
                        let name = core::str::from_utf8_unchecked(slice::from_raw_parts(
                            name_ptr as *const u8,
                            name_len,
                        ));

                        current_depth += 1;

                        // Check if matches path
                        if path == "/" || name == path.trim_start_matches('/') {
                            in_node = true;
                            node.name = name;
                        }

                        // Align to 4 bytes
                        ptr = (ptr as usize + name_len + 4) as *const u32;
                        ptr = (ptr as usize & !3) as *const u32;
                    }

                    FDT_END_NODE => {
                        current_depth -= 1;

                        if in_node && current_depth == 0 {
                            return Some(node);
                        }
                    }

                    FDT_PROP => {
                        if !in_node {
                            continue;
                        }

                        // Read property length and name offset
                        let len = *ptr as usize;
                        ptr = ptr.add(1);
                        let nameoff = *ptr as usize;
                        ptr = ptr.add(1);

                        // Read property value
                        let value_ptr = ptr as *const u8;
                        let value = slice::from_raw_parts(value_ptr, len);

                        // Read property name
                        let name_ptr = self.strings_block.add(nameoff);
                        let name_len = Self::strlen(name_ptr as *const i8);
                        let name = core::str::from_utf8_unchecked(slice::from_raw_parts(
                            name_ptr, name_len,
                        ));

                        // Add property
                        if node.property_count < node.properties.len() {
                            node.properties[node.property_count] = FdtProperty { name, value };
                            node.property_count += 1;
                        }

                        // Move to next token
                        ptr = (ptr as usize + len + 3) as *const u32;
                        ptr = (ptr as usize & !3) as *const u32;
                    }

                    FDT_END => {
                        break;
                    }

                    _ => {}
                }
            }
        }

        None
    }

    /// Get property value
    pub fn get_property(&self, node: &FdtNode, name: &str) -> Option<&'static [u8]> {
        for i in 0..node.property_count {
            if node.properties[i].name == name {
                return Some(node.properties[i].value);
            }
        }
        None
    }

    /// Get string property
    pub fn get_string_property(&self, node: &FdtNode, name: &str) -> Option<&'static str> {
        let value = self.get_property(node, name)?;

        // Remove trailing null character
        let len = value.iter().position(|&c| c == 0).unwrap_or(value.len());
        // SAFETY: FDT property values are valid UTF-8
        Some(unsafe { core::str::from_utf8_unchecked(&value[..len]) })
    }

    /// Get u32 property
    pub fn get_u32_property(&self, node: &FdtNode, name: &str) -> Option<u32> {
        let value = self.get_property(node, name)?;
        if value.len() >= 4 {
            Some(u32::from_be_bytes([value[0], value[1], value[2], value[3]]))
        } else {
            None
        }
    }

    /// Get u64 property
    pub fn get_u64_property(&self, node: &FdtNode, name: &str) -> Option<u64> {
        let value = self.get_property(node, name)?;
        if value.len() >= 8 {
            Some(u64::from_be_bytes([
                value[0], value[1], value[2], value[3], value[4], value[5], value[6], value[7],
            ]))
        } else {
            None
        }
    }

    /// Get memory info
    pub fn get_memory_info(&self) -> Option<(u64, u64)> {
        let node = self.find_node("memory")?;
        let reg = self.get_property(&node, "reg")?;

        if reg.len() >= 16 {
            let address = u64::from_be_bytes([
                reg[0], reg[1], reg[2], reg[3], reg[4], reg[5], reg[6], reg[7],
            ]);
            let size = u64::from_be_bytes([
                reg[8], reg[9], reg[10], reg[11], reg[12], reg[13], reg[14], reg[15],
            ]);
            Some((address, size))
        } else {
            None
        }
    }

    /// Get CPU count
    pub fn get_cpu_count(&self) -> usize {
        let mut count = 0;

        // Iterate through cpu child nodes under cpus node
        // Simplified implementation: assume at most 8 CPUs
        for i in 0..8 {
            let path = format_args!("cpu@{}", i);
            if self.find_node(&path.to_string()).is_some() {
                count += 1;
            }
        }

        count
    }

    /// Calculate string length
    // SAFETY: The caller must ensure `s` is a valid pointer to a
    // null-terminated C string within accessible memory.
    unsafe fn strlen(s: *const i8) -> usize {
        let mut len = 0;
        while *s.add(len) != 0 {
            len += 1;
        }
        len
    }
}

/// Initialize device tree
pub fn init_fdt(fdt_ptr: *const u8) -> Option<FdtParser> {
    FdtParser::new(fdt_ptr)
}

/// Extract platform info from FDT
pub fn extract_platform_info(fdt_ptr: *const u8) -> Result<PlatformInfo, &'static str> {
    let fdt = FdtParser::new(fdt_ptr).ok_or("Invalid FDT: magic number mismatch")?;

    let (memory_base, memory_size) = fdt
        .get_memory_info()
        .ok_or("Failed to parse memory info from FDT")?;

    let cpu_count = fdt.get_cpu_count();
    if cpu_count == 0 {
        return Err("No CPUs found in FDT");
    }

    Ok(PlatformInfo {
        memory_base,
        memory_size,
        cpu_count: cpu_count as u32,
        boot_info: fdt_ptr,
        boot_info_type: BootInfoType::Fdt,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fdt_parser() {
        // Test code
    }
}
