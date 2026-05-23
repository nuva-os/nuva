use crate::{pr_info};
/*
 * Nuva OS - Kernel - Power Management
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Kernel power management subsystem.
 */

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

use crate::posix::errno::Errno;
/// Power State
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    /// Running
    Running = 0,
    /// Idle
    Idle = 1,
    /// Standby
    Standby = 2,
    /// Suspend to RAM
    Suspend = 3,
    /// Hibernate (Suspend to Disk)
    Hibernate = 4,
    /// Power off
    Off = 5,
    /// Reboot
    Reboot = 6,
}

/// Suspend State
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuspendState {
    /// Active
    Active = 0,
    /// Freeze
    Freeze = 1,
    /// Standby
    Standby = 2,
    /// Suspend to RAM
    Mem = 3,
    /// Suspend to Disk
    Disk = 4,
}

/// Power Event
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerEvent {
    /// User request
    User = 0,
    /// System request
    System = 1,
    /// Critical
    Critical = 2,
    /// Battery low
    BatteryLow = 3,
    /// Thermal
    Thermal = 4,
    /// Lid close
    LidClose = 5,
}

/// Power Operations
pub struct PowerOps {
    /// Prepare for suspend
    pub prepare: Option<unsafe extern "C" fn() -> i32>,
    /// Enter suspend
    pub enter: Option<unsafe extern "C" fn(SuspendState) -> i32>,
    /// Resume from suspend
    pub resume: Option<unsafe extern "C" fn() -> i32>,
    /// Prepare for hibernate
    pub freeze: Option<unsafe extern "C" fn() -> i32>,
    /// Restore from hibernate
    pub restore: Option<unsafe extern "C" fn() -> i32>,
    /// Power off
    pub power_off: Option<unsafe extern "C" fn() -> !>,
    /// Reboot
    pub reboot: Option<unsafe extern "C" fn() -> !>,
}

/// Power Domain
pub struct PowerDomain {
    /// Domain name
    pub name: [u8; 32],
    /// Domain ID
    pub id: u32,
    /// Current state
    pub state: AtomicU32,
    /// Target state
    pub target_state: AtomicU32,
    /// Power on latency (us)
    pub power_on_latency: u32,
    /// Power off latency (us)
    pub power_off_latency: u32,
    /// Operations
    pub ops: PowerOps,
    /// Parent domain
    pub parent: *mut PowerDomain,
    /// Children
    pub children: *mut PowerDomain,
    /// Sibling
    pub sibling: *mut PowerDomain,
    /// Device count
    pub dev_count: AtomicU32,
    /// Reference count
    pub ref_count: AtomicU32,
    /// Flags
    pub flags: AtomicU32,
}

/// Power Domain Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct PdFlags: u32 {
        /// Always on
        const ALWAYS_ON = 1 << 0;
        /// Can power off
        const CAN_OFF = 1 << 1;
        /// Can suspend
        const CAN_SUSPEND = 1 << 2;
        /// Can hibernate
        const CAN_HIBERNATE = 1 << 3;
        /// Active
        const ACTIVE = 1 << 4;
        /// Suspended
        const SUSPENDED = 1 << 5;
    }
}

impl PowerDomain {
    pub fn new(name: &[u8], id: u32) -> Self {
        let mut name_arr = [0u8; 32];
        let len = name.len().min(31);
        name_arr[..len].copy_from_slice(&name[..len]);
        
        PowerDomain {
            name: name_arr,
            id,
            state: AtomicU32::new(PowerState::Running as u32),
            target_state: AtomicU32::new(PowerState::Running as u32),
            power_on_latency: 0,
            power_off_latency: 0,
            ops: PowerOps {
                prepare: None,
                enter: None,
                resume: None,
                freeze: None,
                restore: None,
                power_off: None,
                reboot: None,
            },
            parent: core::ptr::null_mut(),
            children: core::ptr::null_mut(),
            sibling: core::ptr::null_mut(),
            dev_count: AtomicU32::new(0),
            ref_count: AtomicU32::new(1),
            flags: AtomicU32::new(PdFlags::ACTIVE.bits()),
        }
    }
    
