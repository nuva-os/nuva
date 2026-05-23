use crate::{pr_info};
/*
 * Nuva OS - Kernel - Virtualization Support
 * 
 * Hardware virtualization support (VMX/SVM).
 * 
 * Copyright (C) 2026 Nuva OS Team
 */

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};

/// Virtualization type
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmxType {
    None = 0,
    IntelVmx = 1,  // Intel VT-x
    AmdSvm = 2,     // AMD-V
}

/// VM state
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmState {
    Created = 0,
    Running = 1,
    Paused = 2,
    Stopped = 3,
    Crashed = 4,
}

/// VMX MSR addresses
pub mod vmx_msr {
    pub const IA32_VMX_BASIC: u32 = 0x480;
    pub const IA32_VMX_PINBASED_CTLS: u32 = 0x481;
    pub const IA32_VMX_PROCBASED_CTLS: u32 = 0x482;
    pub const IA32_VMX_EXIT_CTLS: u32 = 0x483;
    pub const IA32_VMX_ENTRY_CTLS: u32 = 0x484;
    pub const IA32_VMX_MISC: u32 = 0x485;
    pub const IA32_VMX_CR0_FIXED0: u32 = 0x486;
    pub const IA32_VMX_CR0_FIXED1: u32 = 0x487;
    pub const IA32_VMX_CR4_FIXED0: u32 = 0x488;
    pub const IA32_VMX_CR4_FIXED1: u32 = 0x489;
    pub const IA32_VMX_VMCS_ENUM: u32 = 0x48A;
}

/// VMCS field encodings
pub mod vmcs_field {
    // 16-bit control fields
    pub const VPID: u32 = 0x0000;
    
    // 16-bit guest state fields
    pub const GUEST_ES_SELECTOR: u32 = 0x0800;
    pub const GUEST_CS_SELECTOR: u32 = 0x0802;
    pub const GUEST_SS_SELECTOR: u32 = 0x0804;
    pub const GUEST_DS_SELECTOR: u32 = 0x0806;
    pub const GUEST_FS_SELECTOR: u32 = 0x0808;
    pub const GUEST_GS_SELECTOR: u32 = 0x080A;
    pub const GUEST_LDTR_SELECTOR: u32 = 0x080C;
    pub const GUEST_TR_SELECTOR: u32 = 0x080E;
    
    // 64-bit control fields
    pub const IO_BITMAP_A: u32 = 0x2000;
    pub const IO_BITMAP_B: u32 = 0x2002;
    pub const MSR_BITMAP: u32 = 0x2004;
    pub const EPTP: u32 = 0x201A;
    
    // 64-bit guest state fields
    pub const VMCS_LINK_POINTER: u32 = 0x2800;
    pub const GUEST_IA32_DEBUGCTL: u32 = 0x2802;
    
    // 32-bit control fields
    pub const PIN_BASED_VM_EXEC_CONTROL: u32 = 0x4000;
    pub const CPU_BASED_VM_EXEC_CONTROL: u32 = 0x4002;
    pub const EXCEPTION_BITMAP: u32 = 0x4004;
    pub const VM_ENTRY_CONTROLS: u32 = 0x4012;
    pub const VM_EXIT_CONTROLS: u32 = 0x400C;
    
    // 32-bit guest state fields
    pub const GUEST_ES_LIMIT: u32 = 0x4800;
    pub const GUEST_CS_LIMIT: u32 = 0x4802;
    pub const GUEST_SS_LIMIT: u32 = 0x4804;
    pub const GUEST_DS_LIMIT: u32 = 0x4806;
    pub const GUEST_FS_LIMIT: u32 = 0x4808;
    pub const GUEST_GS_LIMIT: u32 = 0x480A;
    pub const GUEST_LDTR_LIMIT: u32 = 0x480C;
    pub const GUEST_TR_LIMIT: u32 = 0x480E;
    pub const GUEST_GDTR_LIMIT: u32 = 0x4810;
    pub const GUEST_IDTR_LIMIT: u32 = 0x4812;
    pub const GUEST_ES_AR_BYTES: u32 = 0x4814;
    pub const GUEST_CS_AR_BYTES: u32 = 0x4816;
    pub const GUEST_SS_AR_BYTES: u32 = 0x4818;
    pub const GUEST_DS_AR_BYTES: u32 = 0x481A;
    pub const GUEST_FS_AR_BYTES: u32 = 0x481C;
    pub const GUEST_GS_AR_BYTES: u32 = 0x481E;
    pub const GUEST_LDTR_AR_BYTES: u32 = 0x4820;
    pub const GUEST_TR_AR_BYTES: u32 = 0x4822;
    pub const GUEST_INTERRUPTIBILITY_INFO: u32 = 0x4824;
    pub const GUEST_ACTIVITY_STATE: u32 = 0x4826;
    
