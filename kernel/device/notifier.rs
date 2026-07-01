/*
 * Nuva OS - Kernel - Device - Notifier
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
use crate::{pr_info};
use crate::syslib::posix::errno::Errno;
/*
 * Nuva OS - Kernel - Notifier Chain
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Kernel notification chain for event broadcasting.
 */

use core::sync::atomic::{AtomicU32, AtomicPtr, Ordering};
use crate::kernel::error::Errno;
use alloc::boxed::Box;

/// Notifier Return Values
pub const NOTIFY_OK: i32 = 0x0001;
pub const NOTIFY_STOP: i32 = 0x0002;
pub const NOTIFY_BAD: i32 = 0x0004;
pub const NOTIFY_DONE: i32 = 0x0000;

/// Notifier Priority
pub const NOTIFY_PRIO_HIGHEST: i32 = 1000;
pub const NOTIFY_PRIO_HIGH: i32 = 100;
pub const NOTIFY_PRIO_NORMAL: i32 = 0;
pub const NOTIFY_PRIO_LOW: i32 = -100;
pub const NOTIFY_PRIO_LOWEST: i32 = -1000;

/// Notifier Callback
pub type NotifierFn = unsafe extern "C" fn(*mut core::ffi::c_void, u32, *mut core::ffi::c_void) -> i32;

/// Notifier Block (deprecated: use EventListener trait instead)
#[deprecated(since = "1.0.0", note = "Use EventListener trait with EventBus instead")]
#[repr(C)]
pub struct NotifierBlock {
    /// Callback function
    pub notifier_call: NotifierFn,
    /// Priority
    pub priority: i32,
    /// Next block
    pub next: *mut NotifierBlock,
}

impl NotifierBlock {
    pub const fn new(callback: NotifierFn, priority: i32) -> Self {
        NotifierBlock {
            notifier_call: callback,
            priority,
            next: core::ptr::null_mut(),
        }
    }
}

/// Notifier Head (deprecated: use EventBus instead)
#[deprecated(since = "1.0.0", note = "Use EventBus instead")]
pub struct NotifierHead {
    /// First block
    pub head: AtomicPtr<NotifierBlock>,
    /// Lock
    pub lock: AtomicU32,
}

impl NotifierHead {
    pub const fn new() -> Self {
        NotifierHead {
            head: AtomicPtr::new(core::ptr::null_mut()),
            lock: AtomicU32::new(0),
        }
    }
    
    /// Lock
    fn lock(&self) {
        while self.lock.compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire).is_err() {
            core::hint::spin_loop();
        }
    }
    
    /// Unlock
    fn unlock(&self) {
        self.lock.store(0, Ordering::Release);
    }
    
    /// Register notifier
    pub fn register(&self, nb: *mut NotifierBlock) -> i32 {
        if nb.is_null() {
            return Errno::Einval.to_ret_i32();
        }
        
        self.lock();
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Find insertion point
            let mut prev = core::ptr::null_mut();
            let mut curr = self.head.load(Ordering::Acquire);
            
            while !curr.is_null() && (*curr).priority >= (*nb).priority {
                prev = curr;
                curr = (*curr).next;
            }
            
            // Insert
            (*nb).next = curr;
            
            if prev.is_null() {
                self.head.store(nb, Ordering::Release);
            } else {
                (*prev).next = nb;
            }
        }
        
        self.unlock();
        0
    }
    
    /// Unregister notifier
    pub fn unregister(&self, nb: *mut NotifierBlock) -> i32 {
        if nb.is_null() {
            return Errno::Einval.to_ret_i32();
        }
        
        self.lock();
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let mut prev = core::ptr::null_mut();
            let mut curr = self.head.load(Ordering::Acquire);
            
            while !curr.is_null() && curr != nb {
                prev = curr;
                curr = (*curr).next;
            }
            
            if curr.is_null() {
                self.unlock();
                return Errno::Enoent.to_ret_i32();
            }
            
            if prev.is_null() {
                self.head.store((*nb).next, Ordering::Release);
            } else {
                (*prev).next = (*nb).next;
            }
            
            (*nb).next = core::ptr::null_mut();
        }
        
        self.unlock();
        0
    }
    
    /// Call notifiers
    pub fn call(&self, val: u32, data: *mut core::ffi::c_void) -> i32 {
        let mut ret = NOTIFY_DONE;
        
        self.lock();
        
        let mut nb = self.head.load(Ordering::Acquire);
        
        while !nb.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let cb = (*nb).notifier_call;
                let r = cb(nb as *mut core::ffi::c_void, val, data);
                
                if (r & NOTIFY_STOP) != 0 {
                    ret = r;
                    break;
                }
                
                if (r & NOTIFY_BAD) != 0 {
                    ret = NOTIFY_BAD;
                    break;
                }
                
                ret |= r;
                nb = (*nb).next;
            }
        }
        
        self.unlock();
        ret
    }
}

