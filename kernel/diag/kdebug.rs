/*
 * Nuva OS - Kernel - Diag - Kdebug
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
use crate::{pr_debug, pr_info};
/*
 * Nuva OS - Kernel - Kernel Debugger
 * 
 * In-kernel debugger support.
 * 
 * Copyright (C) 2026 Nuva OS Team
 */

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};

/// Breakpoint type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakpointType {
    Execute = 0,
    Write = 1,
    IoReadWrite = 2,
    ReadWrite = 3,
}

/// Breakpoint size
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakpointSize {
    Byte = 0,
    Word = 1,
    Qword = 2,  // or Dword on x86
    Dqword = 3, // or Qword on x86
}

/// Hardware breakpoint
#[repr(C)]
pub struct HwBreakpoint {
    pub enabled: bool,
    pub addr: u64,
    pub bp_type: BreakpointType,
    pub size: BreakpointSize,
    pub hit_count: AtomicU64,
}

impl HwBreakpoint {
    pub fn new(addr: u64, bp_type: BreakpointType, size: BreakpointSize) -> Self {
        HwBreakpoint {
            enabled: false,
            addr,
            bp_type,
            size,
            hit_count: AtomicU64::new(0),
        }
    }
}

/// Debug register (DR0-DR7)
pub mod debug_reg {
    pub const DR0: u32 = 0;
    pub const DR1: u32 = 1;
    pub const DR2: u32 = 2;
    pub const DR3: u32 = 3;
    pub const DR6: u32 = 6;
    pub const DR7: u32 = 7;
}

/// Debug status (DR6) bits
pub mod dr6_bits {
    pub const B0: u64 = 1 << 0;
    pub const B1: u64 = 1 << 1;
    pub const B2: u64 = 1 << 2;
    pub const B3: u64 = 1 << 3;
    pub const BD: u64 = 1 << 13;
    pub const BS: u64 = 1 << 14;
    pub const BT: u64 = 1 << 15;
}

/// Debug control (DR7) bits
pub mod dr7_bits {
    pub const L0: u64 = 1 << 0;
    pub const G0: u64 = 1 << 1;
    pub const L1: u64 = 1 << 2;
    pub const G1: u64 = 1 << 3;
    pub const L2: u64 = 1 << 4;
    pub const G2: u64 = 1 << 5;
    pub const L3: u64 = 1 << 6;
    pub const G3: u64 = 1 << 7;
    pub const LE: u64 = 1 << 8;
    pub const GE: u64 = 1 << 9;
    pub const GD: u64 = 1 << 13;
}

/// Stack frame
#[repr(C)]
pub struct StackFrame {
    pub rbp: u64,
    pub rip: u64,
    pub rflags: u64,
    pub cs: u64,
    pub ss: u64,
}

/// Call stack entry
#[repr(C)]
pub struct CallStackEntry {
    pub addr: u64,
    pub symbol: [u8; 64],
    pub offset: u32,
    pub module: [u8; 32],
}

/// Watchpoint
pub struct Watchpoint {
    pub addr: u64,
    pub size: usize,
    pub access: BreakpointType,
    pub callback: Option<unsafe fn(u64, *const u8)>,
}

/// Kernel debugger
pub struct KernelDebugger {
    /// Enabled
    pub enabled: AtomicBool,
    /// Hardware breakpoints
    pub hw_bps: [HwBreakpoint; 4],
    /// Software breakpoints
    pub sw_bps: spin::Mutex<alloc::collections::BTreeMap<u64, u8>>, // addr -> original byte
    /// Watchpoints
    pub watchpoints: spin::Mutex<alloc::vec::Vec<Watchpoint>>,
    /// Single step mode
    pub single_step: AtomicBool,
    /// Break on exception
    pub break_on_exception: AtomicBool,
    /// Current call stack
    pub call_stack: spin::Mutex<alloc::vec::Vec<CallStackEntry>>,
    /// Debug output
    pub output_fn: Option<unsafe fn(&[u8])>,
}

impl KernelDebugger {
    pub fn new() -> Self {
        KernelDebugger {
            enabled: AtomicBool::new(false),
            hw_bps: [
                HwBreakpoint::new(0, BreakpointType::Execute, BreakpointSize::Byte),
                HwBreakpoint::new(0, BreakpointType::Execute, BreakpointSize::Byte),
                HwBreakpoint::new(0, BreakpointType::Execute, BreakpointSize::Byte),
                HwBreakpoint::new(0, BreakpointType::Execute, BreakpointSize::Byte),
            ],
            sw_bps: spin::Mutex::new(alloc::collections::BTreeMap::new()),
            watchpoints: spin::Mutex::new(alloc::vec::Vec::new()),
            single_step: AtomicBool::new(false),
            break_on_exception: AtomicBool::new(true),
            call_stack: spin::Mutex::new(alloc::vec::Vec::new()),
            output_fn: None,
        }
    }
    