    /// Get state
    pub fn get_state(&self) -> PowerState {
        match self.state.load(Ordering::Acquire) {
            0 => PowerState::Running,
            1 => PowerState::Idle,
            2 => PowerState::Standby,
            3 => PowerState::Suspend,
            4 => PowerState::Hibernate,
            5 => PowerState::Off,
            6 => PowerState::Reboot,
            _ => PowerState::Running,
        }
    }
    
    /// Power on
    pub fn power_on(&self) -> i32 {
        if self.get_state() == PowerState::Running {
            return 0;
        }
        
        // Power on parent first
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            if !self.parent.is_null() {
                (*self.parent).power_on();
            }
        }
        
        // Call power on callback
        if let Some(resume) = self.ops.resume {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { resume(); }
        }
        
        self.state.store(PowerState::Running as u32, Ordering::Release);
        self.flags.fetch_or(PdFlags::ACTIVE.bits(), Ordering::AcqRel);
        self.flags.fetch_and(!PdFlags::SUSPENDED.bits(), Ordering::AcqRel);
        
        0
    }
    
    /// Power off
    pub fn power_off(&self) -> i32 {
        if self.get_state() == PowerState::Off {
            return 0;
        }
        
        // Check if can power off
        if (self.flags.load(Ordering::Acquire) & PdFlags::ALWAYS_ON.bits()) != 0 {
            return Errno::Ebusy.to_ret_i32(); // EBUSY
        }
        
        // Check reference count
        if self.ref_count.load(Ordering::Acquire) > 1 {
            return Errno::Ebusy.to_ret_i32(); // EBUSY
        }
        
        // Power off children first
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let mut child = self.children;
            while !child.is_null() {
                (*child).power_off();
                child = (*child).sibling;
            }
        }
        
        // Call power off callback
        if let Some(prepare) = self.ops.prepare {
            // SAFETY: unsafe block required for low-level memory or hardware access
            let ret = unsafe { prepare() };
            if ret != 0 {
                return ret;
            }
        }
        
        self.state.store(PowerState::Off as u32, Ordering::Release);
        self.flags.fetch_and(!PdFlags::ACTIVE.bits(), Ordering::AcqRel);
        
        0
    }
    
    /// Suspend
    pub fn suspend(&self, state: SuspendState) -> i32 {
        // Check if can suspend
        if (self.flags.load(Ordering::Acquire) & PdFlags::CAN_SUSPEND.bits()) == 0 {
            return Errno::Eopnotsupp.to_ret_i32(); // EOPNOTSUPP
        }
        
        // Call prepare
        if let Some(prepare) = self.ops.prepare {
            // SAFETY: unsafe block required for low-level memory or hardware access
            let ret = unsafe { prepare() };
            if ret != 0 {
                return ret;
            }
        }
        
        // Enter suspend
        if let Some(enter) = self.ops.enter {
            // SAFETY: unsafe block required for low-level memory or hardware access
            let ret = unsafe { enter(state) };
            if ret != 0 {
                return ret;
            }
        }
        
        self.state.store(PowerState::Suspend as u32, Ordering::Release);
        self.flags.fetch_or(PdFlags::SUSPENDED.bits(), Ordering::AcqRel);
        
        0
    }
    
    /// Resume
    pub fn resume(&self) -> i32 {
        if self.get_state() != PowerState::Suspend {
            return 0;
        }
        
        // Call resume
        if let Some(resume) = self.ops.resume {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { resume(); }
        }
        
        self.state.store(PowerState::Running as u32, Ordering::Release);
        self.flags.fetch_or(PdFlags::ACTIVE.bits(), Ordering::AcqRel);
        self.flags.fetch_and(!PdFlags::SUSPENDED.bits(), Ordering::AcqRel);
        
        0
    }
}

