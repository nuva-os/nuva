/*
 * Nuva OS - Kernel - IrqMgmt - ApicOps
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
/*
 * Nuva OS - Kernel - APIC Operations
 * 
 * Advanced APIC operations implementation.
 * 
 * Copyright (C) 2026 Nuva OS Team
 */

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// APIC register addresses
pub mod apic_regs {
    pub const LOCAL_APIC_BASE: u64 = 0xFEE0_0000;
    pub const LAPIC_ID: u64 = LOCAL_APIC_BASE + 0x0020;
    pub const LAPIC_VERSION: u64 = LOCAL_APIC_BASE + 0x0030;
    pub const LAPIC_TPR: u64 = LOCAL_APIC_BASE + 0x0080;
    pub const LAPIC_EOI: u64 = LOCAL_APIC_BASE + 0x00B0;
    pub const LAPIC_SVR: u64 = LOCAL_APIC_BASE + 0x00F0;
    pub const LAPIC_ICR_LOW: u64 = LOCAL_APIC_BASE + 0x0300;
    pub const LAPIC_ICR_HIGH: u64 = LOCAL_APIC_BASE + 0x0310;
    pub const LAPIC_LVT_TIMER: u64 = LOCAL_APIC_BASE + 0x0320;
    pub const LAPIC_TIMER_ICR: u64 = LOCAL_APIC_BASE + 0x0380;
    pub const LAPIC_TIMER_DCR: u64 = LOCAL_APIC_BASE + 0x03E0;
    
    pub const IO_APIC_BASE: u64 = 0xFEC0_0000;
    pub const IOAPIC_ADDRESS: u64 = IO_APIC_BASE + 0x0000;
    pub const IOAPIC_DATA: u64 = IO_APIC_BASE + 0x0010;
}

/// Read LAPIC register
#[inline(always)]
pub fn lapic_read(reg: u64) -> u32 {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        let ptr = reg as *const u32;
        *ptr
    }
}

/// Write LAPIC register
#[inline(always)]
pub fn lapic_write(reg: u64, value: u32) {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        let ptr = reg as *mut u32;
        *ptr = value;
    }
}

/// Get LAPIC ID
pub fn get_lapic_id() -> u32 {
    (lapic_read(apic_regs::LAPIC_ID) >> 24) & 0xFF
}

/// Get LAPIC version
pub fn get_lapic_version() -> u32 {
    lapic_read(apic_regs::LAPIC_VERSION) & 0xFF
}

/// Send End of Interrupt
pub fn send_eoi() {
    lapic_write(apic_regs::LAPIC_EOI, 0);
}

/// Set Task Priority Register
pub fn set_tpr(priority: u32) {
    lapic_write(apic_regs::LAPIC_TPR, priority & 0xFF);
}

/// Get Task Priority Register
pub fn get_tpr() -> u32 {
    lapic_read(apic_regs::LAPIC_TPR) & 0xFF
}

/// Enable LAPIC
pub fn enable_lapic() {
    // Set SVR with enable bit (bit 8) and spurious vector
    lapic_write(apic_regs::LAPIC_SVR, 0x1FF);
}

/// Disable LAPIC
pub fn disable_lapic() {
    lapic_write(apic_regs::LAPIC_SVR, 0);
}

/// IPI delivery modes
pub mod ipi_mode {
    pub const FIXED: u32 = 0x000;
    pub const LOWEST: u32 = 0x100;
    pub const SMI: u32 = 0x200;
    pub const NMI: u32 = 0x400;
    pub const INIT: u32 = 0x500;
    pub const STARTUP: u32 = 0x600;
}

/// Send IPI to specific CPU
pub fn send_ipi(dest: u32, vector: u8, mode: u32) {
    // Write destination to ICR high
    lapic_write(apic_regs::LAPIC_ICR_HIGH, dest << 24);
    
    // Write command to ICR low (with trigger mode and level)
    lapic_write(apic_regs::LAPIC_ICR_LOW, mode | (vector as u32) | 0x4000); // Level trigger
}

/// Send broadcast IPI
pub fn broadcast_ipi(vector: u8, mode: u32) {
    // All excluding self: destination shorthand = 0x3
    lapic_write(apic_regs::LAPIC_ICR_LOW, mode | (vector as u32) | 0xC000);
}

