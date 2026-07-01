/*
 * Nuva OS - Kernel - Device - Module
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
 * Nuva OS - Kernel - Module Loader
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Kernel module loading and management.
 */

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

/// Module ID
pub type ModuleId = u32;

/// Module State
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleState {
    /// Unloaded
    Unloaded = 0,
    /// Loading
    Loading = 1,
    /// Loaded
    Loaded = 2,
    /// Initializing
    Initializing = 3,
    /// Live
    Live = 4,
    /// Coming
    Coming = 5,
    /// Going
    Going = 6,
}

/// Module Info
#[repr(C)]
pub struct ModuleInfo {
    /// Module name
    pub name: [u8; 64],
    /// Module version
    pub version: [u8; 32],
    /// Module author
    pub author: [u8; 64],
    /// Module description
    pub description: [u8; 256],
    /// Module license
    pub license: [u8; 32],
    /// Source file
    pub srcversion: [u8; 64],
    /// Dependencies
    pub depends: [u8; 256],
}

/// Module Symbols
#[repr(C)]
pub struct ModuleSymbols {
    /// Symbol table
    pub syms: *mut ModuleSymbol,
    /// Number of symbols
    pub num_syms: u32,
    /// String table
    pub strtab: *mut u8,
    /// String table size
    pub strtab_size: u32,
}

/// Module Symbol
#[repr(C)]
pub struct ModuleSymbol {
    /// Symbol name offset
    pub name_off: u32,
    /// Symbol value
    pub value: u64,
    /// Symbol size
    pub size: u32,
    /// Symbol type
    pub sym_type: u8,
    /// Symbol bind
    pub bind: u8,
    /// Section index
    pub shndx: u16,
}

/// Module Sections
#[repr(C)]
pub struct ModuleSections {
    /// Sections
    pub secs: *mut ModuleSection,
    /// Number of sections
    pub num_secs: u32,
}

/// Module Section
#[repr(C)]
pub struct ModuleSection {
    /// Section name
    pub name: [u8; 32],
    /// Section address
    pub addr: u64,
    /// Section size
    pub size: u64,
    /// Section flags
    pub flags: u64,
}

/// Module Operations
pub struct ModuleOps {
    /// Init function
    pub init: Option<unsafe extern "C" fn() -> i32>,
    /// Exit function
    pub exit: Option<unsafe extern "C" fn()>,
    /// Probe function
    pub probe: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Remove function
    pub remove: Option<unsafe extern "C" fn(*mut core::ffi::c_void)>,
    /// Suspend function
    pub suspend: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
    /// Resume function
    pub resume: Option<unsafe extern "C" fn(*mut core::ffi::c_void) -> i32>,
}

/// Module Structure
pub struct Module {
    /// Module ID
    pub id: ModuleId,
    /// Module info
    pub info: ModuleInfo,
    /// State
    pub state: AtomicU32,
    /// Operations
    pub ops: ModuleOps,
    /// Symbols
    pub syms: ModuleSymbols,
    /// Sections
    pub secs: ModuleSections,
    /// Base address
    pub base: u64,
    /// Size
    pub size: u64,
    /// Reference count
    pub ref_count: AtomicU32,
    /// Taints
    pub taints: AtomicU64,
    /// Flags
    pub flags: AtomicU32,
    /// Next module
    pub next: *mut Module,
}

/// Module Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ModFlags: u32 {
        /// Live patch
        const LIVEPATCH = 1 << 0;
        /// Force load
        const FORCE = 1 << 1;
        /// Experimental
        const EXPERIMENTAL = 1 << 2;
        /// Unsigned
        const UNSIGNED = 1 << 3;
        /// Init ok
        const INIT_OK = 1 << 4;
    }
}

/// Module Taints
pub mod mod_taint {
    pub const TAINT_PROPRIETARY: u64 = 1 << 0;
    pub const TAINT_FORCED: u64 = 1 << 1;
    pub const TAINT_UNSAFE: u64 = 1 << 2;
    pub const TAINT_BUG: u64 = 1 << 3;
    pub const TAINT_FIRMWARE_WORKAROUND: u64 = 1 << 4;
    pub const TAINT_OOT_MODULE: u64 = 1 << 5;
    pub const TAINT_UNSIGNED_MODULE: u64 = 1 << 6;
    pub const TAINT_SOFTLOCKUP: u64 = 1 << 7;
    pub const TAINT_LIVEPATCH: u64 = 1 << 8;
}

