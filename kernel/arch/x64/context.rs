/*
 * Nuva OS - Kernel - x86-64 Context Switch
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
    /// Callee-saved registers rbx, rbp, r12-r15
    pub rbx: u64,
    pub rbp: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    /// Stack pointer
    pub rsp: u64,
    /// Program counter (rip)
    pub rip: u64,
}

impl CpuContext {
    /// Create a new zeroed CPU context
    pub const fn new() -> Self {
        CpuContext {
            rbx: 0, rbp: 0,
            r12: 0, r13: 0, r14: 0, r15: 0,
            rsp: 0, rip: 0,
        }
    }

    /// Initialize context for a new task
    /// @param entry: Entry point address
    /// @param stack_top: Top of stack
    /// @param arg: First argument (rdi)
    pub fn init_for_task(&mut self, entry: u64, stack_top: u64, arg: u64) {
        self.rip = entry;
        self.rsp = stack_top;
        self.rbx = arg;
        self.rbp = stack_top;
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
///   // Save callee-saved registers
///   mov [prev+0], rbx
///   mov [prev+8], rbp
///   mov [prev+16], r12
///   mov [prev+24], r13
///   mov [prev+32], r14
///   mov [prev+40], r15
///   mov [prev+48], rsp
///   // Restore callee-saved registers
///   mov rbx, [next+0]
///   mov rbp, [next+8]
///   mov r12, [next+16]
///   mov r13, [next+24]
///   mov r14, [next+32]
///   mov r15, [next+40]
///   mov rsp, [next+48]
///   ret
/// ```
#[unsafe(naked)]
pub unsafe extern "C" fn switch_to(prev: *mut ThreadInfo, next: *mut ThreadInfo) {
    naked_asm!(
        // Save callee-saved registers of prev task
        "mov [rdi + 0], rbx",
        "mov [rdi + 8], rbp",
        "mov [rdi + 16], r12",
        "mov [rdi + 24], r13",
        "mov [rdi + 32], r14",
        "mov [rdi + 40], r15",
        "mov [rdi + 48], rsp",

        // Restore callee-saved registers of next task
        "mov rbx, [rsi + 0]",
        "mov rbp, [rsi + 8]",
        "mov r12, [rsi + 16]",
        "mov r13, [rsi + 24]",
        "mov r14, [rsi + 32]",
        "mov r15, [rsi + 40]",
        "mov rsp, [rsi + 48]",

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
        return;
    }

    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        switch_to(prev, next);
    }
}

/// Task entry wrapper
/// Called when a new task starts execution.
/// Sets up the initial argument and calls the task function.
#[unsafe(naked)]
pub unsafe extern "C" fn task_entry_wrapper() {
    naked_asm!(
        // rbx contains arg (set by init_for_task)
        // r15 contains entry point (set via rip, stored in r15 for task entry)
        "mov rdi, rbx",
        "call r15",

        // Task returned, call exit
        "mov rdi, rax",
        "jmp task_exit",
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

    loop {
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            asm!("hlt");
        }
    }
}

/// Get current thread info
/// Returns the ThreadInfo for the current task.
/// Uses RSP to calculate the thread info address.
/// @return Pointer to current ThreadInfo
#[inline(always)]
pub fn current_thread_info() -> *mut ThreadInfo {
    let rsp: u64;
    // SAFETY: inline assembly required for hardware instruction
    unsafe {
        asm!(
            "mov {}, rsp",
            out(reg) rsp,
        );
    }

    // Thread info is at the bottom of the kernel stack
    // Align RSP to 8KB (two pages) and get the base
    let thread_info_addr = rsp & !0x1FFF;

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
        return true;
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
        assert_eq!(ctx.rbx, 0);
        assert_eq!(ctx.rsp, 0);
        assert_eq!(ctx.rip, 0);
    }
}
