/*
 * Nuva OS - Kernel - Device - FeaturePlugin
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
 * Nuva OS - Kernel - Feature Plugin Interface
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Feature plugin interface for kernel functionality extension.
 */

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use crate::kernel::plugin::{Plugin, PluginType, PluginFlags, PluginOps, PluginInfo};
use crate::kernel::plugin::core::PluginMeta;

use crate::posix::errno::Errno;
/// Feature Category
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureCategory {
    /// Core kernel feature
    Core = 0,
    /// Memory management
    Memory = 1,
    /// Scheduler
    Scheduler = 2,
    /// Filesystem
    Filesystem = 3,
    /// Network
    Network = 4,
    /// Security
    Security = 5,
    /// Power
    Power = 6,
    /// Debug
    Debug = 7,
    /// Performance
    Performance = 8,
    /// Hardware
    Hardware = 9,
    /// Virtualization
    Virtualization = 10,
    /// User interface
    Ui = 11,
}

/// Feature Hook Point
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookPoint {
    /// Before kernel init
    PreInit = 0,
    /// During kernel init
    Init = 1,
    /// After kernel init
    PostInit = 2,
    /// Before device probe
    PreProbe = 3,
    /// After device probe
    PostProbe = 4,
    /// Before device remove
    PreRemove = 5,
    /// After device remove
    PostRemove = 6,
    /// Before suspend
    PreSuspend = 7,
    /// After suspend
    PostSuspend = 8,
    /// Before resume
    PreResume = 9,
    /// After resume
    PostResume = 10,
    /// Before shutdown
    PreShutdown = 11,
    /// After shutdown
    PostShutdown = 12,
    /// Before fork
    PreFork = 13,
    /// After fork
    PostFork = 14,
    /// Before exec
    PreExec = 15,
    /// After exec
    PostExec = 16,
    /// Before exit
    PreExit = 17,
    /// After exit
    PostExit = 18,
    /// Custom hook
    Custom = 100,
}

/// Feature Hook
pub struct FeatureHook {
    /// Hook point
    pub point: HookPoint,
    /// Hook function
    pub func: unsafe extern "C" fn(*mut FeaturePlugin, *mut core::ffi::c_void) -> i32,
    /// Priority
    pub priority: i32,
    /// Enabled
    pub enabled: AtomicBool,
    /// Next hook
    pub next: *mut FeatureHook,
}

/// Feature Operations
pub struct FeatureOps {
    /// Enable feature
    pub enable: Option<unsafe extern "C" fn(*mut FeaturePlugin) -> i32>,
    /// Disable feature
    pub disable: Option<unsafe extern "C" fn(*mut FeaturePlugin) -> i32>,
    /// Configure feature
    pub configure: Option<unsafe extern "C" fn(*mut FeaturePlugin, u32, *const u8, usize) -> i32>,
    /// Query feature
    pub query: Option<unsafe extern "C" fn(*mut FeaturePlugin, u32, *mut u8, usize) -> i32>,
    /// Handle notification
    pub notify: Option<unsafe extern "C" fn(*mut FeaturePlugin, u32, *mut core::ffi::c_void) -> i32>,
}

/// Feature Capabilities
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct FeatureCaps: u64 {
        /// Can be disabled
        const DISABLEABLE = 1 << 0;
        /// Can be reconfigured
        const RECONFIGURABLE = 1 << 1;
        /// Supports hot enable/disable
        const HOT_TOGGLE = 1 << 2;
        /// Requires reboot
        const REBOOT_REQUIRED = 1 << 3;
        /// System critical
        const CRITICAL = 1 << 4;
        /// Performance impact
        const PERF_IMPACT = 1 << 5;
        /// Memory impact
        const MEM_IMPACT = 1 << 6;
        /// Power impact
        const POWER_IMPACT = 1 << 7;
        /// Security impact
        const SECURITY_IMPACT = 1 << 8;
        /// Experimental
        const EXPERIMENTAL = 1 << 9;
        /// Debug only
        const DEBUG_ONLY = 1 << 10;
    }
}