    /// Enable debugger
    pub fn enable(&mut self) {
        self.enabled.store(true, Ordering::Release);
        log_info!("Kernel debugger enabled");
    }
    
    /// Disable debugger
    pub fn disable(&mut self) {
        self.enabled.store(false, Ordering::Release);
    }
    
    /// Set hardware breakpoint
    pub fn set_hw_breakpoint(&mut self, idx: usize, addr: u64, bp_type: BreakpointType, size: BreakpointSize) -> Result<(), i32> {
        if idx >= 4 {
            return Err(-22);
        }
        
        self.hw_bps[idx] = HwBreakpoint::new(addr, bp_type, size);
        self.hw_bps[idx].enabled = true;
        
        // Update DR7
        let mut dr7: u64 = 0;
        
        // Set local and global enable bits
        match idx {
            0 => dr7 |= dr7_bits::L0 | dr7_bits::G0,
            1 => dr7 |= dr7_bits::L1 | dr7_bits::G1,
            2 => dr7 |= dr7_bits::L2 | dr7_bits::G2,
            3 => dr7 |= dr7_bits::L3 | dr7_bits::G3,
            _ => {}
        }
        
        // Set type and size (bits 16-31)
        let type_bits = (bp_type as u64) << (16 + idx * 4);
        let size_bits = (size as u64) << (18 + idx * 4);
        dr7 |= type_bits | size_bits;
        
        // Write debug registers
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            match idx {
                0 => core::arch::asm!("mov dr0, {0}", in(reg) addr),
                1 => core::arch::asm!("mov dr1, {0}", in(reg) addr),
                2 => core::arch::asm!("mov dr2, {0}", in(reg) addr),
                3 => core::arch::asm!("mov dr3, {0}", in(reg) addr),
                _ => {}
            }
            core::arch::asm!("mov dr7, {0}", in(reg) dr7);
        }
        
