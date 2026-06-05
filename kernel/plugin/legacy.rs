/*
 * Nuva OS - Kernel - Plugin - Legacy
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
 * Nuva OS - Kernel - Plugin System
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Plugin-based device adaptation and kernel feature system.
 * Allows dynamic registration of drivers and features with
 * differential configuration to reduce system overhead.
 */

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicPtr, Ordering};
use crate::{pr_info};

use crate::posix::errno::Errno;
/// Plugin ID
pub type PluginId = u64;

/// Plugin Type
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginType {
    /// Driver plugin
    Driver = 0,
    /// Feature plugin
    Feature = 1,
    /// Filesystem plugin
    Filesystem = 2,
    /// Network protocol plugin
    Network = 3,
    /// Security plugin
    Security = 4,
    /// Power management plugin
    Power = 5,
    /// Debug plugin
    Debug = 6,
    /// Platform plugin
    Platform = 7,
    /// Extension plugin
    Extension = 8,
}

/// Plugin State
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    /// Unregistered
    Unregistered = 0,
    /// Registered
    Registered = 1,
    /// Loading
    Loading = 2,
    /// Loaded
    Loaded = 3,
    /// Initializing
    Initializing = 4,
    /// Active
    Active = 5,
    /// Deactivating
    Deactivating = 6,
    /// Unloading
    Unloading = 7,
    /// Failed
    Failed = 8,
    /// Disabled
    Disabled = 9,
}

/// Plugin Priority
pub const PLUGIN_PRIORITY_HIGHEST: i32 = 1000;
pub const PLUGIN_PRIORITY_HIGH: i32 = 100;
pub const PLUGIN_PRIORITY_NORMAL: i32 = 0;
pub const PLUGIN_PRIORITY_LOW: i32 = -100;
pub const PLUGIN_PRIORITY_LOWEST: i32 = -1000;

/// Plugin Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct PluginFlags: u32 {
        /// Built-in plugin
        const BUILTIN = 1 << 0;
        /// Auto-load
        const AUTO_LOAD = 1 << 1;
        /// Auto-activate
        const AUTO_ACTIVATE = 1 << 2;
        /// Hot-pluggable
        const HOTPLUG = 1 << 3;
        /// Critical (cannot unload)
        const CRITICAL = 1 << 4;
        /// Experimental
        const EXPERIMENTAL = 1 << 5;
        /// Debug only
        const DEBUG_ONLY = 1 << 6;
        /// Platform specific
        const PLATFORM_SPECIFIC = 1 << 7;
        /// Device specific
        const DEVICE_SPECIFIC = 1 << 8;
        /// Lazy load
        const LAZY_LOAD = 1 << 9;
        /// Singleton (only one instance)
        const SINGLETON = 1 << 10;
    }
}

/// Plugin Dependency
#[repr(C)]
pub struct PluginDependency {
    /// Dependency name
    pub name: [u8; 64],
    /// Minimum version
    pub min_version: u32,
    /// Maximum version
    pub max_version: u32,
    /// Required (vs optional)
    pub required: bool,
    /// Next dependency
    pub next: *mut PluginDependency,
}

/// Plugin Info
#[repr(C)]
pub struct PluginInfo {
    /// Plugin name
    pub name: [u8; 64],
    /// Plugin version
    pub version: u32,
    /// Plugin author
    pub author: [u8; 64],
    /// Plugin description
    pub description: [u8; 256],
    /// Plugin license
    pub license: [u8; 32],
    /// Plugin type
    pub plugin_type: PluginType,
    /// Priority
    pub priority: i32,
    /// Flags
    pub flags: PluginFlags,
    /// Dependencies
    pub dependencies: *mut PluginDependency,
    /// Compatible devices
    pub compatible: [u8; 512],
    /// Config schema
    pub config_schema: *const u8,
    /// Config schema size
    pub config_schema_size: u32,
}