/// Feature Config
pub struct FeatureConfig {
    /// Config ID
    pub id: u32,
    /// Config name
    pub name: [u8; 64],
    /// Config type
    pub config_type: ConfigType,
    /// Default value
    pub default_val: ConfigValue,
    /// Current value
    pub current_val: ConfigValue,
    /// Min value (for range)
    pub min_val: ConfigValue,
    /// Max value (for range)
    pub max_val: ConfigValue,
    /// Description
    pub description: [u8; 256],
    /// Flags
    pub flags: AtomicU32,
    /// Next config
    pub next: *mut FeatureConfig,
}

/// Config Type
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigType {
    Bool = 0,
    Int = 1,
    String = 2,
    Enum = 3,
    Range = 4,
}

/// Config Value
#[repr(C)]
pub union ConfigValue {
    pub bool_val: bool,
    pub int_val: i64,
    pub str_val: [u8; 256],
}

/// Feature Plugin
pub struct FeaturePlugin {
    /// Base plugin
    pub base: PluginMeta,
    /// Feature category
    pub category: FeatureCategory,
    /// Feature operations
    pub feature_ops: FeatureOps,
    /// Capabilities
    pub caps: FeatureCaps,
    /// Hooks
    pub hooks: *mut FeatureHook,
    /// Hook count
    pub hook_count: AtomicU32,
    /// Configs
    pub configs: *mut FeatureConfig,
    /// Config count
    pub config_count: AtomicU32,
    /// Enabled
    pub enabled: AtomicBool,
    /// Usage count
    pub usage_count: AtomicU64,
}

impl FeaturePlugin {
    pub fn new(name: &[u8], category: FeatureCategory) -> Self {
        FeaturePlugin {
            base: PluginMeta::new(0, core::str::from_utf8(name).unwrap_or("")),
            category,
            feature_ops: FeatureOps {
                enable: None,
                disable: None,
                configure: None,
                query: None,
                notify: None,
            },
            caps: FeatureCaps::empty(),
            hooks: core::ptr::null_mut(),
            hook_count: AtomicU32::new(0),
            configs: core::ptr::null_mut(),
            config_count: AtomicU32::new(0),
            enabled: AtomicBool::new(false),
            usage_count: AtomicU64::new(0),
        }
    }
    
