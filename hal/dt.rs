/*
 * Nuva OS - HAL - Flattened Device Tree (FDT/DTB) Parser
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

/// FDT magic number.
const FDT_MAGIC: u32 = 0xD00DFEED;

/// FDT version supported.
const FDT_VERSION: u32 = 17;

/// Maximum depth of device tree traversal.
const MAX_DT_DEPTH: usize = 16;

/// Maximum number of device tree nodes.
const MAX_DT_NODES: usize = 256;

/// Maximum property name length.
const MAX_PROP_NAME_LEN: usize = 64;

/// Maximum property data length.
const MAX_PROP_DATA_LEN: usize = 256;

/// FDT header structure (as found at the start of the DTB).
#[repr(C)]
pub struct FdtHeader {
    /// Magic number (0xD00DFEED).
    pub magic: u32,
    /// Total size of the DTB.
    pub totalsize: u32,
    /// Offset of the structure block.
    pub off_dt_struct: u32,
    /// Offset of the strings block.
    pub off_dt_strings: u32,
    /// Offset of the memory reservation block.
    pub off_mem_rsvmap: u32,
    /// FDT version.
    pub version: u32,
    /// Last compatible version.
    pub last_comp_version: u32,
    /// Boot CPU ID (version 2 only).
    pub boot_cpuid_phys: u32,
    /// Offset of the strings block (version 3+).
    pub size_dt_strings: u32,
    /// Size of the structure block (version 17+).
    pub size_dt_struct: u32,
}

/// FDT token types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FdtToken {
    /// Start of a node.
    FdtBeginNode = 0x00000001,
    /// End of a node.
    FdtEndNode = 0x00000002,
    /// Property.
    FdtProp = 0x00000003,
    /// NOP.
    FdtNop = 0x00000004,
    /// End of the structure block.
    FdtEnd = 0x00000009,
}

/// Device tree property.
pub struct DtProperty {
    /// Property name.
    pub name: [u8; MAX_PROP_NAME_LEN],
    /// Length of the property name string.
    pub name_len: usize,
    /// Property data.
    pub data: [u8; MAX_PROP_DATA_LEN],
    /// Length of the property data.
    pub data_len: usize,
}

impl DtProperty {
    /// Create an empty property.
    pub const fn new() -> Self {
        DtProperty {
            name: [0u8; MAX_PROP_NAME_LEN],
            name_len: 0,
            data: [0u8; MAX_PROP_DATA_LEN],
            data_len: 0,
        }
    }

    /// Get the property name as a string slice.
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }

    /// Get the property data as a u32 array.
    pub fn as_u32_array(&self) -> &[u32] {
        let count = self.data_len / 4;
        // SAFETY: self.data is a [u8; MAX_PROP_DATA_LEN] array (256 bytes), which is
        // 4-byte aligned; count = data_len / 4 ensures the slice does not exceed the
        // valid data region; the u32 pointer cast is valid because the array alignment
        // and size are compatible.
        unsafe {
            core::slice::from_raw_parts(
                self.data.as_ptr() as *const u32,
                count,
            )
        }
    }

    /// Get the property data as a single u32.
    pub fn as_u32(&self) -> u32 {
        if self.data_len >= 4 {
            let arr = self.as_u32_array();
            u32::from_be(arr[0])
        } else {
            0
        }
    }

    /// Get the property data as a single u64.
    pub fn as_u64(&self) -> u64 {
        if self.data_len >= 8 {
            let arr = self.as_u32_array();
            ((u64::from(u32::from_be(arr[0]))) << 32) | u64::from(u32::from_be(arr[1]))
        } else {
            0
        }
    }

    /// Get the property data as a string.
    pub fn as_string(&self) -> &str {
        let len = if self.data_len > 0 && self.data[self.data_len - 1] == 0 {
            self.data_len - 1
        } else {
            self.data_len
        };
        core::str::from_utf8(&self.data[..len]).unwrap_or("")
    }

    /// Check if the "compatible" property matches the given string.
    pub fn compatible_matches(&self, compat_str: &str) -> bool {
        if self.name_str() != "compatible" { return false; }
        // The compatible property is a null-separated list of strings
        let mut start = 0;
        for i in 0..self.data_len {
            if self.data[i] == 0 {
                if start < i {
                    if let Ok(s) = core::str::from_utf8(&self.data[start..i]) {
                        if s == compat_str { return true; }
                    }
                }
                start = i + 1;
            }
        }
        false
    }
}

/// Device tree node.
pub struct DtNode {
    /// Node name.
    pub name: [u8; MAX_PROP_NAME_LEN],
    /// Length of the node name.
    pub name_len: usize,
    /// Depth in the tree (0 = root).
    pub depth: usize,
    /// Properties of this node.
    pub properties: [DtProperty; 8],
    /// Number of properties.
    pub prop_count: usize,
    /// Unit address part of the name (after @).
    pub unit_addr: u64,
}

impl DtNode {
    /// Create an empty node.
    pub const fn new() -> Self {
        DtNode {
            name: [0u8; MAX_PROP_NAME_LEN],
            name_len: 0,
            depth: 0,
            properties: [
                DtProperty::new(), DtProperty::new(), DtProperty::new(), DtProperty::new(),
                DtProperty::new(), DtProperty::new(), DtProperty::new(), DtProperty::new(),
            ],
            prop_count: 0,
            unit_addr: 0,
        }
    }

    /// Get the node name as a string slice.
    pub fn name_str(&self) -> &str {
        core::str::from_utf8(&self.name[..self.name_len]).unwrap_or("")
    }

    /// Find a property by name.
    pub fn find_property(&self, prop_name: &str) -> Option<&DtProperty> {
        for i in 0..self.prop_count {
            if self.properties[i].name_str() == prop_name {
                return Some(&self.properties[i]);
            }
        }
        None
    }

    /// Check if this node's "compatible" property matches.
    pub fn is_compatible(&self, compat_str: &str) -> bool {
        if let Some(prop) = self.find_property("compatible") {
            prop.compatible_matches(compat_str)
        } else {
            false
        }
    }

    /// Get the "reg" property as address and size.
    pub fn get_reg(&self) -> Option<(u64, u64)> {
        if let Some(prop) = self.find_property("reg") {
            if prop.data_len >= 16 {
                let addr = prop.as_u64();
                // Size is the second u64
                let size_arr = prop.as_u32_array();
                if size_arr.len() >= 4 {
                    let size = ((u64::from(u32::from_be(size_arr[2]))) << 32) | u64::from(u32::from_be(size_arr[3]));
                    Some((addr, size))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get the "interrupts" property as a u32 array.
    pub fn get_interrupts(&self) -> Option<&[u32]> {
        if let Some(prop) = self.find_property("interrupts") {
            Some(prop.as_u32_array())
        } else {
            None
        }
    }
}

/// Device tree parser state.
pub struct DeviceTree {
    /// Parsed nodes.
    pub nodes: [DtNode; MAX_DT_NODES],
    /// Number of parsed nodes.
    pub node_count: usize,
    /// Total memory from /memory node.
    pub memory_size: u64,
    /// Boot CPU ID.
    pub boot_cpuid: u32,
    /// Model name from root node.
    pub model: [u8; 64],
    /// Length of model string.
    pub model_len: usize,
}

impl DeviceTree {
    /// Create an empty device tree.
    pub const fn new() -> Self {
        DeviceTree {
            nodes: [const { DtNode::new() }; MAX_DT_NODES],
            node_count: 0,
            memory_size: 0,
            boot_cpuid: 0,
            model: [0u8; 64],
            model_len: 0,
        }
    }

    /// Parse a DTB (Flattened Device Tree Blob) from memory.
    /// # Arguments
    /// * `dtb_ptr` - Pointer to the start of the DTB in memory
    /// * `dtb_size` - Size of the DTB in bytes (0 = auto-detect from header)
    /// # Returns
    /// * `true` if parsing succeeded, `false` on error
    pub fn parse(&mut self, dtb_ptr: *const u8, _dtb_size: u32) -> bool {
        // SAFETY: dtb_ptr is provided by the bootloader (UEFI config table or U-Boot)
        // and points to a valid DTB in mapped memory; the FdtHeader is repr(C) and
        // reading it is the first step of DTB validation.
        let header = unsafe { &*(dtb_ptr as *const FdtHeader) };

        // Validate FDT magic
        if u32::from_be(header.magic) != FDT_MAGIC {
            log::error!("FDT: Invalid magic: 0x{:08x} (expected 0x{:08x})",
                u32::from_be(header.magic), FDT_MAGIC);
            return false;
        }

        // Validate version
        if u32::from_be(header.version) < FDT_VERSION {
            log::error!("FDT: Unsupported version: {} (need >= {})",
                u32::from_be(header.version), FDT_VERSION);
            return false;
        }

        let struct_offset = u32::from_be(header.off_dt_struct) as usize;
        let strings_offset = u32::from_be(header.off_dt_strings) as usize;

        log::info!("FDT: DTB at {:p}, size={}, struct={}, strings={}",
            dtb_ptr, u32::from_be(header.totalsize), struct_offset, strings_offset);

        // Parse the structure block
        // SAFETY: struct_offset is derived from the validated FDT header's off_dt_struct
        // field; dtb_ptr is a valid bootloader-provided DTB pointer, so the resulting
        // pointer is within the DTB's mapped memory region.
        let struct_ptr = unsafe { dtb_ptr.add(struct_offset) };
        // SAFETY: strings_offset is derived from the validated FDT header's off_dt_strings
        // field; dtb_ptr is a valid bootloader-provided DTB pointer, so the resulting
        // pointer is within the DTB's mapped memory region.
        let strings_ptr = unsafe { dtb_ptr.add(strings_offset) };
        let struct_size = u32::from_be(header.size_dt_struct) as usize;

        self.parse_struct_block(struct_ptr, strings_ptr, struct_size)
    }

    /// Parse the structure block of the DTB.
    fn parse_struct_block(&mut self, struct_ptr: *const u8, strings_ptr: *const u8, struct_size: usize) -> bool {
        let mut offset: usize = 0;
        let mut depth: usize = 0;
        let mut current_node_idx: usize = 0;

        while offset < struct_size && self.node_count < MAX_DT_NODES {
            // Read token (big-endian u32)
            // SAFETY: struct_ptr points to the DTB structure block; offset is bounded
            // by struct_size; the token is a 4-byte aligned u32 per the FDT spec.
            let token = u32::from_be(unsafe {
                let ptr = struct_ptr.add(offset) as *const u32;
                *ptr
            });
            offset += 4;

            match token {
                0x00000001 => {
                    // FDT_BEGIN_NODE
                    let name_start = offset;
                    // Find null terminator
                    let mut name_end = name_start;
                    while name_end < struct_size {
                        // SAFETY: name_end is bounded by struct_size; struct_ptr
                        // points to the validated DTB structure block.
                        let byte = unsafe { *struct_ptr.add(name_end) };
                        if byte == 0 { break; }
                        name_end += 1;
                    }

                    // Add node
                    if self.node_count < MAX_DT_NODES {
                        let node = &mut self.nodes[self.node_count];
                        let name_len = name_end - name_start;
                        let copy_len = if name_len < MAX_PROP_NAME_LEN { name_len } else { MAX_PROP_NAME_LEN - 1 };

                        for i in 0..copy_len {
                            // SAFETY: name_start + i < name_end < struct_size;
                            // struct_ptr points to the validated DTB structure block.
                            node.name[i] = unsafe { *struct_ptr.add(name_start + i) };
                        }
                        node.name_len = copy_len;
                        node.depth = depth;
                        node.prop_count = 0;
                        node.unit_addr = 0;

                        // Parse unit address from name (e.g., "serial@ffe00000")
                        for i in 0..copy_len {
                            if node.name[i] == b'@' {
                                // Parse hex address after @
                                let mut addr: u64 = 0;
                                for j in (i + 1)..copy_len {
                                    let c = node.name[j];
                                    if c >= b'0' && c <= b'9' {
                                        addr = addr * 16 + (c - b'0') as u64;
                                    } else if c >= b'a' && c <= b'f' {
                                        addr = addr * 16 + (c - b'a' + 10) as u64;
                                    } else if c >= b'A' && c <= b'F' {
                                        addr = addr * 16 + (c - b'A' + 10) as u64;
                                    } else {
                                        break;
                                    }
                                }
                                node.unit_addr = addr;
                                break;
                            }
                        }

                        current_node_idx = self.node_count;
                        self.node_count += 1;
                    }

                    depth += 1;
                    // Align to 4 bytes
                    offset = name_end + 1;
                    offset = (offset + 3) & !3;
                }
                0x00000002 => {
                    // FDT_END_NODE
                    if depth > 0 { depth -= 1; }
                }
                0x00000003 => {
                    // FDT_PROP
                    if offset + 8 > struct_size { break; }

                    // SAFETY: offset + 4 <= struct_size (checked above: offset + 8 <= struct_size);
                    // struct_ptr points to the validated DTB structure block; the FDT_PROP
                    // token is followed by two big-endian u32 fields (len and nameoff).
                    let prop_len = u32::from_be(unsafe {
                        let ptr = struct_ptr.add(offset) as *const u32;
                        *ptr
                    }) as usize;
                    // SAFETY: offset + 4 is within struct_size (checked above);
                    // the nameoff field is a u32 offset into the strings block.
                    let name_offset = u32::from_be(unsafe {
                        let ptr = struct_ptr.add(offset + 4) as *const u32;
                        *ptr
                    }) as usize;
                    offset += 8;

                    // Get property name from strings block
                    let data_start = offset;
                    let data_len = if prop_len < MAX_PROP_DATA_LEN { prop_len } else { MAX_PROP_DATA_LEN };

                    // Add property to current node
                    if current_node_idx < self.node_count {
                        let node = &mut self.nodes[current_node_idx];
                        if node.prop_count < 8 {
                            let prop = &mut node.properties[node.prop_count];

                            // Copy name from strings block
                            let mut name_len = 0;
                            while name_len < MAX_PROP_NAME_LEN - 1 {
                                // SAFETY: strings_ptr points to the validated DTB strings
                                // block; name_offset is from the FDT_PROP field and
                                // name_len is bounded by MAX_PROP_NAME_LEN - 1.
                                let byte = unsafe { *strings_ptr.add(name_offset + name_len) };
                                if byte == 0 { break; }
                                prop.name[name_len] = byte;
                                name_len += 1;
                            }
                            prop.name_len = name_len;

                            // Copy data
                            for i in 0..data_len {
                                if data_start + i < struct_size {
                                    // SAFETY: data_start + i < struct_size (checked);
                                    // struct_ptr points to the validated DTB structure block.
                                    prop.data[i] = unsafe { *struct_ptr.add(data_start + i) };
                                }
                            }
                            prop.data_len = data_len;

                            node.prop_count += 1;
                        }
                    }

                    // Advance past data and align
                    offset = data_start + prop_len;
                    offset = (offset + 3) & !3;
                }
                0x00000004 => {
                    // FDT_NOP - skip
                }
                0x00000009 => {
                    // FDT_END
                    break;
                }
                _ => {
                    log::warn!("FDT: Unknown token 0x{:08x} at offset {}", token, offset - 4);
                    break;
                }
            }
        }

        // Extract key information from parsed nodes
        self.extract_info();

        log::info!("FDT: Parsed {} nodes", self.node_count);
        true
    }

    /// Extract key information from parsed device tree nodes.
    fn extract_info(&mut self) {
        for i in 0..self.node_count {
            let node = &self.nodes[i];

            // Extract model from root node
            if node.depth == 0 {
                if let Some(prop) = node.find_property("model") {
                    let len = if prop.data_len < 64 { prop.data_len } else { 63 };
                    for j in 0..len {
                        self.model[j] = prop.data[j];
                    }
                    self.model_len = len;
                }
            }

            // Extract memory size from /memory node
            if node.name_str().starts_with("memory") {
                if let Some((_, size)) = node.get_reg() {
                    self.memory_size = size;
                }
            }
        }
    }

    /// Find a node by compatible string.
    pub fn find_compatible(&self, compat: &str) -> Option<&DtNode> {
        for i in 0..self.node_count {
            if self.nodes[i].is_compatible(compat) {
                return Some(&self.nodes[i]);
            }
        }
        None
    }

    /// Find a node by name prefix.
    pub fn find_node_by_name(&self, name_prefix: &str) -> Option<&DtNode> {
        for i in 0..self.node_count {
            if self.nodes[i].name_str().starts_with(name_prefix) {
                return Some(&self.nodes[i]);
            }
        }
        None
    }

    /// Get the model name as a string.
    pub fn model_str(&self) -> &str {
        core::str::from_utf8(&self.model[..self.model_len]).unwrap_or("unknown")
    }
}

/// Global DeviceTree instance.
static mut DEVICE_TREE: DeviceTree = DeviceTree::new();

/// Get a reference to the global DeviceTree.
pub fn get_device_tree() -> &'static DeviceTree {
    // SAFETY: DEVICE_TREE is a mutable static accessed only during single-threaded
    // HAL initialization; after init completes, only immutable references are returned.
    unsafe { &DEVICE_TREE }
}

/// Parse the Device Tree from a DTB pointer.
/// # Arguments
/// * `dtb_ptr` - Pointer to the DTB in memory (from UEFI config table or U-Boot)
/// * `dtb_size` - Size of the DTB (0 = auto-detect from header)
/// # Returns
/// * `true` if parsing succeeded
pub fn parse_dtb(dtb_ptr: *const u8, dtb_size: u32) -> bool {
    // SAFETY: parse_dtb() is called only during single-threaded HAL initialization
    // before any other CPU cores are brought online; no concurrent access possible.
    let dt = unsafe { &mut DEVICE_TREE };
    dt.parse(dtb_ptr, dtb_size)
}

/// Initialize the Device Tree subsystem.
/// On ARM64, the DTB pointer is typically obtained from:
/// - UEFI: Configuration table entry for FDT
/// - U-Boot: Passed in register x0 or x21
pub fn init_dt() {
    // In a real implementation, the DTB pointer would be obtained from
    // the bootloader. For now, we just log that the DT subsystem is ready.
    log::info!("FDT: Device Tree parser initialized");
}
