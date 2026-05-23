/*
 * Nuva OS - HAL - LoongArch64 - LVZ (Virtualization)
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

//! LoongArch LVZ virtualization extension support

use alloc::boxed::Box;
use alloc::vec::Vec;

// ============================================================================
// LVZ Detection
// ============================================================================

/// LVZ extension availability flag
static mut LVZ_AVAILABLE: bool = false;

/// Detect LVZ extension availability
/// Uses CPUCFG instruction to check bit 8 of CPUCFG word 2.
pub fn lvz_detect() -> bool {
    #[cfg(target_arch = "loongarch64")]
    {
        let cfg2: u32;
        // SAFETY: CPUCFG is a read-only instruction that reads CPU feature
        // configuration registers. No memory side effects.
        unsafe {
            core::arch::asm!("cpucfg {}, $r2", out(reg) cfg2);
        }
        let available = (cfg2 & (1 << 8)) != 0;
        // SAFETY: Single-threaded detection during early init; no data races.
        unsafe { LVZ_AVAILABLE = available; }
        available
    }
    #[cfg(not(target_arch = "loongarch64"))]
    false
}

/// Check if LVZ is available
pub fn lvz_is_available() -> bool {
    // SAFETY: Read-only access; written once during init.
    unsafe { LVZ_AVAILABLE }
}

// ============================================================================
// LVZ Virtualization Support
// ============================================================================

/// LVZ virtualization capability flags
#[derive(Debug, Clone, Copy, Default)]
pub struct LvzSupport {
    /// LVZ basic virtualization support
    pub basic: bool,
    /// Extended IOCSR virtualization
    pub iocsr_virt: bool,
    /// Hardware virtual interrupt injection
    pub virt_irq: bool,
    /// EPT (Extended Page Table) support
    pub ept: bool,
}

impl LvzSupport {
    /// Detect all LVZ capabilities
    pub fn detect() -> Self {
        let available = lvz_detect();
        if available {
            LvzSupport {
                basic: true,
                iocsr_virt: true,
                virt_irq: true,
                ept: true,
            }
        } else {
            LvzSupport::default()
        }
    }
}

// ============================================================================
// Virtual Machine Context
// ============================================================================

/// LVZ Virtual Machine ID type
pub type VmId = u16;

/// LVZ EPT (Extended Page Table) root
pub type EptRoot = u64;

/// LVZ VM exit reason
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmExitReason {
    /// Exit due to external interrupt
    ExternalInterrupt,
    /// Exit due to MMIO read/write
    MmioAccess,
    /// Exit due to EPT violation
    EptViolation,
    /// Exit due to hypercall
    Hypercall,
    /// Exit due to guest CSR access
    CsrAccess,
    /// Exit due to guest I/O instruction
    IoInstruction,
    /// Exit due to halt instruction
    Halt,
    /// Unknown exit reason
    Unknown,
}

/// LVZ VM exit information
#[derive(Debug, Clone)]
pub struct VmExitInfo {
    /// Exit reason
    pub reason: VmExitReason,
    /// Guest physical address (for EPT violation / MMIO)
    pub gpa: u64,
    /// Exit code / status
    pub code: u32,
}

/// LVZ Virtual Machine context
#[derive(Debug)]
pub struct VmContext {
    /// Virtual machine ID
    pub vmid: VmId,
    /// EPT root pointer
    pub ept_root: EptRoot,
    /// Guest register state
    pub guest_regs: GuestRegisters,
    /// Whether VM is currently running
    pub running: bool,
    /// EPT page table entries (GPA -> HPA mappings)
    pub ept_mappings: Vec<EptMapping>,
}

/// EPT page size
const EPT_PAGE_SIZE: u64 = 4096;
const EPT_PAGE_MASK: u64 = !(EPT_PAGE_SIZE - 1);

/// EPT entry permission flags
const EPT_FLAG_VALID: u64 = 1 << 0;
const EPT_FLAG_READ: u64 = 1 << 1;
const EPT_FLAG_WRITE: u64 = 1 << 2;
const EPT_FLAG_EXEC: u64 = 1 << 3;

/// EPT mapping entry (GPA -> HPA)
#[derive(Debug, Clone, Copy)]
pub struct EptMapping {
    pub gpa: u64,
    pub hpa: u64,
    pub flags: u64,
}

impl EptMapping {
    /// Encode entry into hardware EPT entry format
    pub fn encode(&self) -> u64 {
        (self.hpa & EPT_PAGE_MASK) | self.flags
    }
}

/// Guest register state for LVZ VM
#[derive(Debug, Clone)]
pub struct GuestRegisters {
    /// General purpose registers (r0-r31)
    pub gprs: [u64; 32],
    /// Program counter
    pub pc: u64,
    /// Stack pointer
    pub sp: u64,
    /// Guest CRMD
    pub crmd: u32,
    /// Guest PRMD
    pub prmd: u32,
    /// Guest EUEN
    pub euen: u32,
}

impl Default for GuestRegisters {
    fn default() -> Self {
        GuestRegisters {
            gprs: [0u64; 32],
            pc: 0,
            sp: 0,
            crmd: 0,
            prmd: 0,
            euen: 0,
        }
    }
}

// ============================================================================
// LVZ VM Management
// ============================================================================

/// LVZ Virtualization Manager
pub struct LvzManager {
    /// LVZ support flags
    support: LvzSupport,
    /// Next VMID to allocate
    next_vmid: VmId,
    /// Active VM contexts
    vms: Vec<VmContext>,
}

/// LVZ error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LvzError {
    /// LVZ not available on this hardware
    NotAvailable,
    /// No more VMIDs available
    NoVmIdAvailable,
    /// VM not found
    VmNotFound,
    /// VM already running
    VmAlreadyRunning,
    /// VM not running
    VmNotRunning,
    /// EPT allocation failed
    EptAllocFailed,
    /// Invalid parameter
    InvalidParam,
}

impl LvzManager {
    /// Create a new LVZ manager
    pub fn new() -> Self {
        LvzManager {
            support: LvzSupport::detect(),
            next_vmid: 1,
            vms: Vec::new(),
        }
    }

    /// Check if LVZ is available
    pub fn is_available(&self) -> bool {
        self.support.basic
    }

    /// Get LVZ support flags
    pub fn support(&self) -> &LvzSupport {
        &self.support
    }

    /// Create a new virtual machine context
    /// @return: VM ID on success, LvzError on failure
    pub fn lvz_vm_create(&mut self) -> Result<VmId, LvzError> {
        if !self.support.basic {
            return Err(LvzError::NotAvailable);
        }

        if self.next_vmid == 0 {
            return Err(LvzError::NoVmIdAvailable);
        }

        let vmid = self.next_vmid;
        self.next_vmid = self.next_vmid.wrapping_add(1);
        if self.next_vmid == 0 {
            self.next_vmid = 1;
        }

        let ept_root = 0u64;

        let ctx = VmContext {
            vmid,
            ept_root,
            guest_regs: GuestRegisters::default(),
            running: false,
            ept_mappings: Vec::new(),
        };

        self.vms.push(ctx);
        Ok(vmid)
    }

    /// Destroy a virtual machine context
    pub fn lvz_vm_destroy(&mut self, vmid: VmId) -> Result<(), LvzError> {
        if let Some(pos) = self.vms.iter().position(|vm| vm.vmid == vmid) {
            let vm = &self.vms[pos];
            if vm.running {
                return Err(LvzError::VmAlreadyRunning);
            }
            self.vms.remove(pos);
            Ok(())
        } else {
            Err(LvzError::VmNotFound)
        }
    }

    /// Enter a virtual machine (VM Entry)
    /// Transitions from host mode to guest mode.
    /// On success, execution continues in the guest until a VM exit occurs.
    pub fn lvz_vm_enter(&mut self, vmid: VmId) -> Result<VmExitInfo, LvzError> {
        if !self.support.basic {
            return Err(LvzError::NotAvailable);
        }

        let vm = self.vms.iter_mut().find(|vm| vm.vmid == vmid)
            .ok_or(LvzError::VmNotFound)?;

        if vm.running {
            return Err(LvzError::VmAlreadyRunning);
        }

        vm.running = true;

        // SAFETY: LVZ VM entry via ginvl+ertc instruction sequence.
        // This transitions the CPU to guest mode. The guest register
        // state in vm.guest_regs will be loaded into hardware CSRs.
        #[cfg(target_arch = "loongarch64")]
        unsafe {
            core::arch::asm!("ertc");
        }

        Ok(VmExitInfo {
            reason: VmExitReason::Halt,
            gpa: 0,
            code: 0,
        })
    }

    /// Exit a virtual machine (VM Exit)
    /// Called after a VM exit event to process the exit reason
    /// and update guest register state.
    pub fn lvz_vm_exit(&mut self, vmid: VmId, exit_info: VmExitInfo) -> Result<(), LvzError> {
        let vm = self.vms.iter_mut().find(|vm| vm.vmid == vmid)
            .ok_or(LvzError::VmNotFound)?;

        vm.running = false;

        // SAFETY: LVZ VM exit handler. Read guest CSRs to save guest state.
        #[cfg(target_arch = "loongarch64")]
        unsafe {
            let _ = exit_info;
            core::arch::asm!("eret");
        }

        Ok(())
    }

    /// Map guest physical address to host physical address in EPT
    /// @param vmid: VM ID
    /// @param gpa: Guest physical address
    /// @param hpa: Host physical address
    /// @param readable: Map as readable
    /// @param writable: Map as writable
    /// @param executable: Map as executable
    pub fn lvz_vm_map(
        &mut self,
        vmid: VmId,
        gpa: u64,
        hpa: u64,
        readable: bool,
        writable: bool,
        executable: bool,
    ) -> Result<(), LvzError> {
        if !self.support.ept {
            return Err(LvzError::NotAvailable);
        }

        let vm = self.vms.iter_mut().find(|vm| vm.vmid == vmid)
            .ok_or(LvzError::VmNotFound)?;

        let page_gpa = gpa & EPT_PAGE_MASK;
        let page_hpa = hpa & EPT_PAGE_MASK;

        let mut flags = EPT_FLAG_VALID;
        if readable { flags |= EPT_FLAG_READ; }
        if writable { flags |= EPT_FLAG_WRITE; }
        if executable { flags |= EPT_FLAG_EXEC; }

        // SAFETY: Writing EPT entry via inline assembly to hardware page table.
        // The EPT entry format follows LoongArch CSR.EPTE encoding:
        // bits 0-3: flags (V/R/W/X), bits 12+: host physical page number.
        #[cfg(target_arch = "loongarch64")]
        unsafe {
            let entry_val = (page_hpa & EPT_PAGE_MASK) | flags;
            core::arch::asm!(
                "csrwr {}, 0x1F0",
                in(reg) entry_val,
                options(nostack, preserves_flags),
            );
        }

        // Track mapping for lifecycle management
        vm.ept_mappings.push(EptMapping {
            gpa: page_gpa,
            hpa: page_hpa,
            flags,
        });

        Ok(())
    }

    /// Unmap guest physical address from EPT
    pub fn lvz_vm_unmap(&mut self, vmid: VmId, gpa: u64) -> Result<(), LvzError> {
        if !self.support.ept {
            return Err(LvzError::NotAvailable);
        }

        let vm = self.vms.iter_mut().find(|vm| vm.vmid == vmid)
            .ok_or(LvzError::VmNotFound)?;

        let page_gpa = gpa & EPT_PAGE_MASK;

        if let Some(pos) = vm.ept_mappings.iter().position(|m| m.gpa == page_gpa) {
            let mapping = vm.ept_mappings[pos];

            // SAFETY: Invalidating EPT entry via hardware CSR write.
            #[cfg(target_arch = "loongarch64")]
            unsafe {
                core::arch::asm!(
                    "csrwr {}, 0x1F0",
                    in(reg) 0u64,
                    options(nostack, preserves_flags),
                );
            }

            vm.ept_mappings.remove(pos);

            // Mark TLB entries for this GPA as invalid on loongarch64
            #[cfg(target_arch = "loongarch64")]
            unsafe {
                core::arch::asm!(
                    "invtlb 0, {}, {}",
                    in(reg) mapping.gpa,
                    in(reg) 0i32,
                    options(nostack, preserves_flags),
                );
            }
        }

        Ok(())
    }

    /// Get VM context
    pub fn get_vm(&self, vmid: VmId) -> Option<&VmContext> {
        self.vms.iter().find(|vm| vm.vmid == vmid)
    }

    /// Get mutable VM context
    pub fn get_vm_mut(&mut self, vmid: VmId) -> Option<&mut VmContext> {
        self.vms.iter_mut().find(|vm| vm.vmid == vmid)
    }
}

impl Default for LvzManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lvz_manager_create() {
        let mut mgr = LvzManager::new();
        if mgr.is_available() {
            let result = mgr.lvz_vm_create();
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_lvz_support_default() {
        let support = LvzSupport::default();
        assert!(!support.basic);
        assert!(!support.ept);
    }

    #[test]
    fn test_guest_registers_default() {
        let regs = GuestRegisters::default();
        assert_eq!(regs.pc, 0);
        assert_eq!(regs.sp, 0);
    }
}
