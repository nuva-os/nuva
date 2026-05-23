/*
 * Nuva OS - Kernel - Architecture Abstraction Layer
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

//! Architecture Abstraction Layer
/*!*/
//! Provides multi-architecture support with plugin-based management,
//! dynamically loading architecture plugins based on different devices

// Re-export print macros from crate root
pub use crate::{pr_emerg, pr_alert, pr_crit, pr_err, pr_warn, pr_notice, pr_info, pr_debug};
pub mod platform;

// Architecture plugin system
pub mod plugins;

#[cfg(target_arch = "aarch64")]
pub mod arm64;

#[cfg(target_arch = "x86_64")]
pub mod x64;

#[cfg(target_arch = "loongarch64")]
pub mod loongarch64;

use core::fmt;

/// Physical Address
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PhysAddr(pub u64);

impl PhysAddr {
    pub const fn new(addr: u64) -> Self {
        PhysAddr(addr)
    }
    
    pub const fn zero() -> Self {
        PhysAddr(0)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
    
    pub fn is_aligned(&self, align: u64) -> bool {
        self.0 % align == 0
    }
    
    pub fn align_up(&self, align: u64) -> Self {
        PhysAddr((self.0 + align - 1) & !(align - 1))
    }
    
    pub fn align_down(&self, align: u64) -> Self {
        PhysAddr(self.0 & !(align - 1))
    }
}

impl fmt::Display for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PhysAddr({:#x})", self.0)
    }
}

/// Virtual Address
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct VirtAddr(pub u64);

impl VirtAddr {
    pub const fn new(addr: u64) -> Self {
        VirtAddr(addr)
    }
    
    pub const fn zero() -> Self {
        VirtAddr(0)
    }
    
    pub fn as_u64(&self) -> u64 {
        self.0
    }
    
    pub fn as_mut_ptr<T>(&self) -> *mut T {
        self.0 as *mut T
    }
    
    pub fn as_ptr<T>(&self) -> *const T {
        self.0 as *const T
    }
    
    pub fn is_aligned(&self, align: u64) -> bool {
        self.0 % align == 0
    }
    
    pub fn align_up(&self, align: u64) -> Self {
        VirtAddr((self.0 + align - 1) & !(align - 1))
    }
    
    pub fn align_down(&self, align: u64) -> Self {
        VirtAddr(self.0 & !(align - 1))
    }
    
    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
}

impl fmt::Display for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VirtAddr({:#x})", self.0)
    }
}

/// Page TablePermissionFlag
#[derive(Debug, Clone, Copy)]
pub struct ProtFlags(pub u32);

impl ProtFlags {
    pub const NONE: ProtFlags = ProtFlags(0);
    pub const READ: ProtFlags = ProtFlags(1 << 0);
    pub const WRITE: ProtFlags = ProtFlags(1 << 1);
    pub const EXEC: ProtFlags = ProtFlags(1 << 2);
    pub const USER: ProtFlags = ProtFlags(1 << 3);
    pub const RW: ProtFlags = ProtFlags(ProtFlags::READ.0 | ProtFlags::WRITE.0);
    pub const RX: ProtFlags = ProtFlags(ProtFlags::READ.0 | ProtFlags::EXEC.0);
    pub const RWX: ProtFlags = ProtFlags(ProtFlags::READ.0 | ProtFlags::WRITE.0 | ProtFlags::EXEC.0);
    
    pub fn contains(&self, flag: ProtFlags) -> bool {
        (self.0 & flag.0) != 0
    }
    
    pub fn is_readable(&self) -> bool {
        self.contains(ProtFlags::READ)
    }
    
    pub fn is_writable(&self) -> bool {
        self.contains(ProtFlags::WRITE)
    }
    
    pub fn is_executable(&self) -> bool {
        self.contains(ProtFlags::EXEC)
    }
    
    pub fn is_user(&self) -> bool {
        self.contains(ProtFlags::USER)
    }
}

/// Page TableOperationInterface
pub trait PageTableOps {
/// Create a new page table
    fn create(&self) -> PhysAddr;
    
    /// DestroyPage Table
    fn destroy(&self, pgd: PhysAddr);
    
    /// Map a page
    fn map(&self, pgd: PhysAddr, vaddr: VirtAddr, paddr: PhysAddr, prot: ProtFlags, page_size: u64);
    