/// Plugin Operations
pub struct PluginOps {
    /// Probe for device support
    pub probe: Option<unsafe extern "C" fn(*const core::ffi::c_void) -> i32>,
    /// Initialize plugin
    pub init: Option<unsafe extern "C" fn(*mut Plugin) -> i32>,
    /// Activate plugin
    pub activate: Option<unsafe extern "C" fn(*mut Plugin) -> i32>,
    /// Deactivate plugin
    pub deactivate: Option<unsafe extern "C" fn(*mut Plugin) -> i32>,
    /// Cleanup plugin
    pub cleanup: Option<unsafe extern "C" fn(*mut Plugin)>,
    /// Suspend plugin
    pub suspend: Option<unsafe extern "C" fn(*mut Plugin) -> i32>,
    /// Resume plugin
    pub resume: Option<unsafe extern "C" fn(*mut Plugin) -> i32>,
    /// Handle event
    pub handle_event: Option<unsafe extern "C" fn(*mut Plugin, u32, *mut core::ffi::c_void) -> i32>,
    /// Get config
    pub get_config: Option<unsafe extern "C" fn(*mut Plugin, *mut u8, usize) -> i32>,
    /// Set config
    pub set_config: Option<unsafe extern "C" fn(*mut Plugin, *const u8, usize) -> i32>,
}

/// Plugin Configuration
pub struct PluginConfig {
    /// Config data
    pub data: *mut u8,
    /// Config size
    pub size: usize,
    /// Config version
    pub version: u32,
    /// Dirty flag
    pub dirty: AtomicBool,
}

/// Plugin Statistics
pub struct PluginStats {
    /// Load count
    pub load_count: AtomicU64,
    /// Activate count
    pub activate_count: AtomicU64,
    /// Event count
    pub event_count: AtomicU64,
    /// Error count
    pub error_count: AtomicU64,
    /// Total active time
    pub active_time: AtomicU64,
    /// Last activate time
    pub last_activate: AtomicU64,
}

impl PluginStats {
    pub const fn new() -> Self {
        PluginStats {
            load_count: AtomicU64::new(0),
            activate_count: AtomicU64::new(0),
            event_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
            active_time: AtomicU64::new(0),
            last_activate: AtomicU64::new(0),
        }
    }
}

/// Plugin Structure
pub struct Plugin {
    /// Plugin ID
    pub id: PluginId,
    /// Plugin info
    pub info: PluginInfo,
    /// Operations
    pub ops: PluginOps,
    /// State
    pub state: AtomicU32,
    /// Configuration
    pub config: PluginConfig,
    /// Private data
    pub priv_data: *mut core::ffi::c_void,
    /// Statistics
    pub stats: PluginStats,
    /// Reference count
    pub ref_count: AtomicU32,
    /// Next plugin
    pub next: *mut Plugin,
}

impl Plugin {
    pub fn new(id: PluginId, name: &[u8], plugin_type: PluginType) -> Self {
        let mut name_arr = [0u8; 64];
        let len = name.len().min(63);
        name_arr[..len].copy_from_slice(&name[..len]);
        
        Plugin {
            id,
            info: PluginInfo {
                name: name_arr,
                version: 1,
                author: [0; 64],
                description: [0; 256],
                license: [0; 32],
                plugin_type,
                priority: PLUGIN_PRIORITY_NORMAL,
                flags: PluginFlags::empty(),
                dependencies: core::ptr::null_mut(),
                compatible: [0; 512],
                config_schema: core::ptr::null(),
                config_schema_size: 0,
            },
            ops: PluginOps {
                probe: None,
                init: None,
                activate: None,
                deactivate: None,
                cleanup: None,
                suspend: None,
                resume: None,
                handle_event: None,
                get_config: None,
                set_config: None,
            },
            state: AtomicU32::new(PluginState::Unregistered as u32),
            config: PluginConfig {
                data: core::ptr::null_mut(),
                size: 0,
                version: 0,
                dirty: AtomicBool::new(false),
            },
            priv_data: core::ptr::null_mut(),
            stats: PluginStats::new(),
            ref_count: AtomicU32::new(1),
            next: core::ptr::null_mut(),
        }
    }
    