    // Natural-width guest state fields
    pub const GUEST_CR0: u32 = 0x6800;
    pub const GUEST_CR2: u32 = 0x6802;
    pub const GUEST_CR3: u32 = 0x6804;
    pub const GUEST_CR4: u32 = 0x6806;
    pub const GUEST_ES_BASE: u32 = 0x6808;
    pub const GUEST_CS_BASE: u32 = 0x680A;
    pub const GUEST_SS_BASE: u32 = 0x680C;
    pub const GUEST_DS_BASE: u32 = 0x680E;
    pub const GUEST_FS_BASE: u32 = 0x6810;
    pub const GUEST_GS_BASE: u32 = 0x6812;
    pub const GUEST_LDTR_BASE: u32 = 0x6814;
    pub const GUEST_TR_BASE: u32 = 0x6816;
    pub const GUEST_GDTR_BASE: u32 = 0x6818;
    pub const GUEST_IDTR_BASE: u32 = 0x681A;
    pub const GUEST_RSP: u32 = 0x681C;
    pub const GUEST_RIP: u32 = 0x681E;
    pub const GUEST_RFLAGS: u32 = 0x6820;
}

/// VM exit reasons
pub mod exit_reason {
    pub const EXCEPTION_NMI: u32 = 0;
    pub const EXTERNAL_INTERRUPT: u32 = 1;
    pub const TRIPLE_FAULT: u32 = 2;
    pub const INIT_SIGNAL: u32 = 3;
    pub const SIPI: u32 = 4;
    pub const IO_SMI: u32 = 5;
    pub const OTHER_SMI: u32 = 6;
    pub const INTERRUPT_WINDOW: u32 = 7;
    pub const NMI_WINDOW: u32 = 8;
    pub const TASK_SWITCH: u32 = 9;
    pub const CPUID: u32 = 10;
    pub const HLT: u32 = 12;
    pub const INVD: u32 = 13;
    pub const INVLPG: u32 = 14;
    pub const RDPMC: u32 = 15;
    pub const RDTSC: u32 = 16;
    pub const VMCALL: u32 = 18;
    pub const VMLAUNCH: u32 = 19;
    pub const VMRESUME: u32 = 20;
    pub const VMXOFF: u32 = 22;
    pub const CR_ACCESS: u32 = 28;
    pub const DR_ACCESS: u32 = 29;
    pub const IO_INSTRUCTION: u32 = 30;
    pub const RDMSR: u32 = 31;
    pub const WRMSR: u32 = 32;
    pub const ENTRY_FAIL_GUEST_STATE: u32 = 33;
    pub const ENTRY_FAIL_MSR_LOADING: u32 = 34;
    pub const EPT_VIOLATION: u32 = 48;
    pub const EPT_MISCONFIG: u32 = 49;
    pub const INVEPT: u32 = 50;
    pub const INVVPID: u32 = 51;
}

/// Virtual CPU
pub struct Vcpu {
    /// VCPU ID
    pub id: u32,
    /// VM ID
    pub vm_id: u64,
    /// State
    pub state: AtomicU32,
    /// VMCS pointer
    pub vmcs: AtomicU64,
    /// APIC ID
    pub apic_id: u32,
    /// Exit reason
    pub exit_reason: AtomicU32,
    /// Exit qualification
    pub exit_qual: AtomicU64,
    /// Number of exits
    pub exit_count: AtomicU64,
}