    /// Unmap a page
    fn unmap(&self, pgd: PhysAddr, vaddr: VirtAddr);
    
    /// Translate virtual address to physical address
    fn translate(&self, pgd: PhysAddr, vaddr: VirtAddr) -> Option<PhysAddr>;
    
    /// Modify page permissions
    fn protect(&self, pgd: PhysAddr, vaddr: VirtAddr, prot: ProtFlags);
    
    /// Refresh TLB for a single address
    fn tlb_flush_addr(&self, vaddr: VirtAddr);
    
    /// Refresh entire TLB
    fn tlb_flush_all(&self);
    
    /// SwitchPage Table
    fn switch(&self, pgd: PhysAddr);
    
    /// GetCurrentPage Table
    fn current(&self) -> PhysAddr;
}

/// Interrupt Controller Operation Interface
pub trait IrqControllerOps {
    /// Initialize interrupt controller
    fn init(&self);
    
    /// Allocate interrupt number
    fn alloc_irq(&self) -> Option<u32>;
    
    /// Free interrupt number
    fn free_irq(&self, irq: u32);
    
    /// RegisterInterruptHandleFunction
    fn register_handler(&self, irq: u32, handler: fn(u32), flags: u32) -> bool;
    
    /// UnregisterInterruptHandleFunction
    fn unregister_handler(&self, irq: u32);
    
    /// Enable interrupt
    fn enable_irq(&self, irq: u32);
    
    /// DisableInterrupt
    fn disable_irq(&self, irq: u32);
    
    /// Interrupt acknowledgment (EOI)
    fn eoi(&self, irq: u32);
    
    /// Set interrupt affinity
    fn set_affinity(&self, irq: u32, cpu_mask: u64);
    
    /// GetInterruptCount
    fn get_irq_count(&self, irq: u32) -> u64;
}

/// TimerOperationInterface
pub trait TimerOps {
    /// InitializeTimer
    fn init(&self);
    
    /// Get current time (in nanoseconds)
    fn now(&self) -> u64;
    
    /// Set one-shot timer
    fn set_oneshot(&self, ns: u64);
    
    /// Set periodic timer
    fn set_periodic(&self, ns: u64);
    
    /// StopTimer
    fn stop(&self);
    
    /// GetTimerFrequency
    fn frequency(&self) -> u64;
    
    /// Busy wait for specified nanoseconds
    fn delay(&self, ns: u64);
}

/// Power Management Operation Interface
pub trait PowerOps {
    /// Initialize power management
    fn init(&self);
    
    /// CPU idle
    fn cpu_idle(&self);
    
    /// CPU sleep
    fn cpu_sleep(&self);
    
    /// CPU Wake
    fn cpu_wakeup(&self, cpu_id: u32);
    
    /// System shutdown
    fn system_shutdown(&self);
    
    /// System reboot
    fn system_reboot(&self);
    
    /// System suspend
    fn system_suspend(&self);
}

/// CPU Context - full architectural state for context switching.
/// This structure contains all registers that must be saved/restored
/// during a context switch, including FPU/SIMD state.
#[derive(Debug, Clone, Copy)]
pub struct CpuContext {
    /// General-purpose registers (ARM64: x0-x30, x86_64: rax-r15).
    pub regs: [u64; 31],
    /// Stack pointer.
    pub sp: u64,
    /// Program counter / instruction pointer.
    pub pc: u64,
    /// Processor state (ARM64: pstate, x86_64: rflags).
    pub pstate: u64,
    /// FPU/SIMD registers stored as u64 pairs (128-bit each, 32 registers = 64 u64s).
    /// ARM64: FPSIMD V0-V31 (32 x 128-bit).
    /// x86_64: XMM0-XMM15 (16 x 128-bit, upper half unused).
    pub fpsimd: [u64; 64],
    /// FPU control register (ARM64: FPCR, x86_64: MXCSR).
    pub fpcr: u64,
    /// FPU status register (ARM64: FPSR, x86_64: unused).
    pub fpsr: u64,
    /// Thread-local storage pointer (ARM64: tpidr_el0, x86_64: fs_base).
    pub tls_base: u64,
    /// TLS read-only pointer (ARM64: tpidrro_el0, x86_64: gs_base).
    pub tls_base_ro: u64,
}

impl CpuContext {
    /// Create a new zeroed CPU context.
    pub fn new() -> Self {
        CpuContext {
            regs: [0; 31],
            sp: 0,
            pc: 0,
            pstate: 0,
            fpsimd: [0; 64],
            fpcr: 0,
            fpsr: 0,
            tls_base: 0,
            tls_base_ro: 0,
        }
    }

