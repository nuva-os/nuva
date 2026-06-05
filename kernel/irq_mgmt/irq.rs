/*
 * Nuva OS - Kernel - IrqMgmt - Irq
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
/*
 * Nuva OS - Kernel - Interrupt Controller
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Interrupt controller abstraction (GIC, APIC, etc.).
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use crate::posix::errno::Errno;
/// IRQ Number
pub type IrqNumber = u32;

/// IRQ Handler Type
pub type IrqHandler = extern "C" fn(IrqNumber, *mut core::ffi::c_void);

/// IRQ Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct IrqFlags: u32 {
        /// Shared IRQ
        const SHARED = 0x01;
        /// Probe
        const PROBE = 0x02;
        /// Per-CPU
        const PERCPU = 0x04;
        /// No balloon
        const NOBALLOON = 0x08;
        /// Wake
        const WAKE = 0x10;
        /// No request
        const NOREQUEST = 0x20;
        /// No auto enable
        const NOAUTOEN = 0x40;
        /// No suspend
        const NOSUSPEND = 0x80;
        /// Force resume
        const FORCERESUME = 0x100;
        /// No thread
        const NOTHREAD = 0x200;
        /// Early resume
        const EARLYRESUME = 0x400;
        /// Cond suspend
        const CONDSUSPEND = 0x800;
    }
}

/// IRQ Return
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrqReturn {
    /// Not handled
    None = 0,
    /// Handled
    Handled = 1,
    /// Need thread
    WakeThread = 2,
}

/// IRQ Action
#[repr(C)]
pub struct IrqAction {
    /// Handler
    pub handler: Option<IrqHandler>,
    /// Thread handler
    pub thread_fn: Option<IrqHandler>,
    /// Device name
    pub name: [u8; 32],
    /// Private data
    pub dev_id: *mut core::ffi::c_void,
    /// Next action (for shared IRQ)
    pub next: *mut IrqAction,
    /// Flags
    pub flags: IrqFlags,
    /// Thread
    pub thread: u64,
    /// Secondary
    pub secondary: *mut IrqAction,
}

impl IrqAction {
    pub fn new(handler: IrqHandler, name: &[u8], dev_id: *mut core::ffi::c_void) -> Self {
        let mut name_arr = [0u8; 32];
        let len = name.len().min(31);
        name_arr[..len].copy_from_slice(&name[..len]);

        IrqAction {
            handler: Some(handler),
            thread_fn: None,
            name: name_arr,
            dev_id,
            next: core::ptr::null_mut(),
            flags: IrqFlags::empty(),
            thread: 0,
            secondary: core::ptr::null_mut(),
        }
    }
}

/// IRQ Descriptor
pub struct IrqDesc {
    /// IRQ number
    pub irq: IrqNumber,
    /// Action list
    pub action: *mut IrqAction,
    /// Status
    pub status: AtomicU32,
    /// Depth (disabled count)
    pub depth: AtomicU32,
    /// Wake depth
    pub wake_depth: AtomicU32,
    /// IRQ count
    pub count: AtomicU64,
    /// Chip data
    pub chip_data: *mut core::ffi::c_void,
    /// Affinity
    pub affinity: AtomicU64,
}

/// IRQ Status Flags
pub mod irq_status {
    pub const IRQ_INPROGRESS: u32 = 0x00000100;
    pub const IRQ_DISABLED: u32 = 0x00000200;
    pub const IRQ_MASKED: u32 = 0x00000400;
    pub const IRQ_PENDING: u32 = 0x00000800;
    pub const IRQ_REPLAY: u32 = 0x00001000;
    pub const IRQ_AUTODETECT: u32 = 0x00002000;
    pub const IRQ_WAITING: u32 = 0x00004000;
    pub const IRQ_LEVEL: u32 = 0x00008000;
    pub const IRQ_PER_CPU: u32 = 0x00010000;
    pub const IRQ_NOPROBE: u32 = 0x00020000;
    pub const IRQ_NOREQUEST: u32 = 0x00040000;
    pub const IRQ_NOAUTOEN: u32 = 0x00080000;
    pub const IRQ_NOBALANCING: u32 = 0x00100000;
    pub const IRQ_MOVE_PCNTXT: u32 = 0x00200000;
    pub const IRQ_NO_SUSPEND: u32 = 0x00400000;
    pub const IRQ_FORCE_RESUME: u32 = 0x00800000;
    pub const IRQ_NESTED: u32 = 0x01000000;
    pub const IRQ_PER_CPU_DEVID: u32 = 0x02000000;
}

impl IrqDesc {
    pub fn new(irq: IrqNumber) -> Self {
        IrqDesc {
            irq,
            action: core::ptr::null_mut(),
            status: AtomicU32::new(irq_status::IRQ_DISABLED),
            depth: AtomicU32::new(1),
            wake_depth: AtomicU32::new(0),
            count: AtomicU64::new(0),
            chip_data: core::ptr::null_mut(),
            affinity: AtomicU64::new(0xFFFF), // All CPUs
        }
    }

    /// Check if IRQ is disabled
    pub fn is_disabled(&self) -> bool {
        (self.status.load(Ordering::Acquire) & irq_status::IRQ_DISABLED) != 0
    }

    /// Check if IRQ is in progress
    pub fn is_inprogress(&self) -> bool {
        (self.status.load(Ordering::Acquire) & irq_status::IRQ_INPROGRESS) != 0
    }
}

/// Interrupt Chip Operations
pub struct IrqChipOps {
    /// Chip name
    pub name: [u8; 32],
    /// Startup IRQ
    pub startup: Option<unsafe extern "C" fn(IrqNumber, *mut core::ffi::c_void) -> u32>,
    /// Shutdown IRQ
    pub shutdown: Option<unsafe extern "C" fn(IrqNumber, *mut core::ffi::c_void)>,
    /// Enable IRQ
    pub enable: Option<unsafe extern "C" fn(IrqNumber, *mut core::ffi::c_void)>,
    /// Disable IRQ
    pub disable: Option<unsafe extern "C" fn(IrqNumber, *mut core::ffi::c_void)>,
    /// Acknowledge IRQ
    pub ack: Option<unsafe extern "C" fn(IrqNumber, *mut core::ffi::c_void)>,
    /// Mask IRQ
    pub mask: Option<unsafe extern "C" fn(IrqNumber, *mut core::ffi::c_void)>,
    /// Unmask IRQ
    pub unmask: Option<unsafe extern "C" fn(IrqNumber, *mut core::ffi::c_void)>,
    /// End of interrupt
    pub eoi: Option<unsafe extern "C" fn(IrqNumber, *mut core::ffi::c_void)>,
    /// Set affinity
    pub set_affinity: Option<unsafe extern "C" fn(IrqNumber, *mut core::ffi::c_void, u64) -> i32>,
    /// Set type
    pub set_type: Option<unsafe extern "C" fn(IrqNumber, *mut core::ffi::c_void, u32) -> i32>,
    /// Set wake
    pub set_wake: Option<unsafe extern "C" fn(IrqNumber, *mut core::ffi::c_void, u32) -> i32>,
}

/// Interrupt Controller
pub struct IrqChip {
    /// Operations
    pub ops: IrqChipOps,
    /// Number of IRQs
    pub nr_irqs: u32,
    /// First IRQ
    pub irq_base: u32,
    /// Private data
    pub data: *mut core::ffi::c_void,
    /// Next chip
    pub next: *mut IrqChip,
}

/// IRQ Trigger Type
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrqTrigger {
    /// Not specified
    None = 0,
    /// Rising edge
    Rising = 1,
    /// Falling edge
    Falling = 2,
    /// High level
    High = 4,
    /// Low level
    Low = 8,
}

/// Interrupt Controller Manager
pub struct IrqManager {
    /// IRQ descriptors
    pub irqs: [Option<IrqDesc>; 256],
    /// Number of IRQs
    pub nr_irqs: AtomicU32,
    /// Chip list
    pub chips: *mut IrqChip,
    /// Statistics
    pub stats: IrqStats,
}

/// IRQ Statistics
pub struct IrqStats {
    /// Total interrupts
    pub total: AtomicU64,
    /// Spurious
    pub spurious: AtomicU64,
    /// Handled
    pub handled: AtomicU64,
}

impl IrqStats {
    pub const fn new() -> Self {
        IrqStats {
            total: AtomicU64::new(0),
            spurious: AtomicU64::new(0),
            handled: AtomicU64::new(0),
        }
    }
}

impl IrqManager {
    pub const fn new() -> Self {
        IrqManager {
            irqs: [const { None }; 256],
            nr_irqs: AtomicU32::new(0),
            chips: core::ptr::null_mut(),
            stats: IrqStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("IRQ manager initialized");
    }

    /// Register chip
    pub fn register_chip(&mut self, chip: *mut IrqChip) -> i32 {
        if chip.is_null() {
            return Errno::Einval.to_ret_i32();
        }

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*chip).next = self.chips;
            self.chips = chip;
        }

        0
    }

    /// Request IRQ
    pub fn request_irq(&mut self, irq: IrqNumber, action: *mut IrqAction) -> i32 {
        if irq >= 256 || action.is_null() {
            return Errno::Einval.to_ret_i32();
        }

        let desc = &mut self.irqs[irq as usize];
        if desc.is_none() {
            *desc = Some(IrqDesc::new(irq));
        }

        let desc = match desc.as_mut() {
            Some(d) => d,
            None => return Err(KernelError::InvalidArgument),
        };

        // Add action to list
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*action).next = desc.action;
            desc.action = action;
        }

        // Enable IRQ
        self.enable_irq(irq);

        0
    }

    /// Free IRQ
    pub fn free_irq(&mut self, irq: IrqNumber, dev_id: *mut core::ffi::c_void) {
        if irq >= 256 {
            return;
        }

        let desc = &mut self.irqs[irq as usize];
        if let Some(desc) = desc.as_mut() {
            // Find and remove action
            let mut prev: *mut IrqAction = core::ptr::null_mut();
            let mut action = desc.action;

            while !action.is_null() {
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    if (*action).dev_id == dev_id {
                        if prev.is_null() {
                            desc.action = (*action).next;
                        } else {
                            (*prev).next = (*action).next;
                        }
                        break;
                    }
                    prev = action;
                    action = (*action).next;
                }
            }
        }
    }

    /// Enable IRQ
    pub fn enable_irq(&mut self, irq: IrqNumber) {
        if irq >= 256 {
            return;
        }

        if let Some(desc) = self.irqs[irq as usize].as_mut() {
            let depth = desc.depth.fetch_sub(1, Ordering::AcqRel);
            if depth == 1 {
                desc.status
                    .fetch_and(!irq_status::IRQ_DISABLED, Ordering::AcqRel);
                // TODO: Call chip enable
            }
        }
    }

    /// Disable IRQ
    pub fn disable_irq(&mut self, irq: IrqNumber) {
        if irq >= 256 {
            return;
        }

        if let Some(desc) = self.irqs[irq as usize].as_mut() {
            desc.depth.fetch_add(1, Ordering::AcqRel);
            desc.status
                .fetch_or(irq_status::IRQ_DISABLED, Ordering::AcqRel);
            // TODO: Call chip disable
        }
    }

    /// Handle IRQ
    pub fn handle_irq(&mut self, irq: IrqNumber) {
        self.stats.total.fetch_add(1, Ordering::AcqRel);

        if irq >= 256 {
            self.stats.spurious.fetch_add(1, Ordering::AcqRel);
            return;
        }

        if let Some(desc) = self.irqs[irq as usize].as_mut() {
            desc.count.fetch_add(1, Ordering::AcqRel);

            // Mark in progress
            desc.status
                .fetch_or(irq_status::IRQ_INPROGRESS, Ordering::AcqRel);

            // Call handlers
            let mut action = desc.action;
            while !action.is_null() {
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    if let Some(handler) = (*action).handler {
                        handler(irq, (*action).dev_id);
                    }
                    action = (*action).next;
                }
            }

            // Clear in progress
            desc.status
                .fetch_and(!irq_status::IRQ_INPROGRESS, Ordering::AcqRel);

            self.stats.handled.fetch_add(1, Ordering::AcqRel);
        }
    }

    /// Set IRQ type
    pub fn set_irq_type(&mut self, irq: IrqNumber, trigger: IrqTrigger) -> i32 {
        if irq >= 256 {
            return Errno::Einval.to_ret_i32();
        }

        // TODO: Call chip set_type
        let _ = trigger;
        0
    }

    /// Set IRQ affinity
    pub fn set_affinity(&mut self, irq: IrqNumber, affinity: u64) -> i32 {
        if irq >= 256 {
            return Errno::Einval.to_ret_i32();
        }

        if let Some(desc) = self.irqs[irq as usize].as_mut() {
            desc.affinity.store(affinity, Ordering::Release);
        }

        0
    }
}

/// Global IRQ manager
static IRQ_MANAGER: core::sync::OnceLock<IrqManager> = core::sync::OnceLock::new();

/// Get IRQ manager
pub fn irq_manager() -> &'static IrqManager {
    IRQ_MANAGER.get_or_init(IrqManager::new)
}

pub fn init_irq_manager() -> &'static IrqManager {
    IRQ_MANAGER.get_or_init(IrqManager::new)
}

/// Initialize IRQ
pub fn init_irq() {
    let mgr = irq_manager();
    mgr.init();
}

// Convenience functions

/// Request IRQ
pub fn request_irq(
    irq: IrqNumber,
    handler: IrqHandler,
    name: &[u8],
    dev_id: *mut core::ffi::c_void,
) -> i32 {
    let action = IrqAction::new(handler, name, dev_id);
    // TODO: Allocate action
    let _ = action;
    irq_manager().request_irq(irq, core::ptr::null_mut())
}

/// Free IRQ
pub fn free_irq(irq: IrqNumber, dev_id: *mut core::ffi::c_void) {
    irq_manager().free_irq(irq, dev_id);
}

/// Enable IRQ
pub fn enable_irq(irq: IrqNumber) {
    irq_manager().enable_irq(irq);
}

/// Disable IRQ
pub fn disable_irq(irq: IrqNumber) {
    irq_manager().disable_irq(irq);
}
