/*
* Nuva OS - Kernel - LoongArch64 Context Switch
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

use crate::{pr_info, pr_warn};
use core::arch::asm;
use core::arch::naked_asm;

/// CPU Context Structure
/// Contains all registers that need to be saved/restored during context switch.
/// LoongArch64 callee-saved registers: $s0-$s7 ($r8,$r9,$r10,$r11,$r12,$r13,$r14,$r15),
/// $fp ($r22), $ra ($r1)
#[repr(C)]
pub struct CpuContext {
    /// Callee-saved registers s0-s7 (r8-r15)
    pub s0: u64,
    pub s1: u64,
    pub s2: u64,
    pub s3: u64,
    pub s4: u64,
    pub s5: u64,
    pub s6: u64,
    pub s7: u64,
    /// Frame pointer (r22)
    pub fp: u64,
    /// Return address (r1)
    pub ra: u64,
    /// Stack pointer
    pub sp: u64,
    /// Program counter
    pub pc: u64,
}

impl CpuContext {
    /// Create a new zeroed CPU context
    pub const fn new() -> Self {
        CpuContext {
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            fp: 0,
            ra: 0,
            sp: 0,
            pc: 0,
        }
    }

    /// Initialize context for a new task
    /// @param entry: Entry point address
    /// @param stack_top: Top of stack
    /// @param arg: First argument ($a0 = $r4)
    pub fn init_for_task(&mut self, entry: u64, stack_top: u64, arg: u64) {
        self.pc = entry;
        self.sp = stack_top;
        self.ra = entry;
        self.fp = stack_top;
        self.s0 = arg;
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
    pub fn init(
        &mut self,
        entry: u64,
        kstack: u64,
        kstack_size: usize,
        arg: u64,
        pid: u32,
        tid: u32,
    ) {
        self.kstack_bottom = kstack;
        self.kstack_top = kstack + kstack_size as u64;
        self.pid = pid;
        self.tid = tid;
        self.cpu_context.init_for_task(entry, self.kstack_top, arg);
    }
}

/// Perform context switch
/// Saves current context and switches to new context.
/// @param prev: Pointer to previous thread's ThreadInfo ($a0)
/// @param next: Pointer to next thread's ThreadInfo ($a1)
#[unsafe(naked)]
pub unsafe extern "C" fn switch_to(prev: *mut ThreadInfo, next: *mut ThreadInfo) {
    naked_asm!(
        // Save callee-saved registers of prev task
        "st.d $s0, $a0, 0",
        "st.d $s1, $a0, 8",
        "st.d $s2, $a0, 16",
        "st.d $s3, $a0, 24",
        "st.d $s4, $a0, 32",
        "st.d $s5, $a0, 40",
        "st.d $s6, $a0, 48",
        "st.d $s7, $a0, 56",
        "st.d $fp, $a0, 64",
        "st.d $ra, $a0, 72",
        "st.d $sp, $a0, 80",
        // Restore callee-saved registers of next task
        "ld.d $s0, $a1, 0",
        "ld.d $s1, $a1, 8",
        "ld.d $s2, $a1, 16",
        "ld.d $s3, $a1, 24",
        "ld.d $s4, $a1, 32",
        "ld.d $s5, $a1, 40",
        "ld.d $s6, $a1, 48",
        "ld.d $s7, $a1, 56",
        "ld.d $fp, $a1, 64",
        "ld.d $ra, $a1, 72",
        "ld.d $sp, $a1, 80",
        // Return to new task
        "jirl $zero, $ra, 0",
    );
}

/// Context switch wrapper
/// @param prev: Previous thread info
/// @param next: Next thread info
pub fn context_switch(prev: *mut ThreadInfo, next: *mut ThreadInfo) {
    if prev.is_null() || next.is_null() {
        log_warn!("context_switch: null thread info");
        return;
    }

    if prev == next {
        return;
    }

    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        switch_to(prev, next);
    }
}

/// Task entry wrapper
/// Called when a new task starts execution.
#[unsafe(naked)]
pub unsafe extern "C" fn task_entry_wrapper() {
    naked_asm!(
        // s0 contains arg (set by init_for_task)
        "move $a0, $s0",
        "jirl $zero, $ra, 0",
        // Task returned, call exit
        "move $a0, $a0",
        "b task_exit",
    );
}

/// Task exit handler
/// Called when a task returns from its entry function.
/// @param ret_val: Return value from task
pub extern "C" fn task_exit(ret_val: u64) {
    log_info!("Task exited with return value: {}", ret_val);

    loop {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            asm!("idle 0");
        }
    }
}

/// Get current thread info
/// @return Pointer to current ThreadInfo
#[inline(always)]
pub fn current_thread_info() -> *mut ThreadInfo {
    let sp: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "move {}, $sp",
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
    unsafe { (*ti).flags & flag != 0 }
}

// Thread flags
pub const TIF_NEED_RESCHED: u32 = 1 << 0;
pub const TIF_SIGPENDING: u32 = 1 << 1;
pub const TIF_NOTIFY_RESUME: u32 = 1 << 2;
pub const TIF_UPROBE: u32 = 1 << 3;
pub const TIF_SYSCALL_TRACE: u32 = 1 << 4;

/// Check if rescheduling is needed
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
#[inline(always)]
pub fn is_preempt_disabled() -> bool {
    let ti = current_thread_info();
    if ti.is_null() {
        return true;
    }
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { (*ti).is_preempt_disabled() }
}

/// Initialize context switch subsystem
pub fn init_context() {
    log_info!("Context switch initialized");
    log_info!(
        "  ThreadInfo size: {} bytes",
        core::mem::size_of::<ThreadInfo>()
    );
    log_info!(
        "  CpuContext size: {} bytes",
        core::mem::size_of::<CpuContext>()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_context_new() {
        let ctx = CpuContext::new();
        assert_eq!(ctx.s0, 0);
        assert_eq!(ctx.sp, 0);
        assert_eq!(ctx.pc, 0);
    }
}