impl Module {
    pub fn new(id: ModuleId, name: &[u8]) -> Self {
        let mut name_arr = [0u8; 64];
        let len = name.len().min(63);
        name_arr[..len].copy_from_slice(&name[..len]);
        
        Module {
            id,
            info: ModuleInfo {
                name: name_arr,
                version: [0; 32],
                author: [0; 64],
                description: [0; 256],
                license: [0; 32],
                srcversion: [0; 64],
                depends: [0; 256],
            },
            state: AtomicU32::new(ModuleState::Unloaded as u32),
            ops: ModuleOps {
                init: None,
                exit: None,
                probe: None,
                remove: None,
                suspend: None,
                resume: None,
            },
            syms: ModuleSymbols {
                syms: core::ptr::null_mut(),
                num_syms: 0,
                strtab: core::ptr::null_mut(),
                strtab_size: 0,
            },
            secs: ModuleSections {
                secs: core::ptr::null_mut(),
                num_secs: 0,
            },
            base: 0,
            size: 0,
            ref_count: AtomicU32::new(1),
            taints: AtomicU64::new(0),
            flags: AtomicU32::new(0),
            next: core::ptr::null_mut(),
        }
    }
    
    /// Get state
    pub fn get_state(&self) -> ModuleState {
        match self.state.load(Ordering::Acquire) {
            0 => ModuleState::Unloaded,
            1 => ModuleState::Loading,
            2 => ModuleState::Loaded,
            3 => ModuleState::Initializing,
            4 => ModuleState::Live,
            5 => ModuleState::Coming,
            6 => ModuleState::Going,
            _ => ModuleState::Unloaded,
        }
    }
    
    /// Check if live
    pub fn is_live(&self) -> bool {
        self.get_state() == ModuleState::Live
    }
}

/// Module Manager
pub struct ModuleManager {
    /// Module list
    pub modules: *mut Module,
    /// Module count
    pub module_count: AtomicU32,
    /// Next module ID
    pub next_id: AtomicU32,
    /// Kernel tainted
    pub tainted: AtomicU64,
    /// Loading module
    pub loading: AtomicBool,
    /// Statistics
    pub stats: ModStats,
}

/// Module Statistics
pub struct ModStats {
    pub loaded: AtomicU64,
    pub unloaded: AtomicU64,
    pub failed: AtomicU64,
}

impl ModStats {
    pub const fn new() -> Self {
        ModStats {
            loaded: AtomicU64::new(0),
            unloaded: AtomicU64::new(0),
            failed: AtomicU64::new(0),
        }
    }
}

impl ModuleManager {
    pub const fn new() -> Self {
        ModuleManager {
            modules: core::ptr::null_mut(),
            module_count: AtomicU32::new(0),
            next_id: AtomicU32::new(1),
            tainted: AtomicU64::new(0),
            loading: AtomicBool::new(false),
            stats: ModStats::new(),
        }
    }
    
    /// Initialize
    pub fn init(&self) {
        log_info!("Module manager initialized");
    }
    