/// Power Manager
pub struct PowerManager {
    /// Current power state
    pub state: AtomicU32,
    /// Target power state
    pub target_state: AtomicU32,
    /// Power domains
    pub domains: *mut PowerDomain,
    /// Domain count
    pub domain_count: AtomicU32,
    /// Suspend count
    pub suspend_count: AtomicU32,
    /// Hibernate count
    pub hibernate_count: AtomicU32,
    /// Battery present
    pub battery_present: AtomicBool,
    /// Battery level (percent)
    pub battery_level: AtomicU32,
    /// Battery charging
    pub battery_charging: AtomicBool,
    /// AC online
    pub ac_online: AtomicBool,
    /// Thermal zone count
    pub thermal_zone_count: AtomicU32,
    /// Critical temperature
    pub critical_temp: AtomicU32,
    /// Statistics
    pub stats: PowerStats,
}

/// Power Statistics
pub struct PowerStats {
    pub suspend_count: AtomicU64,
    pub resume_count: AtomicU64,
    pub hibernate_count: AtomicU64,
    pub restore_count: AtomicU64,
    pub power_off_count: AtomicU64,
    pub reboot_count: AtomicU64,
    pub total_suspend_time: AtomicU64,
    pub total_hibernate_time: AtomicU64,
}

impl PowerStats {
    pub const fn new() -> Self {
        PowerStats {
            suspend_count: AtomicU64::new(0),
            resume_count: AtomicU64::new(0),
            hibernate_count: AtomicU64::new(0),
            restore_count: AtomicU64::new(0),
            power_off_count: AtomicU64::new(0),
            reboot_count: AtomicU64::new(0),
            total_suspend_time: AtomicU64::new(0),
            total_hibernate_time: AtomicU64::new(0),
        }
    }
}

impl PowerManager {
    pub const fn new() -> Self {
        PowerManager {
            state: AtomicU32::new(PowerState::Running as u32),
            target_state: AtomicU32::new(PowerState::Running as u32),
            domains: core::ptr::null_mut(),
            domain_count: AtomicU32::new(0),
            suspend_count: AtomicU32::new(0),
            hibernate_count: AtomicU32::new(0),
            battery_present: AtomicBool::new(false),
            battery_level: AtomicU32::new(100),
            battery_charging: AtomicBool::new(false),
            ac_online: AtomicBool::new(true),
            thermal_zone_count: AtomicU32::new(0),
            critical_temp: AtomicU32::new(100),
            stats: PowerStats::new(),
        }
    }
    
    /// Initialize
    pub fn init(&self) {
        // Register default power domains
        self.register_default_domains();
        
        log_info!("Power manager initialized");
    }
    
    /// Register default power domains
    fn register_default_domains(&mut self) {
        // CPU domain
        let cpu_domain = PowerDomain::new(b"cpu", 0);
        // TODO: Register
        
        // Memory domain
        let mem_domain = PowerDomain::new(b"memory", 1);
        // TODO: Register
        
        // Device domain
        let dev_domain = PowerDomain::new(b"devices", 2);
        // TODO: Register
        
        let _ = (cpu_domain, mem_domain, dev_domain);
    }
    
    /// Get power state
    pub fn get_state(&self) -> PowerState {
        match self.state.load(Ordering::Acquire) {
            0 => PowerState::Running,
            1 => PowerState::Idle,
            2 => PowerState::Standby,
            3 => PowerState::Suspend,
            4 => PowerState::Hibernate,
            5 => PowerState::Off,
            6 => PowerState::Reboot,
            _ => PowerState::Running,
        }
    }
    