/// Notifier Event Types
pub mod notifier_event {
    // Reboot events
    pub const SYS_REBOOT: u32 = 0x0001;
    pub const SYS_POWER_OFF: u32 = 0x0002;
    pub const SYS_RESTART: u32 = 0x0003;
    pub const SYS_HALT: u32 = 0x0004;
    
    // Power events
    pub const PM_SUSPEND: u32 = 0x0010;
    pub const PM_RESUME: u32 = 0x0011;
    pub const PM_HIBERNATE: u32 = 0x0012;
    pub const PM_RESTORE: u32 = 0x0013;
    
    // CPU events
    pub const CPU_ONLINE: u32 = 0x0020;
    pub const CPU_OFFLINE: u32 = 0x0021;
    pub const CPU_UP_PREPARE: u32 = 0x0022;
    pub const CPU_DOWN_PREPARE: u32 = 0x0023;
    pub const CPU_UP_CANCELED: u32 = 0x0024;
    pub const CPU_DOWN_CANCELED: u32 = 0x0025;
    pub const CPU_ONLINE_FAILED: u32 = 0x0026;
    pub const CPU_DOWN_FAILED: u32 = 0x0027;
    
    // Memory events
    pub const MEM_ONLINE: u32 = 0x0030;
    pub const MEM_OFFLINE: u32 = 0x0031;
    pub const MEM_GOING_ONLINE: u32 = 0x0032;
    pub const MEM_GOING_OFFLINE: u32 = 0x0033;
    pub const MEM_CANCEL_ONLINE: u32 = 0x0034;
    pub const MEM_CANCEL_OFFLINE: u32 = 0x0035;
    
    // Device events
    pub const DEV_ADD: u32 = 0x0040;
    pub const DEV_REMOVE: u32 = 0x0041;
    pub const DEV_CHANGE: u32 = 0x0042;
    pub const DEV_BIND: u32 = 0x0043;
    pub const DEV_UNBIND: u32 = 0x0044;
    pub const DEV_REGISTER: u32 = 0x0045;
    pub const DEV_UNREGISTER: u32 = 0x0046;
    
    // Network events
    pub const NETDEV_UP: u32 = 0x0050;
    pub const NETDEV_DOWN: u32 = 0x0051;
    pub const NETDEV_REBOOT: u32 = 0x0052;
    pub const NETDEV_CHANGE: u32 = 0x0053;
    pub const NETDEV_REGISTER: u32 = 0x0054;
    pub const NETDEV_UNREGISTER: u32 = 0x0055;
    pub const NETDEV_CHANGEMTU: u32 = 0x0056;
    pub const NETDEV_CHANGEADDR: u32 = 0x0057;
    pub const NETDEV_GOING_DOWN: u32 = 0x0058;
    pub const NETDEV_CHANGENAME: u32 = 0x0059;
    pub const NETDEV_FEAT_CHANGE: u32 = 0x005A;
    pub const NETDEV_BONDING_FAILOVER: u32 = 0x005B;
    pub const NETDEV_PRE_TYPE_CHANGE: u32 = 0x005C;
    pub const NETDEV_POST_TYPE_CHANGE: u32 = 0x005D;
    pub const NETDEV_POST_INIT: u32 = 0x005E;
    pub const NETDEV_PRE_UNINIT: u32 = 0x005F;
    
    // Clock events
    pub const CLOCK_NOTIFY: u32 = 0x0060;
    pub const CLOCK_SET_RATE: u32 = 0x0061;
    pub const CLOCK_ENABLE: u32 = 0x0062;
    pub const CLOCK_DISABLE: u32 = 0x0063;
    
    // Process events
    pub const PROC_EVENT_FORK: u32 = 0x0070;
    pub const PROC_EVENT_EXEC: u32 = 0x0071;
    pub const PROC_EVENT_UID: u32 = 0x0072;
    pub const PROC_EVENT_GID: u32 = 0x0073;
    pub const PROC_EVENT_SID: u32 = 0x0074;
    pub const PROC_EVENT_PTRACE: u32 = 0x0075;
    pub const PROC_EVENT_COMM: u32 = 0x0076;
    pub const PROC_EVENT_COREDUMP: u32 = 0x0077;
    pub const PROC_EVENT_EXIT: u32 = 0x0078;
}