impl Clone for Vcpu {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            vm_id: self.vm_id.clone(),
            state: AtomicU32::new(self.state.load(core::sync::atomic::Ordering::Relaxed)),
            vmcs: AtomicU64::new(self.vmcs.load(core::sync::atomic::Ordering::Relaxed)),
            apic_id: self.apic_id.clone(),
            exit_reason: AtomicU32::new(self.exit_reason.load(core::sync::atomic::Ordering::Relaxed)),
            exit_qual: AtomicU64::new(self.exit_qual.load(core::sync::atomic::Ordering::Relaxed)),
            exit_count: AtomicU64::new(self.exit_count.load(core::sync::atomic::Ordering::Relaxed)),
        }
    }
}

impl Vcpu {
    pub fn new(id: u32, vm_id: u64) -> Self {
        Vcpu {
            id,
            vm_id,
            state: AtomicU32::new(VmState::Created as u32),
            vmcs: AtomicU64::new(0),
            apic_id: id,
            exit_reason: AtomicU32::new(0),
            exit_qual: AtomicU64::new(0),
            exit_count: AtomicU64::new(0),
        }
    }
    
    /// Run VCPU
    pub fn run(&mut self) -> Result<(), i32> {
        self.state.store(VmState::Running as u32, Ordering::Release);
        
        // Execute VMRESUME or VMLAUNCH
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let vmcs = self.vmcs.load(Ordering::Acquire);
            if vmcs == 0 {
                return Err(-22);
            }
            
            // VMRESUME
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!(
                "vmresume",
                "setc {0}",
                "setz {1}",
                out(reg) _,
                out(reg) _,
            );
            #[cfg(not(target_arch = "x86_64"))]
            { /* TODO: VMX not supported on this architecture */ }
        }
        
        Ok(())
    }
    
    /// Pause VCPU
    pub fn pause(&mut self) {
        self.state.store(VmState::Paused as u32, Ordering::Release);
    }
    
    /// Resume VCPU
    pub fn resume(&mut self) {
        self.state.store(VmState::Running as u32, Ordering::Release);
    }
}

/// Virtual Machine
pub struct VirtualMachine {
    /// VM ID
    pub id: u64,
    /// Name
    pub name: [u8; 32],
    /// State
    pub state: AtomicU32,
    /// Number of VCPUs
    pub nr_vcpus: u32,
    /// VCPUs
    pub vcpus: [Option<Vcpu>; 128],
    /// Memory size
    pub mem_size: u64,
    /// Memory base
    pub mem_base: AtomicU64,
    /// EPT root
    pub ept_root: AtomicU64,
    /// Flags
    pub flags: AtomicU32,
}

impl VirtualMachine {
    pub fn new(name: &str, nr_vcpus: u32, mem_size: u64) -> Self {
        let mut name_buf = [0u8; 32];
        let len = name.as_bytes().len().min(31);
        name_buf[..len].copy_from_slice(&name.as_bytes()[..len]);
        
        VirtualMachine {
            id: 0,
            name: name_buf,
            state: AtomicU32::new(VmState::Created as u32),
            nr_vcpus,
            vcpus: core::array::from_fn(|_| None),
            mem_size,
            mem_base: AtomicU64::new(0),
            ept_root: AtomicU64::new(0),
            flags: AtomicU32::new(0),
        }
    }
    
    /// Create VCPU
    pub fn create_vcpu(&mut self, id: u32) -> Result<(), i32> {
        if id as usize >= self.vcpus.len() {
            return Err(-22);
        }
        
        if self.vcpus[id as usize].is_some() {
            return Err(-17); // EEXIST
        }
        
        self.vcpus[id as usize] = Some(Vcpu::new(id, self.id));
        Ok(())
    }
    
    /// Start VM
    pub fn start(&mut self) -> Result<(), i32> {
        self.state.store(VmState::Running as u32, Ordering::Release);
        Ok(())
    }
    
    /// Stop VM
    pub fn stop(&mut self) {
        self.state.store(VmState::Stopped as u32, Ordering::Release);
    }
    