    /// Create a new CPU context for a new task with the given entry point and stack.
    pub fn new_task(entry: u64, stack_top: u64, arg0: u64) -> Self {
        let mut ctx = CpuContext::new();
        ctx.pc = entry;
        ctx.sp = stack_top;
        ctx.regs[0] = arg0; // First argument (x0 on ARM64, rdi on x86_64)

        // Set up initial processor state
        #[cfg(target_arch = "aarch64")]
        {
            // EL1h, IRQ/FIQ unmasked
            ctx.pstate = 0x5; // DAIF cleared, EL1h
        }
        #[cfg(target_arch = "x86_64")]
        {
            // Interrupts enabled, reserved bit 1 set
            ctx.pstate = 0x202; // RFLAGS with IF set
        }

        ctx
    }

    /// Get the stack pointer.
    pub fn get_sp(&self) -> u64 {
        self.sp
    }

    /// Get the program counter.
    pub fn get_pc(&self) -> u64 {
        self.pc
    }

    /// Set the program counter (for signal handler injection).
    pub fn set_pc(&mut self, pc: u64) {
        self.pc = pc;
    }

    /// Get the return value from the context (x0 on ARM64, rax on x86_64).
    pub fn get_return_value(&self) -> u64 {
        self.regs[0]
    }

    /// Set the return value in the context.
    pub fn set_return_value(&mut self, val: u64) {
        self.regs[0] = val;
    }
}

/// ContextSwitchOperationInterface
pub trait ContextOps {
    /// SaveCurrentContext
    fn save_context(&self, ctx: &mut CpuContext);
    
    /// RecoveryContext
    fn restore_context(&self, ctx: &CpuContext);
    
    /// SwitchContext
    fn switch_context(&self, from: &mut CpuContext, to: &CpuContext);
}

/// Architecture abstraction trait
pub trait ArchOps {
    /// InitializeArchitecture
    fn init(&self);
    
    /// Page TableOperation
    fn page_table(&self) -> &'static dyn PageTableOps;
    
    /// Interrupt controller
    fn irq_controller(&self) -> &'static dyn IrqControllerOps;
    
    /// Timer
    fn timer(&self) -> &'static dyn TimerOps;
    
    /// Power management
    fn power(&self) -> &'static dyn PowerOps;
    
    /// ContextOperation
    fn context(&self) -> &'static dyn ContextOps;
    
    /// Enable interrupt
    fn enable_irq(&self);
    
    /// DisableInterrupt
    fn disable_irq(&self);
    
    /// Get CPU ID
    fn cpu_id(&self) -> u32;
    
    /// Get CPU count
    fn cpu_count(&self) -> u32;
}

/// GetCurrentArchitectureImplementation
// TODO: ArchOps is not dyn compatible; these will fail at compile time
#[cfg(target_arch = "aarch64")]
pub fn current_arch() -> &'static dyn ArchOps {
    &arm64::ARM64_ARCH
}

#[cfg(target_arch = "x86_64")]
pub fn current_arch() -> &'static dyn ArchOps {
    &x64::X64_ARCH
}

#[cfg(target_arch = "loongarch64")]
pub fn current_arch() -> &'static dyn ArchOps {
    &loongarch64::LOONGARCH64_ARCH
}

/// InitializeArchitecture
pub fn init_arch() {
    current_arch().init();
}

/// Signal return trampoline address.
/// This is the address of the sigreturn system call stub that signal
/// handlers return to. It restores the original user context from the
/// signal frame on the user stack.
/// On ARM64, this is a small code stub that executes SVC #0 (sigreturn).
/// On x86_64, this is a small code stub that executes syscall (sigreturn).
#[cfg(target_arch = "aarch64")]
pub const SIGRETURN_TRAMPOLINE: usize = 0xFFFF0000; // VDSO sigreturn stub address

#[cfg(target_arch = "x86_64")]
pub const SIGRETURN_TRAMPOLINE: usize = 0xFFFF800000000000; // VDSO sigreturn stub address

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
pub const SIGRETURN_TRAMPOLINE: usize = 0;