/*
 * Nuva OS - Kernel - Core - Privilege
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
/*
 * Nuva OS - Kernel - Three-Level Privilege Definition
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * NvPrivilegeLevel core definition and architecture privilege mapping.
 *
 * EL2: Minimal kernel mode (scheduler, IPC, MM, cap mgr, IRQ, timer)
 * EL1: Equipment mode (filesystem, network, drivers, display services)
 * EL0: User mode (applications)
 *
 * INVARIANT: System always maintains three distinct privilege levels.
 * INVARIANT: Process privilege_level cannot be modified cross-level.
 * INVARIANT: EL2 components are fixed: {NvIPC, Scheduler, MM, CapMgr, IRQ, Timer}.
 */

use core::fmt;
use crate::kernel::types::{NvPrivilegeLevel, NvSupervisorOp, NuvaCapabilityId};

/// Trap context saved when entering from lower privilege level
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct NvTrapContext {
    /// Source privilege level
    pub from_level: NvPrivilegeLevel,
    /// Saved program counter
    pub pc: u64,
    /// Saved status register (SPSR/CR0/Status)
    pub status: u64,
    /// Saved general-purpose registers (x0-x30 / rax-r15)
    pub regs: [u64; 31],
    /// Saved stack pointer
    pub sp: u64,
    /// Trap reason (syscall/irq/fault/supervisor_call)
    pub trap_reason: u64,
}

/// Supervisor call context for EL1→EL2 entry
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct NvSupervisorContext {
    /// Caller capability token
    pub caller_cap: NuvaCapabilityId,
    /// Supervisor operation
    pub operation: NvSupervisorOp,
    /// Operation arguments (up to 6, matching syscall ABI)
    pub args: [u64; 6],
    /// Caller privilege level (must be EquipmentMode)
    pub caller_level: NvPrivilegeLevel,
}

/// NvSupervisorCall result
#[derive(Debug, Clone, Copy)]
pub struct NvSupervisorResult {
    /// Return value
    pub value: u64,
    /// Output arguments
    pub output: [u64; 4],
}

/// Architecture-specific privilege mapping trait
///
/// Maps Nuva's three-level privilege model to hardware-specific
/// exception levels / protection rings / privilege levels.
pub trait NvArchPrivilegeMapping {
    /// Map Nuva privilege level to hardware-specific privilege level
    fn hw_privilege_level(level: NvPrivilegeLevel) -> u8;

    /// Enter from lower privilege level (save context, switch to higher)
    fn enter_from_lower(from: NvPrivilegeLevel, trap_ctx: &mut NvTrapContext);

    /// Return to lower privilege level (restore context, switch to lower)
    fn return_to_lower(ctx: &NvTrapContext) -> !;

    /// Entry point for NvSupervisorCall (EL1→EL2)
    fn supervisor_call_entry(sv_ctx: &NvSupervisorContext) -> NvSupervisorResult;

    /// Return from NvSupervisorCall (EL2→EL1)
    fn supervisor_call_return(result: &NvSupervisorResult) -> !;

    /// Get current running privilege level
    fn current_privilege_level() -> NvPrivilegeLevel;

    /// Check if direct memory access between levels is allowed
    ///
    /// INVARIANT: source_level ≠ target_level → always false
    fn is_cross_level_access_allowed(
        source_level: NvPrivilegeLevel,
        target_level: NvPrivilegeLevel,
    ) -> bool {
        source_level == target_level
    }
}

impl fmt::Display for NvPrivilegeLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NvPrivilegeLevel::KernelMode => write!(f, "EL2-KernelMode"),
            NvPrivilegeLevel::EquipmentMode => write!(f, "EL1-EquipmentMode"),
            NvPrivilegeLevel::UserMode => write!(f, "EL0-UserMode"),
        }
    }
}

/// ARM64 privilege mapping
///
/// EL2 → ARM64 EL2 (Hypervisor Exception Level)
/// EL1 → ARM64 EL1 (Kernel Exception Level)
/// EL0 → ARM64 EL0 (User Exception Level)
pub struct Arm64PrivilegeMapping;

impl NvArchPrivilegeMapping for Arm64PrivilegeMapping {
    fn hw_privilege_level(level: NvPrivilegeLevel) -> u8 {
        match level {
            NvPrivilegeLevel::KernelMode => 2,
            NvPrivilegeLevel::EquipmentMode => 1,
            NvPrivilegeLevel::UserMode => 0,
        }
    }

    fn enter_from_lower(from: NvPrivilegeLevel, trap_ctx: &mut NvTrapContext) {
        trap_ctx.from_level = from;
        // ARM64: hardware automatically saves SPSR/ELR on trap
    }

    fn return_to_lower(ctx: &NvTrapContext) -> ! {
        let _ = ctx;
        // ARM64: ERET instruction restores SPSR/ELR and switches EL
        loop {}
    }

    fn supervisor_call_entry(sv_ctx: &NvSupervisorContext) -> NvSupervisorResult {
        let _ = sv_ctx;
        // ARM64: HVC instruction triggers EL1→EL2 supervisor call
        NvSupervisorResult { value: 0, output: [0; 4] }
    }

    fn supervisor_call_return(result: &NvSupervisorResult) -> ! {
        let _ = result;
        // ARM64: ERET from EL2 back to EL1
        loop {}
    }

