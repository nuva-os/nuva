use crate::{pr_info};
/*
 * Nuva OS - Kernel - Resource Management
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Kernel resource management for memory, I/O, IRQ, etc.
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::posix::errno::Errno;
/// Resource Type
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    /// Invalid
    Invalid = 0,
    /// Physical memory
    Mem = 1,
    /// I/O port
    Io = 2,
    /// IRQ number
    Irq = 3,
    /// DMA channel
    Dma = 4,
    /// Bus number
    Bus = 5,
    /// Prefetchable memory
    MemPref = 6,
    /// I/O memory (MMIO)
    MemIo = 7,
    /// Reserved
    Reserved = 8,
}

/// Resource Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ResourceFlags: u64 {
        /// Readable
        const READABLE = 1 << 0;
        /// Writable
        const WRITABLE = 1 << 1;
        /// Executable
        const EXECUTABLE = 1 << 2;
        /// Cacheable
        const CACHEABLE = 1 << 3;
        /// Prefetchable
        const PREFETCHABLE = 1 << 4;
        /// Shared
        const SHARED = 1 << 5;
        /// Busy
        const BUSY = 1 << 6;
        /// Disabled
        const DISABLED = 1 << 7;
        /// 64-bit
        const SIZE_64 = 1 << 8;
        /// Window
        const WINDOW = 1 << 9;
        /// Has lock
        const HAS_LOCK = 1 << 10;
        /// Driver exclusive
        const DRIVER_EXCLUSIVE = 1 << 11;
        /// Atomic
        const ATOMIC = 1 << 12;
    }
}

/// Resource Structure
#[repr(C)]
pub struct Resource {
    /// Resource type
    pub resource_type: ResourceType,
    /// Start address
    pub start: u64,
    /// End address (inclusive)
    pub end: u64,
    /// Flags
    pub flags: ResourceFlags,
    /// Resource name
    pub name: [u8; 32],
    /// Parent resource
    pub parent: *mut Resource,
    /// Sibling resource
    pub sibling: *mut Resource,
    /// Child resource
    pub child: *mut Resource,
    /// Owner device
    pub owner: u64,
    /// Reference count
    pub ref_count: AtomicU32,
}

impl Resource {
    pub fn new(resource_type: ResourceType, start: u64, end: u64) -> Self {
        Resource {
            resource_type,
            start,
            end,
            flags: ResourceFlags::READABLE | ResourceFlags::WRITABLE,
            name: [0; 32],
            parent: core::ptr::null_mut(),
            sibling: core::ptr::null_mut(),
            child: core::ptr::null_mut(),
            owner: 0,
            ref_count: AtomicU32::new(1),
        }
    }
    
    pub fn new_mem(start: u64, size: u64) -> Self {
        Resource::new(ResourceType::Mem, start, start + size - 1)
    }
    
    pub fn new_io(start: u64, size: u64) -> Self {
        Resource::new(ResourceType::Io, start, start + size - 1)
    }
    
    pub fn new_irq(irq: u32) -> Self {
        Resource::new(ResourceType::Irq, irq as u64, irq as u64)
    }
    
    /// Get size
    pub fn size(&self) -> u64 {
        self.end.saturating_sub(self.start) + 1
    }
    
    /// Check if contains address
    pub fn contains(&self, addr: u64) -> bool {
        addr >= self.start && addr <= self.end
    }
    
    /// Check if overlaps with another resource
    pub fn overlaps(&self, other: &Resource) -> bool {
        self.start <= other.end && self.end >= other.start
    }
    
    /// Check if contains another resource
    pub fn contains_resource(&self, other: &Resource) -> bool {
        self.start <= other.start && self.end >= other.end
    }
    
    /// Set name
    pub fn set_name(&mut self, name: &[u8]) {
        let len = name.len().min(31);
        self.name[..len].copy_from_slice(&name[..len]);
        self.name[len] = 0;
    }
    
    /// Get reference
    pub fn get(&self) {
        self.ref_count.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Put reference
    pub fn put(&self) -> u32 {
        self.ref_count.fetch_sub(1, Ordering::AcqRel)
    }
}

/// Resource List
pub struct ResourceList {
    /// Head resource
    pub head: *mut Resource,
    /// Resource count
    pub count: AtomicU32,
}

impl ResourceList {
    pub const fn new() -> Self {
        ResourceList {
            head: core::ptr::null_mut(),
            count: AtomicU32::new(0),
        }
    }
    
    /// Add resource
    pub fn add(&mut self, res: *mut Resource) {
        if res.is_null() {
            return;
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*res).sibling = self.head;
            self.head = res;
        }
        
        self.count.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Remove resource
    pub fn remove(&mut self, res: *mut Resource) {
        if res.is_null() {
            return;
        }
        
        let mut prev: *mut Resource = core::ptr::null_mut();
        let mut curr = self.head;
        
        while !curr.is_null() {
            if curr == res {
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    if prev.is_null() {
                        self.head = (*res).sibling;
                    } else {
                        (*prev).sibling = (*res).sibling;
                    }
                }
                self.count.fetch_sub(1, Ordering::AcqRel);
                return;
            }
            prev = curr;
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { curr = (*curr).sibling; }
        }
    }
    
    /// Find resource containing address
    pub fn find(&self, addr: u64) -> Option<*mut Resource> {
        let mut curr = self.head;
        
        while !curr.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*curr).contains(addr) {
                    return Some(curr);
                }
                curr = (*curr).sibling;
            }
        }
        
        None
    }
    
    /// Find resource by type
    pub fn find_by_type(&self, resource_type: ResourceType) -> Option<*mut Resource> {
        let mut curr = self.head;
        
        while !curr.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*curr).resource_type == resource_type {
                    return Some(curr);
                }
                curr = (*curr).sibling;
            }
        }
        
        None
    }
    
    /// Find overlapping resource
    pub fn find_overlap(&self, start: u64, end: u64) -> Option<*mut Resource> {
        let mut curr = self.head;
        
        while !curr.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*curr).start <= end && (*curr).end >= start {
                    return Some(curr);
                }
                curr = (*curr).sibling;
            }
        }
        
        None
    }
    
    /// Iterate resources
    pub fn iter(&self) -> ResourceIterator {
        ResourceIterator {
            current: self.head,
        }
    }
}

/// Resource Iterator
pub struct ResourceIterator {
    current: *mut Resource,
}

impl ResourceIterator {
    pub fn next(&mut self) -> Option<*mut Resource> {
        if self.current.is_null() {
            return None;
        }
        
        let res = self.current;
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { self.current = (*res).sibling; }
        Some(res)
    }
}

/// Resource Manager
pub struct ResourceManager {
    /// I/O port resources
    pub io_resources: ResourceList,
    /// Memory resources
    pub mem_resources: ResourceList,
    /// IRQ resources
    pub irq_resources: ResourceList,
    /// DMA resources
    pub dma_resources: ResourceList,
    /// Bus resources
    pub bus_resources: ResourceList,
    /// Total allocated
    pub total_allocated: AtomicU64,
    /// Total freed
    pub total_freed: AtomicU64,
}

impl ResourceManager {
    pub const fn new() -> Self {
        ResourceManager {
            io_resources: ResourceList::new(),
            mem_resources: ResourceList::new(),
            irq_resources: ResourceList::new(),
            dma_resources: ResourceList::new(),
            bus_resources: ResourceList::new(),
            total_allocated: AtomicU64::new(0),
            total_freed: AtomicU64::new(0),
        }
    }
    
    /// Initialize
    pub fn init(&self) {
        // Register standard I/O ports
        self.register_standard_io();
        
        // Register standard memory regions
        self.register_standard_mem();
        
        log_info!("Resource manager initialized");
    }
    
    /// Register standard I/O ports
    fn register_standard_io(&mut self) {
        // PIC 1 (0x20-0x21)
        let pic1 = Resource::new_io(0x20, 2);
        // TODO: Add to list
        
        // PIC 2 (0xA0-0xA1)
        let pic2 = Resource::new_io(0xA0, 2);
        
        // PIT (0x40-0x43)
        let pit = Resource::new_io(0x40, 4);
        
        // DMA (0x00-0x0F, 0x80-0x8F)
        let dma1 = Resource::new_io(0x00, 16);
        let dma2 = Resource::new_io(0x80, 16);
        
        // RTC (0x70-0x71)
        let rtc = Resource::new_io(0x70, 2);
        
        let _ = (pic1, pic2, pit, dma1, dma2, rtc);
    }
    
    /// Register standard memory regions
    fn register_standard_mem(&mut self) {
        // TODO: Register standard memory regions
    }
    
    /// Request resource
    pub fn request_resource(&mut self, res: *mut Resource) -> i32 {
        if res.is_null() {
            return Errno::Einval.to_ret_i32();
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let list = self.get_list((*res).resource_type);
            
            // Check for conflicts
            if let Some(conflict) = list.find_overlap((*res).start, (*res).end) {
                // Check if shareable
                if !(*res).flags.contains(ResourceFlags::SHARED) ||
                   !(*conflict).flags.contains(ResourceFlags::SHARED) {
                    return Errno::Ebusy.to_ret_i32(); // EBUSY
                }
            }
            
            // Add to list
            list.add(res);
            
            self.total_allocated.fetch_add((*res).size(), Ordering::AcqRel);
        }
        
        0
    }
    
    /// Release resource
    pub fn release_resource(&mut self, res: *mut Resource) -> i32 {
        if res.is_null() {
            return Errno::Einval.to_ret_i32();
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let list = self.get_list((*res).resource_type);
            list.remove(res);
            self.total_freed.fetch_add((*res).size(), Ordering::AcqRel);
        }
        
        0
    }
    
    /// Get resource list by type
    fn get_list(&mut self, resource_type: ResourceType) -> &mut ResourceList {
        match resource_type {
            ResourceType::Io | ResourceType::MemIo => &mut self.io_resources,
            ResourceType::Mem | ResourceType::MemPref => &mut self.mem_resources,
            ResourceType::Irq => &mut self.irq_resources,
            ResourceType::Dma => &mut self.dma_resources,
            ResourceType::Bus => &mut self.bus_resources,
            _ => &mut self.mem_resources,
        }
    }
    
    /// Allocate memory region
    pub fn allocate_mem(&mut self, size: u64, align: u64, start: u64, end: u64) -> Option<u64> {
        // Find free region
        let mut addr = start;
        
        while addr + size <= end {
            if self.mem_resources.find_overlap(addr, addr + size - 1).is_none() {
                // Found free region
                let res = Resource::new_mem(addr, size);
                // TODO: Allocate and add resource
                return Some(addr);
            }
            
            // Move to next alignment boundary
            addr = (addr + align) & !(align - 1);
        }
        
        None
    }
    
    /// Allocate I/O port region
    pub fn allocate_io(&mut self, size: u64, align: u64, start: u64, end: u64) -> Option<u64> {
        let mut addr = start;
        
        while addr + size <= end {
            if self.io_resources.find_overlap(addr, addr + size - 1).is_none() {
                return Some(addr);
            }
            
            addr = (addr + align) & !(align - 1);
        }
        
        None
    }
    
    /// Request IRQ
    pub fn request_irq(&mut self, irq: u32, name: &[u8]) -> i32 {
        let mut res = Resource::new_irq(irq);
        res.set_name(name);
        
        // Check if already allocated
        if self.irq_resources.find(irq as u64).is_some() {
            return Errno::Ebusy.to_ret_i32(); // EBUSY
        }
        
        self.request_resource(&mut res as *mut Resource)
    }
    
    /// Free IRQ
    pub fn free_irq(&mut self, irq: u32) -> i32 {
        if let Some(res) = self.irq_resources.find(irq as u64) {
            self.release_resource(res)
        } else {
            -2 // ENOENT
        }
    }
    
    /// Check if address is in use
    pub fn is_address_in_use(&self, addr: u64, resource_type: ResourceType) -> bool {
        let list = match resource_type {
            ResourceType::Io | ResourceType::MemIo => &self.io_resources,
            ResourceType::Mem | ResourceType::MemPref => &self.mem_resources,
            ResourceType::Irq => &self.irq_resources,
            ResourceType::Dma => &self.dma_resources,
            ResourceType::Bus => &self.bus_resources,
            _ => &self.mem_resources,
        };
        
        list.find(addr).is_some()
    }
    
    /// Dump resources
    pub fn dump(&self) {
        log_info!("Resource allocation:");
        log_info!("  Total allocated: {} bytes", self.total_allocated.load(Ordering::Acquire));
        log_info!("  Total freed: {} bytes", self.total_freed.load(Ordering::Acquire));
        
        log_info!("  I/O ports: {} regions", self.io_resources.count.load(Ordering::Acquire));
        log_info!("  Memory: {} regions", self.mem_resources.count.load(Ordering::Acquire));
        log_info!("  IRQs: {} allocated", self.irq_resources.count.load(Ordering::Acquire));
    }
}

/// Global resource manager
static RESOURCE_MANAGER: core::sync::OnceLock<ResourceManager> = core::sync::OnceLock::new();

/// Get resource manager
pub fn resource_manager() -> &'static ResourceManager {
    RESOURCE_MANAGER.get_or_init(ResourceManager::new)
}

pub fn init_resource_manager() -> &'static ResourceManager {
    RESOURCE_MANAGER.get_or_init(ResourceManager::new)
}

/// Initialize resource manager
pub fn init_resource() {
    let mgr = resource_manager();
    mgr.init();
}

// Convenience functions

/// Request memory region
pub fn request_mem_region(start: u64, size: u64, name: &[u8]) -> i32 {
    let mut res = Resource::new_mem(start, size);
    res.set_name(name);
    resource_manager().request_resource(&mut res as *mut Resource)
}

/// Release memory region
pub fn release_mem_region(start: u64, size: u64) -> i32 {
    let mgr = resource_manager();
    if let Some(res) = mgr.mem_resources.find(start) {
        mgr.release_resource(res)
    } else {
        -2
    }
}

/// Request I/O region
pub fn request_region(start: u64, size: u64, name: &[u8]) -> i32 {
    let mut res = Resource::new_io(start, size);
    res.set_name(name);
    resource_manager().request_resource(&mut res as *mut Resource)
}

/// Release I/O region
pub fn release_region(start: u64, size: u64) -> i32 {
    let mgr = resource_manager();
    if let Some(res) = mgr.io_resources.find(start) {
        mgr.release_resource(res)
    } else {
        -2
    }
}

/// Check memory region
pub fn check_mem_region(start: u64, size: u64) -> i32 {
    let mgr = resource_manager();
    if mgr.mem_resources.find_overlap(start, start + size - 1).is_some() {
        -16 // EBUSY
    } else {
        0
    }
}

/// Allocate resource
pub fn allocate_resource(resource_type: ResourceType, size: u64, align: u64, 
                         start: u64, end: u64, name: &[u8]) -> Option<u64> {
    let mgr = resource_manager();
    
    let addr = match resource_type {
        ResourceType::Mem | ResourceType::MemPref => mgr.allocate_mem(size, align, start, end)?,
        ResourceType::Io | ResourceType::MemIo => mgr.allocate_io(size, align, start, end)?,
        _ => return None,
    };
    
    Some(addr)
}