    /// Get state
    pub fn get_state(&self) -> PluginState {
        match self.state.load(Ordering::Acquire) {
            0 => PluginState::Unregistered,
            1 => PluginState::Registered,
            2 => PluginState::Loading,
            3 => PluginState::Loaded,
            4 => PluginState::Initializing,
            5 => PluginState::Active,
            6 => PluginState::Deactivating,
            7 => PluginState::Unloading,
            8 => PluginState::Failed,
            9 => PluginState::Disabled,
            _ => PluginState::Unregistered,
        }
    }
    
    /// Set state
    pub fn set_state(&self, state: PluginState) {
        self.state.store(state as u32, Ordering::Release);
    }
    
    /// Check if active
    pub fn is_active(&self) -> bool {
        self.get_state() == PluginState::Active
    }
    
    /// Check if loaded
    pub fn is_loaded(&self) -> bool {
        matches!(self.get_state(), PluginState::Loaded | PluginState::Active)
    }
    
    /// Get reference
    pub fn get(&self) {
        self.ref_count.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Put reference
    pub fn put(&self) -> u32 {
        self.ref_count.fetch_sub(1, Ordering::AcqRel)
    }
    
    /// Probe for support
    pub fn probe(&self, device: *const core::ffi::c_void) -> i32 {
        if let Some(probe) = self.ops.probe {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { probe(device) }
        } else {
            0
        }
    }
    
    /// Initialize
    pub fn init(&self) -> i32 {
        if self.is_loaded() {
            return 0;
        }
        
        self.set_state(PluginState::Initializing);
        
        if let Some(init) = self.ops.init {
            // SAFETY: unsafe block required for low-level memory or hardware access
            let ret = unsafe { init(self as *mut Plugin) };
            if ret != 0 {
                self.set_state(PluginState::Failed);
                self.stats.error_count.fetch_add(1, Ordering::AcqRel);
                return ret;
            }
        }
        
        self.set_state(PluginState::Loaded);
        self.stats.load_count.fetch_add(1, Ordering::AcqRel);
        0
    }
    
    /// Activate
    pub fn activate(&mut self) -> i32 {
        if self.is_active() {
            return 0;
        }
        
        if !self.is_loaded() {
            let ret = self.init();
            if ret != 0 {
                return ret;
            }
        }
        
        if let Some(activate) = self.ops.activate {
            // SAFETY: unsafe block required for low-level memory or hardware access
            let ret = unsafe { activate(self as *mut Plugin) };
            if ret != 0 {
                self.set_state(PluginState::Failed);
                self.stats.error_count.fetch_add(1, Ordering::AcqRel);
                return ret;
            }
        }
        
        self.set_state(PluginState::Active);
        self.stats.activate_count.fetch_add(1, Ordering::AcqRel);
        0
    }
    
    /// Deactivate
    pub fn deactivate(&mut self) -> i32 {
        if !self.is_active() {
            return 0;
        }
        
        // Check if critical
        if self.info.flags.contains(PluginFlags::CRITICAL) {
            return Errno::Ebusy.to_ret_i32(); // EBUSY
        }
        
        // Check ref count
        if self.ref_count.load(Ordering::Acquire) > 1 {
            return Errno::Ebusy.to_ret_i32(); // EBUSY
        }
        
        self.set_state(PluginState::Deactivating);
        
        if let Some(deactivate) = self.ops.deactivate {
            // SAFETY: unsafe block required for low-level memory or hardware access
            let ret = unsafe { deactivate(self as *mut Plugin) };
            if ret != 0 {
                self.set_state(PluginState::Active);
                self.stats.error_count.fetch_add(1, Ordering::AcqRel);
                return ret;
            }
        }
        
        self.set_state(PluginState::Loaded);
        0
    }
    
    /// Cleanup
    pub fn cleanup(&mut self) {
        if self.is_active() {
            let _ = self.deactivate();
        }
        
        if let Some(cleanup) = self.ops.cleanup {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { cleanup(self as *mut Plugin); }
        }
        
        self.set_state(PluginState::Unregistered);
    }
    
    /// Handle event
    pub fn handle_event(&mut self, event: u32, data: *mut core::ffi::c_void) -> i32 {
        if !self.is_active() {
            return Errno::Eperm.to_ret_i32();
        }
        
        self.stats.event_count.fetch_add(1, Ordering::AcqRel);
        
        if let Some(handle) = self.ops.handle_event {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { handle(self as *mut Plugin, event, data) }
        } else {
            0
        }
    }
    
    /// Suspend
    pub fn suspend(&mut self) -> i32 {
        if !self.is_active() {
            return 0;
        }
        
        if let Some(suspend) = self.ops.suspend {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { suspend(self as *mut Plugin) }
        } else {
            0
        }
    }
    
    /// Resume
    pub fn resume(&mut self) -> i32 {
        if self.get_state() != PluginState::Active {
            return 0;
        }
        
        if let Some(resume) = self.ops.resume {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { resume(self as *mut Plugin) }
        } else {
            0
        }
    }
}

/// Plugin Category
pub struct PluginCategory {
    /// Category name
    pub name: [u8; 32],
    /// Category type
    pub category_type: PluginType,
    /// Plugins in category
    pub plugins: *mut Plugin,
    /// Plugin count
    pub count: AtomicU32,
    /// Active count
    pub active_count: AtomicU32,
    /// Next category
    pub next: *mut PluginCategory,
}

impl PluginCategory {
    pub fn new(name: &[u8], category_type: PluginType) -> Self {
        let mut name_arr = [0u8; 32];
        let len = name.len().min(31);
        name_arr[..len].copy_from_slice(&name[..len]);
        
        PluginCategory {
            name: name_arr,
            category_type,
            plugins: core::ptr::null_mut(),
            count: AtomicU32::new(0),
            active_count: AtomicU32::new(0),
            next: core::ptr::null_mut(),
        }
    }
}

/// Plugin Manager
pub struct PluginManager {
    /// All plugins
    pub plugins: *mut Plugin,
    /// Plugin count
    pub plugin_count: AtomicU32,
    /// Categories
    pub categories: *mut PluginCategory,
    /// Category count
    pub category_count: AtomicU32,
    /// Next plugin ID
    pub next_id: AtomicU64,
    /// Auto-load enabled
    pub auto_load: AtomicBool,
    /// Lazy load enabled
    pub lazy_load: AtomicBool,
    /// Statistics
    pub stats: PluginMgrStats,
    /// Lock
    pub lock: AtomicU32,
}

/// Plugin Manager Statistics
pub struct PluginMgrStats {
    pub total_plugins: AtomicU64,
    pub active_plugins: AtomicU64,
    pub failed_plugins: AtomicU64,
    pub total_loads: AtomicU64,
    pub total_activations: AtomicU64,
}

impl PluginMgrStats {
    pub const fn new() -> Self {
        PluginMgrStats {
            total_plugins: AtomicU64::new(0),
            active_plugins: AtomicU64::new(0),
            failed_plugins: AtomicU64::new(0),
            total_loads: AtomicU64::new(0),
            total_activations: AtomicU64::new(0),
        }
    }
}

impl PluginManager {
    pub const fn new() -> Self {
        PluginManager {
            plugins: core::ptr::null_mut(),
            plugin_count: AtomicU32::new(0),
            categories: core::ptr::null_mut(),
            category_count: AtomicU32::new(0),
            next_id: AtomicU64::new(1),
            auto_load: AtomicBool::new(true),
            lazy_load: AtomicBool::new(true),
            stats: PluginMgrStats::new(),
            lock: AtomicU32::new(0),
        }
    }
    
