/*
 * Nuva OS - Kernel - ARM64 Context Switch
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

use core::arch::asm;
use core::arch::naked_asm;
use crate::{pr_info, pr_warn};

/// CPU Context Structure
/// Contains all registers that need to be saved/restored during context switch.
/// This structure must match the assembly code expectations.
#[repr(C)]
pub struct CpuContext {
    /// General purpose registers x19-x28 (callee-saved)
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,

    /// Frame pointer (x29)
    pub fp: u64,
    /// Link register (x30)
    pub lr: u64,
    /// Stack pointer
    pub sp: u64,
    /// Program counter
    pub pc: u64,
}

impl CpuContext {
    /// Create a new zeroed CPU context
    pub const fn new() -> Self {
        CpuContext {
            x19: 0, x20: 0, x21: 0, x22: 0,
            x23: 0, x24: 0, x25: 0, x26: 0,
            x27: 0, x28: 0,
            fp: 0, lr: 0, sp: 0, pc: 0,
        }
    }

    /// Initialize context for a new task
    /// @param entry: Entry point address
    /// @param stack_top: Top of stack
    /// @param arg: First argument (x0)
    pub fn init_for_task(&mut self, entry: u64, stack_top: u64, arg: u64) {
        self.pc = entry;
        self.sp = stack_top;
        self.lr = entry;  /* Return to entry if function returns */
        self.fp = stack_top;

        // Store arg in x19, will be moved to x0 in entry
        self.x19 = arg;
    }
}

/// Thread Info Structure
/// Contains thread-local information and CPU context.
#[repr(C)]
pub struct ThreadInfo {
    /// CPU context for context switch
    pub cpu_context: CpuContext,
    /// Thread flags
    pub flags: u32,
    /// Preempt count
    pub preempt_count: u32,
    /// Thread local storage
    pub tls_base: u64,
    /// Kernel stack top
    pub kstack_top: u64,
    /// Kernel stack bottom
    pub kstack_bottom: u64,
    /// Process ID
    pub pid: u32,
    /// Thread ID
    pub tid: u32,
}

impl ThreadInfo {
    /// Create a new thread info
    pub const fn new() -> Self {
        ThreadInfo {
            cpu_context: CpuContext::new(),
            flags: 0,
            preempt_count: 0,
            tls_base: 0,
            kstack_top: 0,
            kstack_bottom: 0,
            pid: 0,
            tid: 0,
        }
    }

    /// Initialize thread info for a new task
    /// @param entry: Entry point
    /// @param kstack: Kernel stack pointer
    /// @param kstack_size: Kernel stack size
    /// @param arg: First argument
    /// @param pid: Process ID
    /// @param tid: Thread ID
    pub fn init(&mut self, entry: u64, kstack: u64, kstack_size: usize, arg: u64, pid: u32, tid: u32) {
        self.kstack_bottom = kstack;
        self.kstack_top = kstack + kstack_size as u64;
        self.pid = pid;
        self.tid = tid;
        self.cpu_context.init_for_task(entry, self.kstack_top, arg);
    }
}

/// Perform context switch
/// Saves current context and switches to new context.
/// This is the core of task switching.
/// @param prev: Pointer to previous thread's ThreadInfo
/// @param next: Pointer to next thread's ThreadInfo
/// Assembly equivalent:
/// ```asm
/// switch_to:
/// // Save callee-saved registers
/// stp x19, x20, [x0, #0]
/// stp x21, x22, [x0, #16]
/// stp x23, x24, [x0, #32]
/// stp x25, x26, [x0, #48]
/// stp x27, x28, [x0, #64]
/// stp x29, x30, [x0, #80]
/// mov x19, sp
/// str x19, [x0, #96]
/// // Restore callee-saved registers
/// ldp x19, x20, [x1, #0]
/// ldp x21, x22, [x1, #16]
/// ldp x23, x24, [x1, #32]
/// ldp x25, x26, [x1, #48]
/// ldp x27, x28, [x1, #64]
/// ldp x29, x30, [x1, #80]
/// ldr x19, [x1, #96]
/// mov sp, x19
/// ret
/// ```
#[unsafe(naked)]
pub unsafe extern "C" fn switch_to(prev: *mut ThreadInfo, next: *mut ThreadInfo) {
    naked_asm!(
        // Save callee-saved registers of prev task
        "stp x19, x20, [x0, #0]",      /* offset 0: x19, x20 */
        "stp x21, x22, [x0, #16]",     /* offset 16: x21, x22 */
        "stp x23, x24, [x0, #32]",     /* offset 32: x23, x24 */
        "stp x25, x26, [x0, #48]",     /* offset 48: x25, x26 */
        "stp x27, x28, [x0, #64]",     /* offset 64: x27, x28 */
        "stp x29, x30, [x0, #80]",     /* offset 80: fp, lr */
        "mov x19, sp",
        "str x19, [x0, #96]",          /* offset 96: sp */

        // Restore callee-saved registers of next task
        "ldp x19, x20, [x1, #0]",
        "ldp x21, x22, [x1, #16]",
        "ldp x23, x24, [x1, #32]",
        "ldp x25, x26, [x1, #48]",
        "ldp x27, x28, [x1, #64]",
        "ldp x29, x30, [x1, #80]",
        "ldr x19, [x1, #96]",
        "mov sp, x19",

        // Return to new task
        "ret",
    );
}

/// Context switch wrapper
/// High-level wrapper for context switch with additional processing.
/// @param prev: Previous thread info
/// @param next: Next thread info
pub fn context_switch(prev: *mut ThreadInfo, next: *mut ThreadInfo) {
    if prev.is_null() || next.is_null() {
        log_warn!("context_switch: null thread info");
        return;
    }

    if prev == next {
        // No need to switch to same task
        return;
    }

    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        // Perform the actual context switch
        switch_to(prev, next);
    }
}