/// Notifier Manager
pub struct NotifierManager {
    /// Reboot chain
    pub reboot_chain: NotifierHead,
    /// Power chain
    pub power_chain: NotifierHead,
    /// CPU chain
    pub cpu_chain: NotifierHead,
    /// Memory chain
    pub memory_chain: NotifierHead,
    /// Device chain
    pub device_chain: NotifierHead,
    /// Network chain
    pub net_chain: NotifierHead,
    /// Clock chain
    pub clock_chain: NotifierHead,
    /// Process chain
    pub proc_chain: NotifierHead,
}

impl NotifierManager {
    pub const fn new() -> Self {
        NotifierManager {
            reboot_chain: NotifierHead::new(),
            power_chain: NotifierHead::new(),
            cpu_chain: NotifierHead::new(),
            memory_chain: NotifierHead::new(),
            device_chain: NotifierHead::new(),
            net_chain: NotifierHead::new(),
            clock_chain: NotifierHead::new(),
            proc_chain: NotifierHead::new(),
        }
    }
    
    /// Initialize (no-op with OnceLock, initialization happens on first access)
    pub fn init(&self) {
        log_info!("Notifier manager initialized");
    }
    
    /// Get chain for event type
    pub fn get_chain(&self, event_type: u32) -> &NotifierHead {
        match event_type & 0xFF00 {
            0x0000 => &self.reboot_chain,
            0x0010 => &self.power_chain,
            0x0020 => &self.cpu_chain,
            0x0030 => &self.memory_chain,
            0x0040 => &self.device_chain,
            0x0050 => &self.net_chain,
            0x0060 => &self.clock_chain,
            0x0070 => &self.proc_chain,
            _ => &self.device_chain,
        }
    }
    
    /// Register notifier
    pub fn register(&self, event_type: u32, nb: *mut NotifierBlock) -> i32 {
        self.get_chain(event_type).register(nb)
    }
    
    /// Unregister notifier
    pub fn unregister(&self, event_type: u32, nb: *mut NotifierBlock) -> i32 {
        self.get_chain(event_type).unregister(nb)
    }
    
    /// Notify event
    pub fn notify(&self, event_type: u32, data: *mut core::ffi::c_void) -> i32 {
        self.get_chain(event_type).call(event_type, data)
    }
}

/// Global notifier manager
static EVENT_BUS: crate::sync_oncelock::OnceLock<NotifierManager> = crate::sync_oncelock::OnceLock::new();

/// Get notifier manager
pub fn notifier_manager() -> &'static NotifierManager {
    EVENT_BUS.get_or_init(NotifierManager::new)
}

pub fn init_notifier_manager() -> &'static NotifierManager {
    EVENT_BUS.get_or_init(NotifierManager::new)
}

/// Initialize notifier
pub fn init_notifier() {
    let mgr = notifier_manager();
    mgr.init();
}

// Convenience functions

/// Register reboot notifier
pub fn register_reboot_listener(nb: *mut NotifierBlock) -> i32 {
    notifier_manager().reboot_chain.register(nb)
}

/// Register power notifier
pub fn register_power_listener(nb: *mut NotifierBlock) -> i32 {
    notifier_manager().power_chain.register(nb)
}

/// Register CPU notifier
pub fn register_cpu_listener(nb: *mut NotifierBlock) -> i32 {
    notifier_manager().cpu_chain.register(nb)
}

/// Register device notifier
pub fn register_device_listener(nb: *mut NotifierBlock) -> i32 {
    notifier_manager().device_chain.register(nb)
}

/// Register network notifier
pub fn register_net_device_listener(nb: *mut NotifierBlock) -> i32 {
    notifier_manager().net_chain.register(nb)
}

/// Notify reboot
pub fn notify_reboot(event: u32) -> i32 {
    notifier_manager().reboot_chain.call(event, core::ptr::null_mut())
}

/// Notify power event
pub fn notify_pm(event: u32, data: *mut core::ffi::c_void) -> i32 {
    notifier_manager().power_chain.call(event, data)
}

/// Notify CPU event
pub fn notify_cpu(event: u32, cpu: u32) -> i32 {
    notifier_manager().cpu_chain.call(event, cpu as *mut core::ffi::c_void)
}

/// Notify device event
pub fn notify_device(event: u32, data: *mut core::ffi::c_void) -> i32 {
    notifier_manager().device_chain.call(event, data)
}