    /// Lock
    fn lock(&self) {
        while self.lock.compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire).is_err() {
            core::hint::spin_loop();
        }
    }
    
    /// Unlock
    fn unlock(&self) {
        self.lock.store(0, Ordering::Release);
    }
    
    /// Initialize
    pub fn init(&self) {
        // Register built-in categories
        self.register_builtin_categories();
        
        log_info!("Plugin manager initialized");
    }
    
    /// Register built-in categories
    fn register_builtin_categories(&mut self) {
        self.add_category(b"driver", PluginType::Driver);
        self.add_category(b"feature", PluginType::Feature);
        self.add_category(b"filesystem", PluginType::Filesystem);
        self.add_category(b"network", PluginType::Network);
        self.add_category(b"security", PluginType::Security);
        self.add_category(b"power", PluginType::Power);
        self.add_category(b"debug", PluginType::Debug);
        self.add_category(b"platform", PluginType::Platform);
        self.add_category(b"extension", PluginType::Extension);
    }
    
    /// Add category
    fn add_category(&mut self, name: &[u8], category_type: PluginType) {
        // TODO: Allocate and add category
        let _ = (name, category_type);
        self.category_count.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Register plugin
    pub fn register(&mut self, plugin: *mut Plugin) -> Result<PluginId, i32> {
        if plugin.is_null() {
            return Err(-22);
        }
        
        self.lock();
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Assign ID
            (*plugin).id = self.next_id.fetch_add(1, Ordering::AcqRel);
            (*plugin).set_state(PluginState::Registered);
            
            // Add to plugin list
            (*plugin).next = self.plugins;
            self.plugins = plugin;
            
            // Add to category
            self.add_to_category(plugin);
        }
        
        self.plugin_count.fetch_add(1, Ordering::AcqRel);
        self.stats.total_plugins.fetch_add(1, Ordering::AcqRel);
        
        self.unlock();
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { Ok((*plugin).id) }
    }
    