    fn current_privilege_level() -> NvPrivilegeLevel {
        // Read from CurrentEL register in real implementation
        NvPrivilegeLevel::KernelMode
    }
}

/// x86_64 privilege mapping
///
/// EL2 → Ring 0 (highest privilege)
/// EL1 → Ring 1 (intermediate privilege)
/// EL0 → Ring 3 (lowest privilege)
pub struct X64PrivilegeMapping;

impl NvArchPrivilegeMapping for X64PrivilegeMapping {
    fn hw_privilege_level(level: NvPrivilegeLevel) -> u8 {
        match level {
            NvPrivilegeLevel::KernelMode => 0,
            NvPrivilegeLevel::EquipmentMode => 1,
            NvPrivilegeLevel::UserMode => 3,
        }
    }

    fn enter_from_lower(from: NvPrivilegeLevel, trap_ctx: &mut NvTrapContext) {
        trap_ctx.from_level = from;
    }

    fn return_to_lower(ctx: &NvTrapContext) -> ! {
        let _ = ctx;
        loop {}
    }

    fn supervisor_call_entry(sv_ctx: &NvSupervisorContext) -> NvSupervisorResult {
        let _ = sv_ctx;
        NvSupervisorResult { value: 0, output: [0; 4] }
    }

    fn supervisor_call_return(result: &NvSupervisorResult) -> ! {
        let _ = result;
        loop {}
    }

    fn current_privilege_level() -> NvPrivilegeLevel {
        NvPrivilegeLevel::KernelMode
    }
}

/// RISC-V privilege mapping
///
/// EL2 → Machine Mode (M-mode)
/// EL1 → Supervisor Mode (S-mode)
/// EL0 → User Mode (U-mode)
pub struct Riscv64PrivilegeMapping;

impl NvArchPrivilegeMapping for Riscv64PrivilegeMapping {
    fn hw_privilege_level(level: NvPrivilegeLevel) -> u8 {
        match level {
            NvPrivilegeLevel::KernelMode => 3,
            NvPrivilegeLevel::EquipmentMode => 1,
            NvPrivilegeLevel::UserMode => 0,
        }
    }

    fn enter_from_lower(from: NvPrivilegeLevel, trap_ctx: &mut NvTrapContext) {
        trap_ctx.from_level = from;
    }

    fn return_to_lower(ctx: &NvTrapContext) -> ! {
        let _ = ctx;
        loop {}
    }

    fn supervisor_call_entry(sv_ctx: &NvSupervisorContext) -> NvSupervisorResult {
        let _ = sv_ctx;
        NvSupervisorResult { value: 0, output: [0; 4] }
    }

    fn supervisor_call_return(result: &NvSupervisorResult) -> ! {
        let _ = result;
        loop {}
    }

    fn current_privilege_level() -> NvPrivilegeLevel {
        NvPrivilegeLevel::KernelMode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privilege_ordering() {
        assert!(NvPrivilegeLevel::KernelMode > NvPrivilegeLevel::EquipmentMode);
        assert!(NvPrivilegeLevel::EquipmentMode > NvPrivilegeLevel::UserMode);
    }

    #[test]
    fn test_cross_level_access_denied() {
        assert!(!NvArchPrivilegeMapping::is_cross_level_access_allowed(
            Arm64PrivilegeMapping, NvPrivilegeLevel::UserMode, NvPrivilegeLevel::KernelMode
        ));
        assert!(!NvArchPrivilegeMapping::is_cross_level_access_allowed(
            Arm64PrivilegeMapping, NvPrivilegeLevel::EquipmentMode, NvPrivilegeLevel::KernelMode
        ));
    }

    #[test]
    fn test_same_level_access_allowed() {
        assert!(NvArchPrivilegeMapping::is_cross_level_access_allowed(
            Arm64PrivilegeMapping, NvPrivilegeLevel::UserMode, NvPrivilegeLevel::UserMode
        ));
    }

    #[test]
    fn test_arm64_hw_mapping() {
        assert_eq!(Arm64PrivilegeMapping::hw_privilege_level(NvPrivilegeLevel::KernelMode), 2);
        assert_eq!(Arm64PrivilegeMapping::hw_privilege_level(NvPrivilegeLevel::EquipmentMode), 1);
        assert_eq!(Arm64PrivilegeMapping::hw_privilege_level(NvPrivilegeLevel::UserMode), 0);
    }

    #[test]
    fn test_x64_hw_mapping() {
        assert_eq!(X64PrivilegeMapping::hw_privilege_level(NvPrivilegeLevel::KernelMode), 0);
        assert_eq!(X64PrivilegeMapping::hw_privilege_level(NvPrivilegeLevel::EquipmentMode), 1);
        assert_eq!(X64PrivilegeMapping::hw_privilege_level(NvPrivilegeLevel::UserMode), 3);
    }

    #[test]
    fn test_riscv64_hw_mapping() {
        assert_eq!(Riscv64PrivilegeMapping::hw_privilege_level(NvPrivilegeLevel::KernelMode), 3);
        assert_eq!(Riscv64PrivilegeMapping::hw_privilege_level(NvPrivilegeLevel::EquipmentMode), 1);
        assert_eq!(Riscv64PrivilegeMapping::hw_privilege_level(NvPrivilegeLevel::UserMode), 0);
    }
}