    /// Suspend system
    pub fn suspend(&mut self, state: SuspendState, event: PowerEvent) -> i32 {
        // Check if already suspended
        if self.get_state() != PowerState::Running {
            return Errno::Ebusy.to_ret_i32(); // EBUSY
        }
        
        // Check battery level for critical events
        if event == PowerEvent::BatteryLow && self.battery_level.load(Ordering::Acquire) > 5 {
            // Not critical yet
            return 0;
        }
        
        log_info!("Suspending system (state={:?}, event={:?})", state, event);
        
        // Set target state
        self.target_state.store(PowerState::Suspend as u32, Ordering::Release);
        
        // Suspend all power domains
        let mut domain = self.domains;
        while !domain.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let _ = (*domain).suspend(state);
                domain = (*domain).sibling;
            }
        }
        
        // Update state
        self.state.store(PowerState::Suspend as u32, Ordering::Release);
        self.suspend_count.fetch_add(1, Ordering::AcqRel);
        self.stats.suspend_count.fetch_add(1, Ordering::AcqRel);
        
        0
    }
    
    /// Resume system
    pub fn resume(&mut self) -> i32 {
        if self.get_state() != PowerState::Suspend {
            return 0;
        }
        
        log_info!("Resuming system");
        
        // Resume all power domains
        let mut domain = self.domains;
        while !domain.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let _ = (*domain).resume();
                domain = (*domain).sibling;
            }
        }
        
        // Update state
        self.state.store(PowerState::Running as u32, Ordering::Release);
        self.stats.resume_count.fetch_add(1, Ordering::AcqRel);
        
        0
    }
    
    /// Hibernate system
    pub fn hibernate(&mut self) -> i32 {
        if self.get_state() != PowerState::Running {
            return Errno::Ebusy.to_ret_i32();
        }
        
        log_info!("Hibernating system");
        
        self.target_state.store(PowerState::Hibernate as u32, Ordering::Release);
        self.state.store(PowerState::Hibernate as u32, Ordering::Release);
        self.hibernate_count.fetch_add(1, Ordering::AcqRel);
        self.stats.hibernate_count.fetch_add(1, Ordering::AcqRel);
        
        0
    }
    
    /// Restore from hibernate
    pub fn restore(&mut self) -> i32 {
        if self.get_state() != PowerState::Hibernate {
            return 0;
        }
        
        log_info!("Restoring from hibernate");
        
        self.state.store(PowerState::Running as u32, Ordering::Release);
        self.stats.restore_count.fetch_add(1, Ordering::AcqRel);
        
        0
    }
    
    /// Power off system
    pub fn power_off(&mut self) -> ! {
        log_info!("Powering off system");
        
        self.state.store(PowerState::Off as u32, Ordering::Release);
        self.stats.power_off_count.fetch_add(1, Ordering::AcqRel);
        
        // TODO: Call platform power off
        loop {
            core::hint::spin_loop();
        }
    }
    
    /// Reboot system
    pub fn reboot(&mut self) -> ! {
        log_info!("Rebooting system");
        
        self.state.store(PowerState::Reboot as u32, Ordering::Release);
        self.stats.reboot_count.fetch_add(1, Ordering::AcqRel);
        
        // TODO: Call platform reboot
        loop {
            core::hint::spin_loop();
        }
    }
    
    /// Check battery
    pub fn check_battery(&self) -> bool {
        if !self.battery_present.load(Ordering::Acquire) {
            return true;
        }
        
        let level = self.battery_level.load(Ordering::Acquire);
        let charging = self.battery_charging.load(Ordering::Acquire);
        
        if level < 5 && !charging {
            return false;
        }
        
        true
    }
    
    /// Get battery status
    pub fn get_battery_status(&self) -> (u32, bool, bool) {
        (
            self.battery_level.load(Ordering::Acquire),
            self.battery_charging.load(Ordering::Acquire),
            self.ac_online.load(Ordering::Acquire),
        )
    }
    
    /// Register power domain
    pub fn register_domain(&mut self, domain: *mut PowerDomain) -> i32 {
        if domain.is_null() {
            return Errno::Einval.to_ret_i32();
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*domain).sibling = self.domains;
            self.domains = domain;
        }
        
        self.domain_count.fetch_add(1, Ordering::AcqRel);
        0
    }
    
    /// Find power domain
    pub fn find_domain(&self, name: &[u8]) -> Option<*mut PowerDomain> {
        let mut domain = self.domains;
        
        while !domain.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let domain_name = &(*domain).name;
                if domain_name[..name.len()] == *name {
                    return Some(domain);
                }
                domain = (*domain).sibling;
            }
        }
        
        None
    }
}