    /// Add plugin to category
    fn add_to_category(&mut self, plugin: *mut Plugin) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let plugin_type = (*plugin).info.plugin_type;
            
            let mut category = self.categories;
            while !category.is_null() {
                if (*category).category_type == plugin_type {
                    (*plugin).next = (*category).plugins;
                    (*category).plugins = plugin;
                    (*category).count.fetch_add(1, Ordering::AcqRel);
                    return;
                }
                category = (*category).next;
            }
        }
    }
    
    /// Unregister plugin
    pub fn unregister(&mut self, id: PluginId) -> i32 {
        let plugin = match self.find_plugin(id) {
            Some(p) => p,
            None => return Errno::Enoent.to_ret_i32(), // ENOENT
        };
        
        self.lock();
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Check if can unregister
            if (*plugin).info.flags.contains(PluginFlags::CRITICAL) {
                self.unlock();
                return Errno::Ebusy.to_ret_i32();
            }
            
            if (*plugin).ref_count.load(Ordering::Acquire) > 1 {
                self.unlock();
                return Errno::Ebusy.to_ret_i32();
            }
            
            // Cleanup
            (*plugin).cleanup();
        }
        
        // Remove from lists
        self.remove_plugin(id);
        
        self.plugin_count.fetch_sub(1, Ordering::AcqRel);
        self.unlock();
        
        0
    }
    
    /// Remove plugin from lists
    fn remove_plugin(&mut self, id: PluginId) {
        // Remove from main list
        let mut prev: *mut Plugin = core::ptr::null_mut();
        let mut plugin = self.plugins;
        
        while !plugin.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*plugin).id == id {
                    if prev.is_null() {
                        self.plugins = (*plugin).next;
                    } else {
                        (*prev).next = (*plugin).next;
                    }
                    break;
                }
                prev = plugin;
                plugin = (*plugin).next;
            }
        }
        
        // Remove from category
        // TODO: Remove from category list
    }
    
    /// Find plugin by ID
    pub fn find_plugin(&self, id: PluginId) -> Option<*mut Plugin> {
        let mut plugin = self.plugins;
        
        while !plugin.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*plugin).id == id {
                    return Some(plugin);
                }
                plugin = (*plugin).next;
            }
        }
        
        None
    }
    
    /// Find plugin by name
    pub fn find_plugin_by_name(&self, name: &[u8]) -> Option<*mut Plugin> {
        let mut plugin = self.plugins;
        
        while !plugin.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let plugin_name = &(*plugin).info.name;
                if plugin_name[..name.len()] == *name {
                    return Some(plugin);
                }
                plugin = (*plugin).next;
            }
        }
        
        None
    }
    
    /// Load plugin
    pub fn load(&mut self, id: PluginId) -> i32 {
        let plugin = match self.find_plugin(id) {
            Some(p) => p,
            None => return Errno::Enoent.to_ret_i32(), // ENOENT
        };
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let ret = (*plugin).init();
            if ret == 0 {
                self.stats.total_loads.fetch_add(1, Ordering::AcqRel);
            }
            ret
        }
    }
    
    /// Activate plugin
    pub fn activate(&mut self, id: PluginId) -> i32 {
        let plugin = match self.find_plugin(id) {
            Some(p) => p,
            None => return Errno::Enoent.to_ret_i32(), // ENOENT
        };
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Check dependencies
            let ret = self.check_dependencies((*plugin).info.dependencies);
            if ret != 0 {
                return ret;
            }
            
            let ret = (*plugin).activate();
            if ret == 0 {
                self.stats.active_plugins.fetch_add(1, Ordering::AcqRel);
                self.stats.total_activations.fetch_add(1, Ordering::AcqRel);
            }
            ret
        }
    }
    
    /// Deactivate plugin
    pub fn deactivate(&mut self, id: PluginId) -> i32 {
        let plugin = match self.find_plugin(id) {
            Some(p) => p,
            None => return Errno::Enoent.to_ret_i32(), // ENOENT
        };
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Check if other plugins depend on this
            if self.has_dependents(id) {
                return Errno::Ebusy.to_ret_i32();
            }
            
            let ret = (*plugin).deactivate();
            if ret == 0 {
                self.stats.active_plugins.fetch_sub(1, Ordering::AcqRel);
            }
            ret
        }
    }
    
    /// Check dependencies
    fn check_dependencies(&self, deps: *mut PluginDependency) -> i32 {
        let mut dep = deps;
        
        while !dep.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let name = &(*dep).name;
                let len = name.iter().position(|&c| c == 0).unwrap_or(64);
                
                if let Some(plugin) = self.find_plugin_by_name(&name[..len]) {
                    if !(*plugin).is_active() {
                        if (*dep).required {
                            return Errno::Eexist.to_ret_i32(); // EDEPNOTSAT
                        }
                    }
                } else if (*dep).required {
                    return Errno::Enoent.to_ret_i32(); // ENOENT
                }
                
                dep = (*dep).next;
            }
        }
        
        0
    }
    
    /// Check if plugin has dependents
    fn has_dependents(&self, id: PluginId) -> bool {
        let mut plugin = self.plugins;
        
        while !plugin.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let mut dep = (*plugin).info.dependencies;
                
                while !dep.is_null() {
                    let name = &(*dep).name;
                    let len = name.iter().position(|&c| c == 0).unwrap_or(64);
                    
                    if let Some(dep_plugin) = self.find_plugin_by_name(&name[..len]) {
                        if (*dep_plugin).id == id && (*plugin).is_active() {
                            return true;
                        }
                    }
                    
                    dep = (*dep).next;
                }
                
                plugin = (*plugin).next;
            }
        }
        
        false
    }
    
    /// Probe device
    pub fn probe_device(&self, device: *const core::ffi::c_void, plugin_type: PluginType) -> Option<*mut Plugin> {
        let mut best_plugin: *mut Plugin = core::ptr::null_mut();
        let mut best_score: i32 = -1;
        
        let mut plugin = self.plugins;
        
        while !plugin.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*plugin).info.plugin_type == plugin_type {
                    let score = (*plugin).probe(device);
                    if score > best_score {
                        best_score = score;
                        best_plugin = plugin;
                    }
                }
                plugin = (*plugin).next;
            }
        }
        
        if best_plugin.is_null() {
            None
        } else {
            Some(best_plugin)
        }
    }
    
    /// Auto-load plugins for device
    pub fn autoload_for_device(&mut self, device: *const core::ffi::c_void) {
        if !self.auto_load.load(Ordering::Acquire) {
            return;
        }
        
        // Probe all driver plugins
        if let Some(plugin) = self.probe_device(device, PluginType::Driver) {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*plugin).info.flags.contains(PluginFlags::AUTO_LOAD) {
                    let id = (*plugin).id;
                    let _ = self.activate(id);
                }
            }
        }
    }
    
    /// Get plugins by type
    pub fn get_plugins_by_type(&self, plugin_type: PluginType) -> PluginIterator {
        PluginIterator {
            current: self.plugins,
            filter_type: Some(plugin_type),
        }
    }
    
    /// Get all plugins
    pub fn get_all_plugins(&self) -> PluginIterator {
        PluginIterator {
            current: self.plugins,
            filter_type: None,
        }
    }
    
    /// Get active count
    pub fn active_count(&self) -> u64 {
        self.stats.active_plugins.load(Ordering::Acquire)
    }
    
    /// Get total count
    pub fn total_count(&self) -> u64 {
        self.stats.total_plugins.load(Ordering::Acquire)
    }
    
    /// List plugins
    pub fn list_plugins(&self) {
        log_info!("Registered plugins:");
        
        let mut plugin = self.plugins;
        while !plugin.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let name = core::str::from_utf8_unchecked(&(*plugin).info.name);
                let state = match (*plugin).get_state() {
                    PluginState::Active => "Active",
                    PluginState::Loaded => "Loaded",
                    PluginState::Registered => "Registered",
                    PluginState::Failed => "Failed",
                    PluginState::Disabled => "Disabled",
                    _ => "Other",
                };
                log_info!("  {} [{}]", name, state);
                plugin = (*plugin).next;
            }
        }
    }
}