/// Set up LAPIC timer
pub fn setup_timer(vector: u8, initial_count: u32, divide_config: u32) {
    // Set divide configuration (0=1, 1=2, 2=4, 3=8, 8=16, 9=32, 10=64, 11=128)
    lapic_write(apic_regs::LAPIC_TIMER_DCR, divide_config & 0x0B);
    
    // Set initial count
    lapic_write(apic_regs::LAPIC_TIMER_ICR, initial_count);
    
    // Set LVT timer register with vector
    lapic_write(apic_regs::LAPIC_LVT_TIMER, vector as u32);
}

/// One-shot timer mode
pub fn set_timer_oneshot(vector: u8) {
    lapic_write(apic_regs::LAPIC_LVT_TIMER, vector as u32);
}

/// Periodic timer mode
pub fn set_timer_periodic(vector: u8) {
    lapic_write(apic_regs::LAPIC_LVT_TIMER, (vector as u32) | 0x20000);
}

/// Read IOAPIC register
pub fn ioapic_read(reg: u32) -> u32 {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        let addr_ptr = apic_regs::IOAPIC_ADDRESS as *mut u32;
        let data_ptr = apic_regs::IOAPIC_DATA as *const u32;
        *addr_ptr = reg;
        *data_ptr
    }
}

/// Write IOAPIC register
pub fn ioapic_write(reg: u32, value: u32) {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        let addr_ptr = apic_regs::IOAPIC_ADDRESS as *mut u32;
        let data_ptr = apic_regs::IOAPIC_DATA as *mut u32;
        *addr_ptr = reg;
        *data_ptr = value;
    }
}

/// Get IOAPIC ID
pub fn get_ioapic_id() -> u32 {
    (ioapic_read(0x00) >> 24) & 0x0F
}

/// Get IOAPIC version
pub fn get_ioapic_version() -> u32 {
    ioapic_read(0x01) & 0xFF
}

/// Get maximum IRQ number
pub fn get_ioapic_max_irq() -> u32 {
    (ioapic_read(0x01) >> 16) & 0xFF
}

/// Set IRQ redirection
pub fn set_irq_redirect(irq: u8, vector: u8, delivery_mode: u8, dest: u8) {
    // Redirection table entry is at 0x10 + 2*irq
    let reg_low = 0x10 + 2 * (irq as u32);
    let reg_high = reg_low + 1;
    
    // Low 32 bits: vector, delivery mode, destination mode, etc.
    let low: u32 = (vector as u32)
        | ((delivery_mode as u32) << 8)
        | (0 << 11)    // Destination mode: physical
        | (0 << 13)    // Delivery status
        | (0 << 15)    // Mask: not masked
        | (0 << 16);   // Trigger mode: edge
    
    // High 32 bits: destination
    let high: u32 = (dest as u32) << 24;
    
    ioapic_write(reg_low, low);
    ioapic_write(reg_high, high);
}

/// Mask IRQ
pub fn mask_irq(irq: u8) {
    let reg_low = 0x10 + 2 * (irq as u32);
    let current = ioapic_read(reg_low);
    ioapic_write(reg_low, current | (1 << 16)); // Set mask bit
}

/// Unmask IRQ
pub fn unmask_irq(irq: u8) {
    let reg_low = 0x10 + 2 * (irq as u32);
    let current = ioapic_read(reg_low);
    ioapic_write(reg_low, current & !(1 << 16)); // Clear mask bit
}

/// APIC statistics
#[repr(C)]
pub struct ApicStats {
    pub ipi_count: AtomicU64,
    pub eoi_count: AtomicU64,
    pub timer_count: AtomicU64,
    pub spurious_count: AtomicU64,
}

impl ApicStats {
    pub const fn new() -> Self {
        ApicStats {
            ipi_count: AtomicU64::new(0),
            eoi_count: AtomicU64::new(0),
            timer_count: AtomicU64::new(0),
            spurious_count: AtomicU64::new(0),
        }
    }
}

/// Global APIC statistics
pub static APIC_STATS: ApicStats = ApicStats::new();

/// Initialize APIC subsystem
pub fn init_apic_ops() {
    log_info!("Initializing APIC operations...");
    
    // Enable LAPIC
    enable_lapic();
    
    // Get and log info
    let id = get_lapic_id();
    let version = get_lapic_version();
    log_info!("LAPIC ID: {}, Version: {}", id, version);
    
    // Initialize IOAPIC
    let ioapic_id = get_ioapic_id();
    let ioapic_ver = get_ioapic_version();
    let max_irq = get_ioapic_max_irq();
    log_info!("IOAPIC ID: {}, Version: {}, Max IRQ: {}", ioapic_id, ioapic_ver, max_irq);
    
    log_info!("APIC operations initialized");
}