    /// Add hook
    pub fn add_hook(&mut self, hook: *mut FeatureHook) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*hook).next = self.hooks;
            self.hooks = hook;
        }
        self.hook_count.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Add config
    pub fn add_config(&mut self, config: *mut FeatureConfig) {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*config).next = self.configs;
            self.configs = config;
        }
        self.config_count.fetch_add(1, Ordering::AcqRel);
    }
    
    /// Enable feature
    pub fn enable(&mut self) -> i32 {
        if self.enabled.load(Ordering::Acquire) {
            return 0;
        }
        
        if let Some(enable) = self.feature_ops.enable {
            // SAFETY: unsafe block required for low-level memory or hardware access
            let ret = unsafe { enable(self as *mut FeaturePlugin) };
            if ret != 0 {
                return ret;
            }
        }
        
        self.enabled.store(true, Ordering::Release);
        0
    }
    
    /// Disable feature
    pub fn disable(&mut self) -> i32 {
        if !self.enabled.load(Ordering::Acquire) {
            return 0;
        }
        
        // Check if can disable
        if self.caps.contains(FeatureCaps::CRITICAL) {
            return Errno::Ebusy.to_ret_i32(); // EBUSY
        }
        
        if !self.caps.contains(FeatureCaps::DISABLEABLE) {
            return Errno::Eopnotsupp.to_ret_i32(); // EOPNOTSUPP
        }
        
        if let Some(disable) = self.feature_ops.disable {
            // SAFETY: unsafe block required for low-level memory or hardware access
            let ret = unsafe { disable(self as *mut FeaturePlugin) };
            if ret != 0 {
                return ret;
            }
        }
        
        self.enabled.store(false, Ordering::Release);
        0
    }
    
    /// Configure
    pub fn configure(&mut self, id: u32, data: *const u8, size: usize) -> i32 {
        if !self.enabled.load(Ordering::Acquire) {
            return Errno::Eperm.to_ret_i32();
        }
        
        if let Some(configure) = self.feature_ops.configure {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { configure(self as *mut FeaturePlugin, id, data, size) }
        } else {
            -95
        }
    }
    
    /// Query
    pub fn query(&self, id: u32, data: *mut u8, size: usize) -> i32 {
        if let Some(query) = self.feature_ops.query {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { query(self as *const FeaturePlugin as *mut FeaturePlugin, id, data, size) }
        } else {
            -95
        }
    }
    
    /// Call hooks
    pub fn call_hooks(&mut self, point: HookPoint, data: *mut core::ffi::c_void) -> i32 {
        let mut hook = self.hooks;
        let mut result = 0;
        
        while !hook.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*hook).point == point && (*hook).enabled.load(Ordering::Acquire) {
                    let ret = ((*hook).func)(self as *mut FeaturePlugin, data);
                    if ret != 0 {
                        result = ret;
                    }
                }
                hook = (*hook).next;
            }
        }
        
        result
    }
    
    /// Get config
    pub fn get_config(&self, id: u32) -> Option<*mut FeatureConfig> {
        let mut config = self.configs;
        
        while !config.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                if (*config).id == id {
                    return Some(config);
                }
                config = (*config).next;
            }
        }
        
        None
    }
    
    /// Set config value
    pub fn set_config(&mut self, id: u32, value: ConfigValue) -> i32 {
        let config = match self.get_config(id) {
            Some(c) => c,
            None => return Errno::Einval.to_ret_i32(), // EINVAL
        };
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*config).current_val = value;
        }
        
        0
    }
}

/// Feature Plugin Registry
pub struct FeaturePluginRegistry {
    /// Features
    pub features: *mut FeaturePlugin,
    /// Feature count
    pub count: AtomicU32,
    /// Enabled count
    pub enabled_count: AtomicU32,
}

impl FeaturePluginRegistry {
    pub const fn new() -> Self {
        FeaturePluginRegistry {
            features: core::ptr::null_mut(),
            count: AtomicU32::new(0),
            enabled_count: AtomicU32::new(0),
        }
    }
    
    /// Register feature
    pub fn register(&mut self, feature: *mut FeaturePlugin) -> i32 {
        if feature.is_null() {
            return Errno::Einval.to_ret_i32();
        }
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            (*feature).base.next = (*self.features).base.next;
            self.features = feature;
        }
        
        self.count.fetch_add(1, Ordering::AcqRel);
        0
    }
    
    /// Find feature by name
    pub fn find(&self, name: &[u8]) -> Option<*mut FeaturePlugin> {
        let mut feature = self.features;
        
        while !feature.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let feature_name = &(*feature).base.info.name;
                if feature_name.as_bytes()[..name.len()] == *name {
                    return Some(feature);
                }
                feature = (*feature).base.next as *mut FeaturePlugin;
            }
        }
        
        None
    }
    
    /// Get features by category
    pub fn get_by_category(&self, category: FeatureCategory) -> FeatureIterator {
        FeatureIterator {
            current: self.features,
            filter_category: Some(category),
        }
    }
    
    /// Call hooks for all features
    pub fn call_hooks(&mut self, point: HookPoint, data: *mut core::ffi::c_void) -> i32 {
        let mut feature = self.features;
        let mut result = 0;
        
        while !feature.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let ret = (*feature).call_hooks(point, data);
                if ret != 0 {
                    result = ret;
                }
                feature = (*feature).base.next as *mut FeaturePlugin;
            }
        }
        
        result
    }
    
    /// Enable feature
    pub fn enable(&mut self, name: &[u8]) -> i32 {
        let feature = match self.find(name) {
            Some(f) => f,
            None => return Errno::Enoent.to_ret_i32(), // ENOENT
        };
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let ret = (*feature).enable();
            if ret == 0 {
                self.enabled_count.fetch_add(1, Ordering::AcqRel);
            }
            ret
        }
    }
    
    /// Disable feature
    pub fn disable(&mut self, name: &[u8]) -> i32 {
        let feature = match self.find(name) {
            Some(f) => f,
            None => return Errno::Enoent.to_ret_i32(), // ENOENT
        };
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let ret = (*feature).disable();
            if ret == 0 {
                self.enabled_count.fetch_sub(1, Ordering::AcqRel);
            }
            ret
        }
    }
}