/// Task entry wrapper
/// Called when a new task starts execution.
/// Sets up the initial argument and calls the task function.
/// @param func: Task function pointer
/// @param arg: Task argument
#[unsafe(naked)]
pub unsafe extern "C" fn task_entry_wrapper() {
    naked_asm!(
        // x19 contains arg (set by init_for_task)
        // x30 (lr) contains entry point
        "mov x0, x19",     /* Move arg to x0 */
        "blr x30",         /* Call the task function */

        // Task returned, call exit
        "mov x0, x0",      /* Return value in x0 */
        "b task_exit",     /* Jump to task exit */
    );
}

/// Task exit handler
/// Called when a task returns from its entry function.
/// @param ret_val: Return value from task
pub extern "C" fn task_exit(ret_val: u64) {
    log_info!("Task exited with return value: {}", ret_val);

    // TODO:
    // 1. Mark task as zombie
    // 2. Wake up parent if waiting
    // 3. Schedule next task

    // Loop forever - should never reach here after proper implementation
    loop {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            asm!("wfi");
        }
    }
}

/// Get current thread info
/// Returns the ThreadInfo for the current task.
/// Uses SP to calculate the thread info address.
/// @return Pointer to current ThreadInfo
#[inline(always)]
pub fn current_thread_info() -> *mut ThreadInfo {
    let sp: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mov {}, sp",
            out(reg) sp,
        );
    }

    // Thread info is at the bottom of the kernel stack
    // Align SP to 8KB (two pages) and get the base
    let thread_info_addr = sp & !0x1FFF;

    thread_info_addr as *mut ThreadInfo
}

/// Get current process ID
/// @return Current process ID
#[inline(always)]
pub fn current_pid() -> u32 {
    let ti = current_thread_info();
    if ti.is_null() {
        return 0;
    }
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { (*ti).pid }
}

/// Get current thread ID
/// @return Current thread ID
#[inline(always)]
pub fn current_tid() -> u32 {
    let ti = current_thread_info();
    if ti.is_null() {
        return 0;
    }
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { (*ti).tid }
}

/// Set thread flag
/// @param flag: Flag to set
#[inline(always)]
pub fn set_thread_flag(flag: u32) {
    let ti = current_thread_info();
    if !ti.is_null() {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*ti).flags |= flag;
        }
    }
}

/// Clear thread flag
/// @param flag: Flag to clear
#[inline(always)]
pub fn clear_thread_flag(flag: u32) {
    let ti = current_thread_info();
    if !ti.is_null() {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*ti).flags &= !flag;
        }
    }
}

/// Test thread flag
/// @param flag: Flag to test
/// @return true if flag is set
#[inline(always)]
pub fn test_thread_flag(flag: u32) -> bool {
    let ti = current_thread_info();
    if ti.is_null() {
        return false;
    }
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        (*ti).flags & flag != 0
    }
}

// Thread flags
pub const TIF_NEED_RESCHED: u32 = 1 << 0;    /* Rescheduling needed */
pub const TIF_SIGPENDING: u32 = 1 << 1;      /* Signal pending */
pub const TIF_NOTIFY_RESUME: u32 = 1 << 2;   /* Callback before return to user */
pub const TIF_UPROBE: u32 = 1 << 3;          /* Uprobe breakpoint */
pub const TIF_SYSCALL_TRACE: u32 = 1 << 4;   /* Syscall trace active */

/// Check if rescheduling is needed
/// @return true if rescheduling is needed
#[inline(always)]
pub fn need_resched() -> bool {
    test_thread_flag(TIF_NEED_RESCHED)
}

/// Set need reschedule flag
#[inline(always)]
pub fn set_need_resched() {
    set_thread_flag(TIF_NEED_RESCHED);
}

/// Clear need reschedule flag
#[inline(always)]
pub fn clear_need_resched() {
    clear_thread_flag(TIF_NEED_RESCHED);
}

/// Preempt count operations
impl ThreadInfo {
    /// Increment preempt count
    #[inline(always)]
    pub fn preempt_disable(&mut self) {
        self.preempt_count += 1;
    }

    /// Decrement preempt count
    #[inline(always)]
    pub fn preempt_enable(&mut self) {
        if self.preempt_count > 0 {
            self.preempt_count -= 1;
        }
    }

    /// Check if preemption is disabled
    #[inline(always)]
    pub fn is_preempt_disabled(&self) -> bool {
        self.preempt_count != 0
    }
}

/// Disable preemption
#[inline(always)]
pub fn preempt_disable() {
    let ti = current_thread_info();
    if !ti.is_null() {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*ti).preempt_disable();
        }
    }
}

/// Enable preemption
#[inline(always)]
pub fn preempt_enable() {
    let ti = current_thread_info();
    if !ti.is_null() {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*ti).preempt_enable();
        }
    }
}

/// Check if preemption is disabled
/// @return true if preemption is disabled
#[inline(always)]
pub fn is_preempt_disabled() -> bool {
    let ti = current_thread_info();
    if ti.is_null() {
        return true;  /* Safe default */
    }
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        (*ti).is_preempt_disabled()
    }
}

/// Initialize context switch subsystem
pub fn init_context() {
    log_info!("Context switch initialized");
    log_info!("  ThreadInfo size: {} bytes", core::mem::size_of::<ThreadInfo>());
    log_info!("  CpuContext size: {} bytes", core::mem::size_of::<CpuContext>());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_context_new() {
        let ctx = CpuContext::new();
        assert_eq!(ctx.x19, 0);
        assert_eq!(ctx.sp, 0);
        assert_eq!(ctx.pc, 0);
    }
}