/// Plugin Iterator
pub struct PluginIterator {
    current: *mut Plugin,
    filter_type: Option<PluginType>,
}

impl PluginIterator {
    pub fn next(&mut self) -> Option<*mut Plugin> {
        while !self.current.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let plugin = self.current;
                self.current = (*plugin).next;
                
                if let Some(filter_type) = self.filter_type {
                    if (*plugin).info.plugin_type != filter_type {
                        continue;
                    }
                }
                
                return Some(plugin);
            }
        }
        
        None
    }
}

/// Global plugin manager
static PLUGIN_MANAGER: core::sync::OnceLock<PluginManager> = core::sync::OnceLock::new();

/// Get plugin manager
pub fn plugin_manager() -> &'static PluginManager {
    PLUGIN_MANAGER.get_or_init(PluginManager::new)
}

pub fn init_plugin_manager() -> &'static PluginManager {
    PLUGIN_MANAGER.get_or_init(PluginManager::new)
}

/// Initialize plugin system
pub fn init_plugin() {
    let mgr = plugin_manager();
    mgr.init();
}

// Convenience functions

/// Register plugin
pub fn plugin_register(plugin: *mut Plugin) -> Result<PluginId, i32> {
    plugin_manager().register(plugin)
}

/// Find plugin by name
pub fn plugin_find(name: &[u8]) -> Option<*mut Plugin> {
    plugin_manager().find_plugin_by_name(name)
}

