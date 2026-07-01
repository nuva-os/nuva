/*
 * Nuva OS - Kernel - Kernel
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

use crate::pr_info;
use core::sync::atomic::{AtomicU32, Ordering};

/// Device tree node
pub struct DeviceTreeNode {
    /// Node name
    pub name: [u8; 64],
    /// Node path
    pub path: [u8; 256],
    /// Device type
    pub device_type: [u8; 32],
    /// Compatible
    pub compatible: [u8; 128],
    /// Parent node
    pub parent: *mut DeviceTreeNode,
    /// Child node
    pub child: *mut DeviceTreeNode,
    /// Sibling node
    pub sibling: *mut DeviceTreeNode,
    /// Property count
    pub prop_count: u32,
    /// Whether initialized
    pub initialized: bool,
}

impl DeviceTreeNode {
    /// Create new node
    pub fn new(name: &[u8]) -> Self {
        let mut node = DeviceTreeNode {
            name: [0; 64],
            path: [0; 256],
            device_type: [0; 32],
            compatible: [0; 128],
            parent: core::ptr::null_mut(),
            child: core::ptr::null_mut(),
            sibling: core::ptr::null_mut(),
            prop_count: 0,
            initialized: false,
        };

        let len = name.len().min(63);
        node.name[..len].copy_from_slice(&name[..len]);

        node
    }

    /// Get node name
    pub fn get_name(&self) -> &[u8] {
        let mut len = 0;
        for i in 0..64 {
            if self.name[i] == 0 {
                break;
            }
            len = i + 1;
        }
        &self.name[..len]
    }

    /// Add child node
    pub fn add_child(&mut self, child: &mut DeviceTreeNode) {
        child.parent = self;

        if self.child.is_null() {
            self.child = child;
        } else {
            // Add to end of sibling list
            let mut sibling = self.child;
            while !sibling.is_null() {
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    if (*sibling).sibling.is_null() {
                        (*sibling).sibling = child;
                        break;
                    }
                    sibling = (*sibling).sibling;
                }
            }
        }
    }

    /// Traverse child nodes
    pub fn for_each_child<F>(&self, mut f: F)
    where
        F: FnMut(&DeviceTreeNode),
    {
        let mut child = self.child;
        while !child.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                f(&*child);
                child = (*child).sibling;
            }
        }
    }
}

/// Device tree property
pub struct DeviceTreeProperty {
    /// Property name
    pub name: [u8; 32],
    /// Property value
    pub value: [u8; 256],
    /// Value length
    pub length: u32,
}

impl DeviceTreeProperty {
    /// Create new property
    pub fn new(name: &[u8], value: &[u8]) -> Self {
        let mut prop = DeviceTreeProperty {
            name: [0; 32],
            value: [0; 256],
            length: 0,
        };

        let name_len = name.len().min(31);
        prop.name[..name_len].copy_from_slice(&name[..name_len]);

        let value_len = value.len().min(256);
        prop.value[..value_len].copy_from_slice(&value[..value_len]);
        prop.length = value_len as u32;

        prop
    }

    /// Get property name
    pub fn get_name(&self) -> &[u8] {
        let mut len = 0;
        for i in 0..32 {
            if self.name[i] == 0 {
                break;
            }
            len = i + 1;
        }
        &self.name[..len]
    }

    /// Get property value
    pub fn get_value(&self) -> &[u8] {
        &self.value[..self.length as usize]
    }

    /// Get u32 value
    pub fn as_u32(&self) -> Option<u32> {
        if self.length >= 4 {
            let bytes: [u8; 4] = [self.value[0], self.value[1], self.value[2], self.value[3]];
            Some(u32::from_be_bytes(bytes))
        } else {
            None
        }
    }

    /// Get u64 value
    pub fn as_u64(&self) -> Option<u64> {
        if self.length >= 8 {
            let bytes: [u8; 8] = [
                self.value[0],
                self.value[1],
                self.value[2],
                self.value[3],
                self.value[4],
                self.value[5],
                self.value[6],
                self.value[7],
            ];
            Some(u64::from_be_bytes(bytes))
        } else {
            None
        }
    }

    /// Get string value
    pub fn as_string(&self) -> Option<&[u8]> {
        if self.length > 0 && self.value[self.length as usize - 1] == 0 {
            Some(&self.value[..self.length as usize - 1])
        } else {
            None
        }
    }
}

/// Device tree memory information
#[derive(Clone, Copy)]
pub struct DeviceTreeMemory {
    /// Base address
    pub base: u64,
    /// Size
    pub size: u64,
}

/// Device tree reserved memory
#[derive(Clone, Copy)]
pub struct DeviceTreeReservedMemory {
    /// Base address
    pub base: u64,
    /// Size
    pub size: u64,
}

/// Device tree
pub struct DeviceTree {
    /// Root node
    pub root: *mut DeviceTreeNode,
    /// Node count
    pub node_count: AtomicU32,
    /// Memory region count
    pub memory_count: u32,
    /// Memory regions
    pub memories: [Option<DeviceTreeMemory>; 8],
    /// Reserved memory count
    pub reserved_count: u32,
    /// Reserved memory
    pub reserved: [Option<DeviceTreeReservedMemory>; 16],
    /// Boot arguments
    pub bootargs: [u8; 256],
}

impl DeviceTree {
    pub const fn new() -> Self {
        DeviceTree {
            root: core::ptr::null_mut(),
            node_count: AtomicU32::new(0),
            memory_count: 0,
            memories: [None; 8],
            reserved_count: 0,
            reserved: [None; 16],
            bootargs: [0; 256],
        }
    }

    /// Initialize
    pub fn init(&mut self) {
        // Create root node
        let root = DeviceTreeNode::new(b"/");
        // TODO: Allocate memory

        log_info!("Device tree initialized");
    }

    /// Add memory region
    pub fn add_memory(&mut self, base: u64, size: u64) -> bool {
        if self.memory_count >= 8 {
            return false;
        }

        self.memories[self.memory_count as usize] = Some(DeviceTreeMemory { base, size });
        self.memory_count += 1;

        log_info!("Memory region: base=0x{:x}, size=0x{:x}", base, size);

        true
    }

    /// Add reserved memory
    pub fn add_reserved(&mut self, base: u64, size: u64) -> bool {
        if self.reserved_count >= 16 {
            return false;
        }

        self.reserved[self.reserved_count as usize] = Some(DeviceTreeReservedMemory { base, size });
        self.reserved_count += 1;

        true
    }

    /// Set boot arguments
    pub fn set_bootargs(&mut self, args: &[u8]) {
        let len = args.len().min(255);
        self.bootargs[..len].copy_from_slice(&args[..len]);
        self.bootargs[len] = 0;
    }

    /// Get boot arguments
    pub fn get_bootargs(&self) -> &[u8] {
        let mut len = 0;
        for i in 0..256 {
            if self.bootargs[i] == 0 {
                break;
            }
            len = i + 1;
        }
        &self.bootargs[..len]
    }

    /// Get total memory size
    pub fn get_total_memory(&self) -> u64 {
        let mut total = 0u64;
        for i in 0..self.memory_count as usize {
            if let Some(ref mem) = self.memories[i] {
                total += mem.size;
            }
        }
        total
    }

    /// Find node
    pub fn find_node(&self, path: &[u8]) -> Option<&DeviceTreeNode> {
        if self.root.is_null() {
            return None;
        }

        // Simple implementation: only supports root node
        if path == b"/" {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                return Some(&*self.root);
            }
        }

        None
    }

    /// Traverse all nodes
    pub fn for_each_node<F>(&self, mut f: F)
    where
        F: FnMut(&DeviceTreeNode),
    {
        fn traverse<F>(node: *mut DeviceTreeNode, f: &mut F)
        where
            F: FnMut(&DeviceTreeNode),
        {
            if node.is_null() {
                return;
            }

            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                f(&*node);
                traverse((*node).child, f);
                traverse((*node).sibling, f);
            }
        }

        traverse(self.root, &mut f);
    }
}

/// Global device tree
static DEVICE_TREE: crate::sync_oncelock::OnceLock<DeviceTree> = crate::sync_oncelock::OnceLock::new();

pub fn device_tree() -> &'static DeviceTree {
    DEVICE_TREE.get_or_init(DeviceTree::new)
}

pub fn init_device_tree() {
    let dt = device_tree();
    dt.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_tree_node_new() {
        let node = DeviceTreeNode::new(b"test");

        assert_eq!(node.get_name(), b"test");
        assert!(node.parent.is_null());
        assert!(node.child.is_null());
        assert!(node.sibling.is_null());
        assert_eq!(node.prop_count, 0);
        assert!(!node.initialized);
    }

    #[test]
    fn test_device_tree_node_name_truncation() {
        let long_name = [b'a'; 100];
        let node = DeviceTreeNode::new(&long_name);

        // Name should be truncated to 63 bytes
        assert_eq!(node.name[63], 0);
    }

    #[test]
    fn test_device_tree_node_add_child() {
        let mut parent = DeviceTreeNode::new(b"parent");
        let mut child = DeviceTreeNode::new(b"child");

        parent.add_child(&mut child);

        assert!(!parent.child.is_null());
        assert_eq!(child.parent, &mut parent as *mut _);
    }

    #[test]
    fn test_device_tree_node_add_multiple_children() {
        let mut parent = DeviceTreeNode::new(b"parent");
        let mut child1 = DeviceTreeNode::new(b"child1");
        let mut child2 = DeviceTreeNode::new(b"child2");
        let mut child3 = DeviceTreeNode::new(b"child3");

        parent.add_child(&mut child1);
        parent.add_child(&mut child2);
        parent.add_child(&mut child3);

        assert!(!parent.child.is_null());

        // Validate child node list
        let mut count = 0;
        parent.for_each_child(|_| count += 1);
        assert_eq!(count, 3);
    }

    #[test]
    fn test_device_tree_property_new() {
        let prop = DeviceTreeProperty::new(b"status", b"okay");

        assert_eq!(prop.get_name(), b"status");
        assert_eq!(prop.get_value(), b"okay");
        assert_eq!(prop.length, 5);
    }

    #[test]
    fn test_device_tree_property_value_truncation() {
        let long_value = [b'x'; 300];
        let prop = DeviceTreeProperty::new(b"test", &long_value);

        assert_eq!(prop.length, 256);
    }

    #[test]
    fn test_device_tree_property_as_u32() {
        let prop = DeviceTreeProperty::new(b"reg", &[0x00, 0x00, 0x10, 0x00]);

        let value = prop.as_u32();
        assert!(value.is_some());
        assert_eq!(value.unwrap(), 0x00001000);
    }

    #[test]
    fn test_device_tree_property_as_u32_too_short() {
        let prop = DeviceTreeProperty::new(b"reg", &[0x00, 0x00]);

        let value = prop.as_u32();
        assert!(value.is_none());
    }

    #[test]
    fn test_device_tree_property_as_u64() {
        let prop =
            DeviceTreeProperty::new(b"reg", &[0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00]);

        let value = prop.as_u64();
        assert!(value.is_some());
        assert_eq!(value.unwrap(), 0x0000000010000000);
    }

    #[test]
    fn test_device_tree_property_as_u64_too_short() {
        let prop = DeviceTreeProperty::new(b"reg", &[0x00, 0x00, 0x00, 0x00]);

        let value = prop.as_u64();
        assert!(value.is_none());
    }

    #[test]
    fn test_device_tree_property_as_string() {
        let mut prop = DeviceTreeProperty::new(b"compatible", b"arm,cortex-a53");
        prop.value[prop.length as usize] = 0; // null terminator
        prop.length += 1;

        let value = prop.as_string();
        assert!(value.is_some());
        assert_eq!(value.unwrap(), b"arm,cortex-a53");
    }

    #[test]
    fn test_device_tree_property_as_string_no_null() {
        let prop = DeviceTreeProperty::new(b"compatible", b"test");

        let value = prop.as_string();
        assert!(value.is_none());
    }

    #[test]
    fn test_device_tree_memory() {
        let mem = DeviceTreeMemory {
            base: 0x40000000,
            size: 0x10000000, // 256MB
        };

        assert_eq!(mem.base, 0x40000000);
        assert_eq!(mem.size, 0x10000000);
    }

    #[test]
    fn test_device_tree_reserved_memory() {
        let rmem = DeviceTreeReservedMemory {
            base: 0x40000000,
            size: 0x100000,
        };

        assert_eq!(rmem.base, 0x40000000);
        assert_eq!(rmem.size, 0x100000);
    }

    #[test]
    fn test_device_tree_new() {
        let dt = DeviceTree::new();

        assert!(dt.root.is_null());
        assert_eq!(dt.node_count.load(Ordering::Relaxed), 0);
        assert_eq!(dt.memory_count, 0);
        assert_eq!(dt.reserved_count, 0);
    }

    #[test]
    fn test_device_tree_add_memory() {
        let mut dt = DeviceTree::new();

        let result = dt.add_memory(0x40000000, 0x10000000);
        assert!(result);
        assert_eq!(dt.memory_count, 1);

        let result = dt.add_memory(0x80000000, 0x20000000);
        assert!(result);
        assert_eq!(dt.memory_count, 2);
    }

    #[test]
    fn test_device_tree_add_memory_max() {
        let mut dt = DeviceTree::new();

        for _ in 0..8 {
            assert!(dt.add_memory(0, 0x10000000));
        }

        // The 9th should fail
        let result = dt.add_memory(0, 0x10000000);
        assert!(!result);
    }

    #[test]
    fn test_device_tree_add_reserved() {
        let mut dt = DeviceTree::new();

        let result = dt.add_reserved(0x40000000, 0x100000);
        assert!(result);
        assert_eq!(dt.reserved_count, 1);

        let result = dt.add_reserved(0x50000000, 0x200000);
        assert!(result);
        assert_eq!(dt.reserved_count, 2);
    }

    #[test]
    fn test_device_tree_add_reserved_max() {
        let mut dt = DeviceTree::new();

        for _ in 0..16 {
            assert!(dt.add_reserved(0, 0x10000));
        }

        // The 17th should fail
        let result = dt.add_reserved(0, 0x10000);
        assert!(!result);
    }

    #[test]
    fn test_device_tree_bootargs() {
        let mut dt = DeviceTree::new();

        dt.set_bootargs(b"console=tty0 root=/dev/mmcblk0p2");

        let args = dt.get_bootargs();
        assert_eq!(args, b"console=tty0 root=/dev/mmcblk0p2");
    }

    #[test]
    fn test_device_tree_bootargs_truncation() {
        let mut dt = DeviceTree::new();

        let long_args = [b'a'; 300];
        dt.set_bootargs(&long_args);

        // Should be truncated to 255 bytes
        assert_eq!(dt.bootargs[255], 0);
    }

    #[test]
    fn test_device_tree_get_total_memory() {
        let mut dt = DeviceTree::new();

        dt.add_memory(0x40000000, 0x10000000); // 256MB
        dt.add_memory(0x80000000, 0x20000000); // 512MB

        let total = dt.get_total_memory();
        assert_eq!(total, 0x30000000); // 768MB
    }

    #[test]
    fn test_device_tree_get_total_memory_empty() {
        let dt = DeviceTree::new();

        let total = dt.get_total_memory();
        assert_eq!(total, 0);
    }

    #[test]
    fn test_device_tree_find_node_no_root() {
        let dt = DeviceTree::new();

        let result = dt.find_node(b"/");
        assert!(result.is_none());
    }

    #[test]
    fn test_device_tree_node_initialized() {
        let mut node = DeviceTreeNode::new(b"test");

        assert!(!node.initialized);

        node.initialized = true;
        assert!(node.initialized);
    }

    #[test]
    fn test_device_tree_node_device_type() {
        let mut node = DeviceTreeNode::new(b"cpu");

        let device_type = b"cpu";
        node.device_type[..device_type.len()].copy_from_slice(device_type);

        assert_eq!(&node.device_type[..3], b"cpu");
    }

    #[test]
    fn test_device_tree_node_compatible() {
        let mut node = DeviceTreeNode::new(b"cpu");

        let compatible = b"arm,cortex-a53";
        node.compatible[..compatible.len()].copy_from_slice(compatible);

        assert_eq!(&node.compatible[..14], b"arm,cortex-a53");
    }
}