    /// Load module from file
    pub fn load_module(&mut self, path: &[u8]) -> Result<ModuleId, i32> {
        // Check if already loading
        if self.loading.swap(true, Ordering::AcqRel) {
            return Err(-16); // EBUSY
        }
        
        // Read module file
        let data = self.read_module_file(path)?;
        
        // Parse ELF
        let module = self.parse_module(&data)?;
        
        // Allocate module ID
        let id = self.next_id.fetch_add(1, Ordering::AcqRel);
        
        // Add to list
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*module).id = id;
            (*module).next = self.modules;
            self.modules = module;
        }
        
        self.module_count.fetch_add(1, Ordering::AcqRel);
        self.loading.store(false, Ordering::Release);
        
        Ok(id)
    }
    
    /// Read module file
    fn read_module_file(&self, path: &[u8]) -> Result<&'static [u8], i32> {
        // TODO: Read from filesystem
        let _ = path;
        Err(-2) // ENOENT
    }
    
    /// Parse module ELF
    fn parse_module(&mut self, data: &[u8]) -> Result<*mut Module, i32> {
        // TODO: Parse ELF format
        let _ = data;
        Err(-22) // EINVAL
    }
    
    /// Initialize module
    pub fn init_module(&mut self, id: ModuleId) -> Result<(), i32> {
        let module = self.find_module(id).ok_or(-22)?;
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Set state to initializing
            (*module).state.store(ModuleState::Initializing as u32, Ordering::Release);
            
            // Call init function
            if let Some(init) = (*module).ops.init {
                let ret = init();
                if ret != 0 {
                    (*module).state.store(ModuleState::Loaded as u32, Ordering::Release);
                    self.stats.failed.fetch_add(1, Ordering::AcqRel);
                    return Err(ret);
                }
            }
            
            // Set state to live
            (*module).state.store(ModuleState::Live as u32, Ordering::Release);
            (*module).flags.fetch_or(ModFlags::INIT_OK.bits(), Ordering::AcqRel);
        }
        
        self.stats.loaded.fetch_add(1, Ordering::AcqRel);
        Ok(())
    }
    
    /// Unload module
    pub fn unload_module(&mut self, id: ModuleId) -> Result<(), i32> {
        let module = self.find_module(id).ok_or(-22)?;
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Check if live
            if !(*module).is_live() {
                return Err(-22);
            }
            
            // Check ref count
            if (*module).ref_count.load(Ordering::Acquire) > 1 {
                return Err(-16); // EBUSY
            }
            
            // Set state to going
            (*module).state.store(ModuleState::Going as u32, Ordering::Release);
            
            // Call exit function
            if let Some(exit) = (*module).ops.exit {
                exit();
            }
            
            // Set state to unloaded
            (*module).state.store(ModuleState::Unloaded as u32, Ordering::Release);
        }
        
        // Remove from list
        self.remove_module(id);
        
        self.module_count.fetch_sub(1, Ordering::AcqRel);
        self.stats.unloaded.fetch_add(1, Ordering::AcqRel);
        
        Ok(())
    }
    
    /// Find module by ID
    pub fn find_module(&self, id: ModuleId) -> Option<*mut Module> {
        let mut module = self.modules;
        
        while !module.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*module).id == id {
                    return Some(module);
                }
                module = (*module).next;
            }
        }
        
        None
    }
    
    /// Find module by name
    pub fn find_module_by_name(&self, name: &[u8]) -> Option<*mut Module> {
        let mut module = self.modules;
        
        while !module.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let mod_name = &(*module).info.name;
                if mod_name[..name.len()] == *name {
                    return Some(module);
                }
                module = (*module).next;
            }
        }
        
        None
    }
    
    /// Remove module from list
    fn remove_module(&mut self, id: ModuleId) {
        let mut prev: *mut Module = core::ptr::null_mut();
        let mut module = self.modules;
        
        while !module.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*module).id == id {
                    if prev.is_null() {
                        self.modules = (*module).next;
                    } else {
                        (*prev).next = (*module).next;
                    }
                    return;
                }
                prev = module;
                module = (*module).next;
            }
        }
    }
    
    /// Get module count
    pub fn count(&self) -> u32 {
        self.module_count.load(Ordering::Acquire)
    }
    
    /// List modules
    pub fn list_modules(&self) {
        log_info!("Loaded modules:");
        
        let mut module = self.modules;
        while !module.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let name = core::str::from_utf8_unchecked(&(*module).info.name);
                let state = match (*module).get_state() {
                    ModuleState::Live => "Live",
                    ModuleState::Loaded => "Loaded",
                    _ => "Other",
                };
                log_info!("  {} [{}]", name, state);
                module = (*module).next;
            }
        }
    }
    
    /// Add taint
    pub fn add_taint(&mut self, taint: u64) {
        self.tainted.fetch_or(taint, Ordering::AcqRel);
    }
    
    /// Check taint
    pub fn tainted(&self, taint: u64) -> bool {
        (self.tainted.load(Ordering::Acquire) & taint) != 0
    }
}

/// Global module manager
static MODULE_MANAGER: crate::sync_oncelock::OnceLock<ModuleManager> = crate::sync_oncelock::OnceLock::new();

/// Get module manager
pub fn module_manager() -> &'static ModuleManager {
    MODULE_MANAGER.get_or_init(ModuleManager::new)
}

pub fn init_module_manager() -> &'static ModuleManager {
    MODULE_MANAGER.get_or_init(ModuleManager::new)
}

/// Initialize module manager
pub fn init_module() {
    let mgr = module_manager();
    mgr.init();
}

// Convenience functions

/// Request module
pub fn request_module(name: &[u8]) -> i32 {
    let mgr = module_manager();
    
    // Check if already loaded
    if mgr.find_module_by_name(name).is_some() {
        return 0;
    }
    
    // Try to load
    match mgr.load_module(name) {
        Ok(id) => {
            let _ = mgr.init_module(id);
            0
        }
        Err(e) => e,
    }
}

/// Release module
pub fn module_put(module: *mut Module) {
    if module.is_null() {
        return;
    }
    
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        (*module).ref_count.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Try to get module
pub fn try_module_get(module: *mut Module) -> bool {
    if module.is_null() {
        return false;
    }
    
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        let state = (*module).get_state();
        if state != ModuleState::Live {
            return false;
        }
        
        (*module).ref_count.fetch_add(1, Ordering::AcqRel);
        true
    }
}

/// Module Export Macro Helper
pub struct ModuleExport {
    pub name: &'static [u8],
    pub value: u64,
}

/// Export symbol
#[macro_export]
macro_rules! EXPORT_SYMBOL {
    ($sym:ident) => {
        #[used]
        #[link_section = ".export_symbols"]
        static __EXPORT_ $sym: $crate::module::ModuleExport = $crate::module::ModuleExport {
            name: stringify!($sym).as_bytes(),
            value: $sym as u64,
        };
    };
}

/// Module init macro
#[macro_export]
macro_rules! module_init {
    ($init_fn:ident) => {
        #[used]
        #[link_section = ".module_init"]
        static __MODULE_INIT: unsafe extern "C" fn() -> i32 = $init_fn;
    };
}

/// Module exit macro
#[macro_export]
macro_rules! module_exit {
    ($exit_fn:ident) => {
        #[used]
        #[link_section = ".module_exit"]
        static __MODULE_EXIT: unsafe extern "C" fn() = $exit_fn;
    };
}