/// Notify network event
pub fn notify_netdev(event: u32, data: *mut core::ffi::c_void) -> i32 {
    notifier_manager().net_chain.call(event, data)
}

/// Notify process event
pub fn notify_proc(event: u32, data: *mut core::ffi::c_void) -> i32 {
    notifier_manager().proc_chain.call(event, data)
}

/// Atomic Notifier Head (for use in interrupt context)
pub struct AtomicNotifierHead {
    pub head: AtomicPtr<NotifierBlock>,
}

impl AtomicNotifierHead {
    pub const fn new() -> Self {
        AtomicNotifierHead {
            head: AtomicPtr::new(core::ptr::null_mut()),
        }
    }
    
    /// Call notifiers (lock-free)
    pub fn call(&self, val: u32, data: *mut core::ffi::c_void) -> i32 {
        let mut ret = NOTIFY_DONE;
        let mut nb = self.head.load(Ordering::Acquire);
        
        while !nb.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let cb = (*nb).notifier_call;
                let r = cb(nb as *mut core::ffi::c_void, val, data);
                
                if (r & NOTIFY_STOP) != 0 {
                    ret = r;
                    break;
                }
                
                ret |= r;
                nb = (*nb).next;
            }
        }
        
        ret
    }
}

/// Blocking Notifier Head (for use in process context)
pub struct BlockingNotifierHead {
    pub head: NotifierHead,
}

impl BlockingNotifierHead {
    pub const fn new() -> Self {
        BlockingNotifierHead {
            head: NotifierHead::new(),
        }
    }
    
    /// Register
    pub fn register(&self, nb: *mut NotifierBlock) -> i32 {
        self.head.register(nb)
    }
    
    /// Unregister
    pub fn unregister(&self, nb: *mut NotifierBlock) -> i32 {
        self.head.unregister(nb)
    }
    
    /// Call
    pub fn call(&self, val: u32, data: *mut core::ffi::c_void) -> i32 {
        self.head.call(val, data)
    }
}

/// Raw Notifier Head (no locking)
pub struct RawNotifierHead {
    pub head: AtomicPtr<NotifierBlock>,
}

impl RawNotifierHead {
    pub const fn new() -> Self {
        RawNotifierHead {
            head: AtomicPtr::new(core::ptr::null_mut()),
        }
    }
    
    /// Call notifiers (no locking)
    pub fn call(&self, val: u32, data: *mut core::ffi::c_void) -> i32 {
        let mut ret = NOTIFY_DONE;
        let mut nb = self.head.load(Ordering::Acquire);
        
        while !nb.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let cb = (*nb).notifier_call;
                let r = cb(nb as *mut core::ffi::c_void, val, data);
                
                if (r & NOTIFY_STOP) != 0 {
                    ret = r;
                    break;
                }
                
                ret |= r;
                nb = (*nb).next;
            }
        }
        
        ret
    }
}

// ============================================================================
// Modern Event System (Phase 6)
// ============================================================================

/// Result of event handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResult {
    /// Continue notifying other listeners
    Continue,
    /// Event was handled, stop propagation
    Handled,
    /// Error occurred during handling
    Error(Errno),
}

/// Category of kernel events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EventCategory {
    System = 0x00,
    Power = 0x01,
    Cpu = 0x02,
    Memory = 0x03,
    Device = 0x04,
    Network = 0x05,
    Clock = 0x06,
    Process = 0x07,
}

/// Kernel event descriptor
#[derive(Debug, Clone)]
pub struct Event {
    /// Event category
    pub category: EventCategory,
    /// Event type within category
    pub event_type: u32,
    /// Event data pointer
    pub data: *mut core::ffi::c_void,
}

impl Event {
    pub fn new(category: EventCategory, event_type: u32, data: *mut core::ffi::c_void) -> Self {
        Event { category, event_type, data }
    }
}

/// Priority for event listeners
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EventPriority(i32);

impl EventPriority {
    pub const HIGHEST: EventPriority = EventPriority(1000);
    pub const HIGH: EventPriority = EventPriority(100);
    pub const NORMAL: EventPriority = EventPriority(0);
    pub const LOW: EventPriority = EventPriority(-100);
    pub const LOWEST: EventPriority = EventPriority(-1000);

    pub const fn new(val: i32) -> Self {
        EventPriority(val)
    }

    pub const fn value(self) -> i32 {
        self.0
    }
}

/// Trait for type-safe event listeners
pub trait EventListener {
    /// Handle an event, return result indicating propagation behavior
    fn on_event(&self, event: &Event) -> EventResult;