/// Activate plugin
pub fn plugin_activate(name: &[u8]) -> i32 {
    let mgr = plugin_manager();
    if let Some(plugin) = mgr.find_plugin_by_name(name) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { mgr.activate((*plugin).id) }
    } else {
        -2
    }
}

/// Deactivate plugin
pub fn plugin_deactivate(name: &[u8]) -> i32 {
    let mgr = plugin_manager();
    if let Some(plugin) = mgr.find_plugin_by_name(name) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe { mgr.deactivate((*plugin).id) }
    } else {
        -2
    }
}

/// Probe device for best matching plugin
pub fn plugin_probe(device: *const core::ffi::c_void, plugin_type: PluginType) -> Option<*mut Plugin> {
    plugin_manager().probe_device(device, plugin_type)
}

/// Plugin Registration Macro
#[macro_export]
macro_rules! register_plugin {
    ($name:ident, $type:expr, $init:expr) => {
        static mut $name: $crate::plugin::Plugin = {
            let mut plugin = $crate::plugin::Plugin::new(
                0,
                stringify!($name).as_bytes(),
                $type,
            );
            plugin.ops.init = Some($init);
            plugin
        };
        
        #[used]
        #[link_section = ".plugins"]
        static __PLUGIN_REG_ $name: unsafe extern "C" fn() = || {
            // SAFETY: unsafe block required for low-level memory or hardware access
            let _ = $crate::plugin::plugin_register(unsafe { &mut $name as *mut _ });
        };
    };
}