/// Global power manager
static POWER_MANAGER: core::sync::OnceLock<PowerManager> = core::sync::OnceLock::new();

/// Get power manager
pub fn power_manager() -> &'static PowerManager {
    POWER_MANAGER.get_or_init(PowerManager::new)
}

pub fn init_power_manager() -> &'static PowerManager {
    POWER_MANAGER.get_or_init(PowerManager::new)
}

/// Initialize power management
pub fn init_pm() {
    let mgr = power_manager();
    mgr.init();
}

// Convenience functions

/// Suspend system
pub fn pm_suspend(state: SuspendState) -> i32 {
    power_manager().suspend(state, PowerEvent::User)
}

/// Resume system
pub fn pm_resume() -> i32 {
    power_manager().resume()
}

/// Hibernate system
pub fn pm_hibernate() -> i32 {
    power_manager().hibernate()
}

/// Power off system
pub fn pm_power_off() -> ! {
    power_manager().power_off()
}

/// Reboot system
pub fn pm_reboot() -> ! {
    power_manager().reboot()
}

/// Check if system is suspended
pub fn pm_suspended() -> bool {
    power_manager().get_state() == PowerState::Suspend
}

/// Get battery level
pub fn pm_battery_level() -> u32 {
    power_manager().battery_level.load(Ordering::Acquire)
}

/// Check if on battery
pub fn pm_on_battery() -> bool {
    let mgr = power_manager();
    mgr.battery_present.load(Ordering::Acquire) 
        && !mgr.ac_online.load(Ordering::Acquire)
}

/// Device Power Operations
pub struct DevPowerOps {
    pub prepare: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    pub complete: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    pub suspend: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    pub resume: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    pub freeze: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    pub thaw: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    pub poweroff: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    pub restore: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    pub suspend_late: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    pub resume_early: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    pub suspend_noirq: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    pub resume_noirq: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
}

/// Runtime Power Management
pub struct RuntimePower {
    pub usage_count: AtomicU32,
    pub disable_depth: AtomicU32,
    pub runtime_status: AtomicU32,
    pub runtime_suspended: AtomicBool,
    pub idle_notification: AtomicBool,
    pub request_pending: AtomicBool,
    pub deferred_resume: AtomicBool,
    pub run_wq: AtomicBool,
}

impl RuntimePower {
    pub const fn new() -> Self {
        RuntimePower {
            usage_count: AtomicU32::new(0),
            disable_depth: AtomicU32::new(0),
            runtime_status: AtomicU32::new(0),
            runtime_suspended: AtomicBool::new(false),
            idle_notification: AtomicBool::new(false),
            request_pending: AtomicBool::new(false),
            deferred_resume: AtomicBool::new(false),
            run_wq: AtomicBool::new(false),
        }
    }
    
    /// Get reference
    pub fn get(&self) -> i32 {
        if self.disable_depth.load(Ordering::Acquire) > 0 {
            return Errno::Eacces.to_ret_i32(); // EACCES
        }
        
        self.usage_count.fetch_add(1, Ordering::AcqRel);
        0
    }
    
    /// Put reference
    pub fn put(&self) {
        self.usage_count.fetch_sub(1, Ordering::AcqRel);
    }
    
    /// Suspend runtime
    pub fn runtime_suspend(&self) -> i32 {
        if self.runtime_suspended.load(Ordering::Acquire) {
            return 0;
        }
        
        self.runtime_suspended.store(true, Ordering::Release);
        0
    }
    
    /// Resume runtime
    pub fn runtime_resume(&self) -> i32 {
        if !self.runtime_suspended.load(Ordering::Acquire) {
            return 0;
        }
        
        self.runtime_suspended.store(false, Ordering::Release);
        0
    }
}