/// Feature Iterator
pub struct FeatureIterator {
    current: *mut FeaturePlugin,
    filter_category: Option<FeatureCategory>,
}

impl FeatureIterator {
    pub fn next(&mut self) -> Option<*mut FeaturePlugin> {
        while !self.current.is_null() {
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe {
                let feature = self.current;
                self.current = (*feature).base.next as *mut FeaturePlugin;
                
                if let Some(category) = self.filter_category {
                    if (*feature).category != category {
                        continue;
                    }
                }
                
                return Some(feature);
            }
        }
        
        None
    }
}

/// Global feature registry
static FEATURE_REGISTRY: core::sync::OnceLock<FeaturePluginRegistry> = core::sync::OnceLock::new();

/// Get feature registry
pub fn feature_registry() -> &'static FeaturePluginRegistry {
    FEATURE_REGISTRY.get_or_init(FeaturePluginRegistry::new)
}

pub fn init_feature_registry() -> &'static FeaturePluginRegistry {
    FEATURE_REGISTRY.get_or_init(FeaturePluginRegistry::new)
}

/// Initialize feature plugin system
pub fn init_feature_plugin() {
    let reg = feature_registry();
    let _ = reg;
    log_info!("Feature plugin system initialized");
}

// Convenience functions

/// Enable feature
pub fn feature_enable(name: &[u8]) -> i32 {
    feature_registry().enable(name)
}

/// Disable feature
pub fn feature_disable(name: &[u8]) -> i32 {
    feature_registry().disable(name)
}

/// Call hooks
pub fn feature_call_hooks(point: HookPoint, data: *mut core::ffi::c_void) -> i32 {
    feature_registry().call_hooks(point, data)
}

/// Check if feature is enabled
pub fn feature_is_enabled(name: &[u8]) -> bool {
    if let Some(feature) = feature_registry().find(name) {
        // SAFETY: atomic memory operation on shared state
        unsafe { (*feature).enabled.load(Ordering::Acquire) }
    } else {
        false
    }
}

/// Define feature plugin
#[macro_export]
macro_rules! define_feature_plugin {
    ($name:ident, $category:expr, $enable:expr, $disable:expr) => {
        static mut $name: $crate::plugin::feature_plugin::FeaturePlugin = {
            let mut feature = $crate::plugin::feature_plugin::FeaturePlugin::new(
                stringify!($name).as_bytes(),
                $category,
            );
            feature.feature_ops.enable = Some($enable);
            feature.feature_ops.disable = Some($disable);
            feature
        };
    };
}

/// Add feature hook
#[macro_export]
macro_rules! feature_add_hook {
    ($feature:ident, $point:expr, $func:expr) => {
        {
            static mut HOOK: $crate::plugin::feature_plugin::FeatureHook = {
                $crate::plugin::feature_plugin::FeatureHook {
                    point: $point,
                    func: $func,
                    priority: 0,
                    enabled: core::sync::atomic::AtomicBool::new(true),
                    next: core::ptr::null_mut(),
                }
            };
            // SAFETY: unsafe block required for low-level memory or hardware access
            unsafe { $feature.add_hook(&mut HOOK); }
        }
    };
}