        log_debug!("Hardware breakpoint {} set at {:#x}", idx, addr);
        Ok(())
    }
    
    /// Clear hardware breakpoint
    pub fn clear_hw_breakpoint(&mut self, idx: usize) -> Result<(), i32> {
        if idx >= 4 {
            return Err(-22);
        }
        
        self.hw_bps[idx].enabled = false;
        
        // Clear DR7 bits
        let mut dr7: u64;
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("mov {0}, dr7", out(reg) dr7);
        }
        
        match idx {
            0 => dr7 &= !(dr7_bits::L0 | dr7_bits::G0),
            1 => dr7 &= !(dr7_bits::L1 | dr7_bits::G1),
            2 => dr7 &= !(dr7_bits::L2 | dr7_bits::G2),
            3 => dr7 &= !(dr7_bits::L3 | dr7_bits::G3),
            _ => {}
        }
        
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("mov dr7, {0}", in(reg) dr7);
        }
        
        Ok(())
    }
    
    /// Set software breakpoint (INT3)
    pub fn set_sw_breakpoint(&mut self, addr: u64) -> Result<(), i32> {
        let mut sw_bps = self.sw_bps.lock();
        
        if sw_bps.contains_key(&addr) {
            return Err(-17); // EEXIST
        }
        
        // Save original byte and write INT3 (0xCC)
        // SAFETY: unsafe block required for low-level memory or hardware access
        let original = unsafe { *(addr as *const u8) };
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { *(addr as *mut u8) = 0xCC; }
        
        sw_bps.insert(addr, original);
        log_debug!("Software breakpoint set at {:#x}", addr);
        Ok(())
    }
    
    /// Clear software breakpoint
    pub fn clear_sw_breakpoint(&mut self, addr: u64) -> Result<(), i32> {
        let mut sw_bps = self.sw_bps.lock();
        
        if let Some(original) = sw_bps.remove(&addr) {
            // Restore original byte
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { *(addr as *mut u8) = original; }
            Ok(())
        } else {
            Err(-2)
        }
    }
    
    /// Enable single stepping
    pub fn enable_single_step(&mut self) {
        self.single_step.store(true, Ordering::Release);
        
        // Set TF flag in RFLAGS
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let mut rflags: u64;
            core::arch::asm!("pushf; pop {0}", out(reg) rflags);
            rflags |= 0x100; // TF bit
            core::arch::asm!("push {0}; popf", in(reg) rflags);
        }
    }
    
    /// Disable single stepping
    pub fn disable_single_step(&mut self) {
        self.single_step.store(false, Ordering::Release);
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let mut rflags: u64;
            core::arch::asm!("pushf; pop {0}", out(reg) rflags);
            rflags &= !0x100;
            core::arch::asm!("push {0}; popf", in(reg) rflags);
        }
    }
    
    /// Capture call stack
    pub fn capture_call_stack(&mut self, rbp: u64) -> usize {
        let mut stack = self.call_stack.lock();
        stack.clear();
        
        let mut current_rbp = rbp;
        let max_depth = 64;
        
        for _ in 0..max_depth {
            if current_rbp == 0 {
                break;
            }
            
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let rip = *(current_rbp as *const u64).add(1);
                if rip == 0 {
                    break;
                }
                
                let entry = CallStackEntry {
                    addr: rip,
                    symbol: [0; 64],
                    offset: 0,
                    module: [0; 32],
                };
                stack.push(entry);
                
                current_rbp = *(current_rbp as *const u64);
            }
        }
        
        stack.len()
    }
    
    /// Handle breakpoint
    pub fn handle_breakpoint(&mut self, rip: u64) {
        if !self.enabled.load(Ordering::Acquire) {
            return;
        }
        
        log_debug!("Breakpoint hit at {:#x}", rip);
        
        // Check hardware breakpoints
        for i in 0..4 {
            if self.hw_bps[i].enabled && self.hw_bps[i].addr == rip {
                self.hw_bps[i].hit_count.fetch_add(1, Ordering::AcqRel);
                log_debug!("Hardware breakpoint {} hit (count: {})", 
                    i, self.hw_bps[i].hit_count.load(Ordering::Acquire));
            }
        }
        
        // Check software breakpoints
        let sw_bps = self.sw_bps.lock();
        if sw_bps.contains_key(&(rip - 1)) {
            log_debug!("Software breakpoint hit at {:#x}", rip - 1);
        }
    }
    
    /// Handle debug exception
    pub fn handle_debug_exception(&mut self, dr6: u64, rip: u64) {
        if !self.enabled.load(Ordering::Acquire) {
            return;
        }
        
        // Check which breakpoint triggered
        if dr6 & dr6_bits::B0 != 0 {
            self.hw_bps[0].hit_count.fetch_add(1, Ordering::AcqRel);
            log_debug!("Debug: BP0 at {:#x}", rip);
        }
        if dr6 & dr6_bits::B1 != 0 {
            self.hw_bps[1].hit_count.fetch_add(1, Ordering::AcqRel);
            log_debug!("Debug: BP1 at {:#x}", rip);
        }
        if dr6 & dr6_bits::B2 != 0 {
            self.hw_bps[2].hit_count.fetch_add(1, Ordering::AcqRel);
            log_debug!("Debug: BP2 at {:#x}", rip);
        }
        if dr6 & dr6_bits::B3 != 0 {
            self.hw_bps[3].hit_count.fetch_add(1, Ordering::AcqRel);
            log_debug!("Debug: BP3 at {:#x}", rip);
        }
        if dr6 & dr6_bits::BS != 0 {
            log_debug!("Debug: Single step at {:#x}", rip);
        }
        
        // Clear DR6
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!("mov dr6, {0}", in(reg) 0u64);
        }
    }
    
    /// Print call stack
    pub fn print_call_stack(&self) {
        let stack = self.call_stack.lock();
        log_debug!("Call stack:");
        for (i, entry) in stack.iter().enumerate() {
            log_debug!("  #{}: {:#x}", i, entry.addr);
        }
    }
}

impl Default for KernelDebugger {
    fn default() -> Self { Self::new() }
}

/// Global kernel debugger
static mut KERNEL_DEBUGGER: core::mem::MaybeUninit<KernelDebugger> = core::mem::MaybeUninit::uninit();

/// Get kernel debugger
pub fn get_debugger() -> &'static mut KernelDebugger {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { KERNEL_DEBUGGER.assume_init_mut() }
}

/// Initialize kernel debugger
pub fn init_kdebug() {
    // SAFETY: KERNEL_DEBUGGER is only written here during init
    unsafe { KERNEL_DEBUGGER.write(KernelDebugger::new()); }
    let dbg = get_debugger();
    log_info!("Kernel debugger initialized");
}