    /// Priority of this listener (higher = called first)
    fn priority(&self) -> EventPriority {
        EventPriority::NORMAL
    }
}

/// Slot for storing an EventListener reference in the bus
struct ListenerSlot {
    listener: &'static dyn EventListener,
    priority: EventPriority,
    next: *mut ListenerSlot,
}

/// Modern type-safe event bus
pub struct EventBus {
    heads: [AtomicPtr<ListenerSlot>; 8],
    locks: [AtomicU32; 8],
}

impl EventBus {
    pub const fn new() -> Self {
        EventBus {
            heads: [
                AtomicPtr::new(core::ptr::null_mut()),
                AtomicPtr::new(core::ptr::null_mut()),
                AtomicPtr::new(core::ptr::null_mut()),
                AtomicPtr::new(core::ptr::null_mut()),
                AtomicPtr::new(core::ptr::null_mut()),
                AtomicPtr::new(core::ptr::null_mut()),
                AtomicPtr::new(core::ptr::null_mut()),
                AtomicPtr::new(core::ptr::null_mut()),
            ],
            locks: [
                AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
                AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0),
            ],
        }
    }

    fn category_index(cat: EventCategory) -> usize {
        cat as usize
    }

    fn lock_category(&self, cat: EventCategory) {
        let idx = Self::category_index(cat);
        while self.locks[idx].compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire).is_err() {
            core::hint::spin_loop();
        }
    }

    fn unlock_category(&self, cat: EventCategory) {
        let idx = Self::category_index(cat);
        self.locks[idx].store(0, Ordering::Release);
    }

    /// Subscribe a listener to a specific event category
    pub fn subscribe(&self, category: EventCategory, listener: &'static dyn EventListener) -> Result<(), Errno> {
        self.lock_category(category);
        let idx = Self::category_index(category);
        let priority = listener.priority();

        let slot = alloc::boxed::Box::new(ListenerSlot {
            listener,
            priority,
            next: core::ptr::null_mut(),
        });
        let slot_ptr = alloc::boxed::Box::into_raw(slot);

        // SAFETY: slot_ptr is valid, we just created it
        unsafe {
            let mut prev: *mut ListenerSlot = core::ptr::null_mut();
            let mut curr = self.heads[idx].load(Ordering::Acquire);

            while !curr.is_null() && (*curr).priority >= priority {
                prev = curr;
                curr = (*curr).next;
            }

            (*slot_ptr).next = curr;

            if prev.is_null() {
                self.heads[idx].store(slot_ptr, Ordering::Release);
            } else {
                (*prev).next = slot_ptr;
            }
        }

        self.unlock_category(category);
        Ok(())
    }

    /// Unsubscribe a listener (by pointer identity)
    pub fn unsubscribe(&self, category: EventCategory, listener: &'static dyn EventListener) -> Result<(), Errno> {
        self.lock_category(category);
        let idx = Self::category_index(category);
        let listener_ptr = listener as *const dyn EventListener as *mut ListenerSlot;

        // SAFETY: walking the linked list with lock held
        unsafe {
            let mut prev: *mut ListenerSlot = core::ptr::null_mut();
            let mut curr = self.heads[idx].load(Ordering::Acquire);

            while !curr.is_null() {
                let curr_listener = (*curr).listener as *const dyn EventListener;
                if core::ptr::eq(curr_listener, listener as *const dyn EventListener) {
                    if prev.is_null() {
                        self.heads[idx].store((*curr).next, Ordering::Release);
                    } else {
                        (*prev).next = (*curr).next;
                    }
                    let _ = alloc::boxed::Box::from_raw(curr);
                    self.unlock_category(category);
                    return Ok(());
                }
                prev = curr;
                curr = (*curr).next;
            }
        }

        self.unlock_category(category);
        Err(Errno::Enoent)
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event: &Event) -> EventResult {
        let idx = Self::category_index(event.category);
        self.lock_category(event.category);

        let mut result = EventResult::Continue;
        let mut curr = self.heads[idx].load(Ordering::Acquire);

        while !curr.is_null() {
            // SAFETY: walking list with lock held
            unsafe {
                let r = (*curr).listener.on_event(event);
                match r {
                    EventResult::Continue => {}
                    EventResult::Handled => {
                        result = EventResult::Handled;
                        break;
                    }
                    EventResult::Error(e) => {
                        result = EventResult::Error(e);
                        break;
                    }
                }
                curr = (*curr).next;
            }
        }

        self.unlock_category(event.category);
        result
    }
}