    /// Pause VM
    pub fn pause(&mut self) {
        self.state.store(VmState::Paused as u32, Ordering::Release);
    }
}

/// Virtualization manager
pub struct VmxManager {
    /// Virtualization type
    pub vmx_type: AtomicU32,
    /// Supported
    pub supported: AtomicBool,
    /// VMs
    vms: spin::Mutex<alloc::collections::BTreeMap<u64, VirtualMachine>>,
    next_vm_id: AtomicU64,
}

impl VmxManager {
    pub const fn new() -> Self {
        VmxManager {
            vmx_type: AtomicU32::new(VmxType::None as u32),
            supported: AtomicBool::new(false),
            vms: spin::Mutex::new(alloc::collections::BTreeMap::new()),
            next_vm_id: AtomicU64::new(1),
        }
    }
    
    /// Check virtualization support
    pub fn check_support(&mut self) -> bool {
        // Check CPUID for VMX support
        let mut eax: u32 = 0;
        let mut ebx: u32 = 0;
        let mut ecx: u32 = 0;
        let mut edx: u32 = 0;
        
        // SAFETY: inline assembly required for hardware instruction
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!(
                "cpuid",
                inout("eax") 1 => eax,
                out("ebx") ebx,
                out("ecx") ecx,
                out("edx") edx,
            );
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            // TODO: CPUID not available on this architecture
            ecx = 0;
        }
        
        // Check VMX bit (bit 5 of ECX)
        if ecx & (1 << 5) != 0 {
            self.vmx_type.store(VmxType::IntelVmx as u32, Ordering::Release);
            self.supported.store(true, Ordering::Release);
            return true;
        }
        
        // Check for AMD SVM
        // SAFETY: inline assembly required for hardware instruction
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!(
                "cpuid",
                inout("eax") 0x80000001 => eax,
                out("ebx") ebx,
                out("ecx") ecx,
                out("edx") edx,
            );
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            ecx = 0;
        }
        
        if ecx & (1 << 2) != 0 {
            self.vmx_type.store(VmxType::AmdSvm as u32, Ordering::Release);
            self.supported.store(true, Ordering::Release);
            return true;
        }
        
        false
    }
    
    /// Create VM
    pub fn create_vm(&self, name: &str, nr_vcpus: u32, mem_size: u64) -> Result<u64, i32> {
        if !self.supported.load(Ordering::Acquire) {
            return Err(-95); // EOPNOTSUPP
        }
        
        let id = self.next_vm_id.fetch_add(1, Ordering::AcqRel);
        let mut vm = VirtualMachine::new(name, nr_vcpus, mem_size);
        vm.id = id;
        
        // Create VCPUs
        for i in 0..nr_vcpus {
            vm.create_vcpu(i)?;
        }
        
        self.vms.lock().insert(id, vm);
        Ok(id)
    }
    
    /// Destroy VM
    pub fn destroy_vm(&self, id: u64) -> Result<(), i32> {
        if self.vms.lock().remove(&id).is_some() {
            Ok(())
        } else {
            Err(-2)
        }
    }
}

impl Default for VmxManager {
    fn default() -> Self { Self::new() }
}

/// Global VMX manager
static VMX_MANAGER: core::sync::OnceLock<VmxManager> = core::sync::OnceLock::new();

/// Get VMX manager
pub fn vmx_manager() -> &'static VmxManager {
    VMX_MANAGER.get_or_init(VmxManager::new)
}

pub fn init_vmx_manager() -> &'static VmxManager {
    VMX_MANAGER.get_or_init(VmxManager::new)
}

/// Initialize virtualization
pub fn init_vmx() {
    let mgr = vmx_manager();
    
    if mgr.check_support() {
        let vmx_type = match mgr.vmx_type.load(Ordering::Acquire) {
            1 => "Intel VT-x",
            2 => "AMD-V",
            _ => "Unknown",
        };
        log_info!("Virtualization support detected: {}", vmx_type);
    } else {
        log_info!("No virtualization support detected");
    }
}
