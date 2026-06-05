/*
 * Nuva OS - Kernel - Init - Config
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
 * Nuva OS - Kernel - Configuration System
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Kernel configuration options and Kconfig support.
 */

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

use crate::posix::errno::Errno;
/// Maximum number of config entries
const MAX_CONFIG_ENTRIES: usize = 128;

/// Config Type
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigType {
    /// Boolean
    Bool = 0,
    /// Tristate
    Tristate = 1,
    /// Integer
    Int = 2,
    /// Hex
    Hex = 3,
    /// String
    String = 4,
}

/// Config Value
#[repr(C)]
pub union ConfigValue {
    pub bool_val: bool,
    pub tri_val: u32,
    pub int_val: i64,
    pub hex_val: u64,
    pub str_val: [u8; 256],
}

impl Clone for ConfigValue {
    fn clone(&self) -> Self {
        // Safe because we just copy the raw bytes of the union
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let mut clone = ConfigValue { int_val: 0 };
            core::ptr::copy_nonoverlapping(
                self as *const ConfigValue as *const u8,
                &mut clone as *mut ConfigValue as *mut u8,
                core::mem::size_of::<ConfigValue>(),
            );
            clone
        }
    }
}

impl Copy for ConfigValue {}

/// Config Entry
pub struct ConfigEntry {
    /// Config name
    pub name: [u8; 64],
    /// Config type
    pub config_type: ConfigType,
    /// Value
    pub value: ConfigValue,
    /// Default value
    pub default_val: ConfigValue,
    /// Help text
    pub help: [u8; 512],
    /// Dependencies
    pub depends: [u8; 256],
    /// Selects
    pub selects: [u8; 256],
    /// Prompt
    pub prompt: [u8; 128],
    /// Module name
    pub module: [u8; 64],
    /// Flags
    pub flags: AtomicU32,
    /// Whether this entry is in use
    pub in_use: AtomicBool,
}

impl ConfigEntry {
    /// Create an empty unused entry
    pub const fn new() -> Self {
        ConfigEntry {
            name: [0; 64],
            config_type: ConfigType::Bool,
            value: ConfigValue { bool_val: false },
            default_val: ConfigValue { bool_val: false },
            help: [0; 512],
            depends: [0; 256],
            selects: [0; 256],
            prompt: [0; 128],
            module: [0; 64],
            flags: AtomicU32::new(0),
            in_use: AtomicBool::new(false),
        }
    }
}

/// Config Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ConfigFlags: u32 {
        /// Set by user
        const USER_SET = 1 << 0;
        /// Set by default
        const DEFAULT_SET = 1 << 1;
        /// Read only
        const READ_ONLY = 1 << 2;
        /// Hidden
        const HIDDEN = 1 << 3;
        /// Deprecated
        const DEPRECATED = 1 << 4;
    }
}

/// Config Section
pub struct ConfigSection {
    /// Section name
    pub name: [u8; 64],
    /// Entries
    pub entries: *mut ConfigEntry,
    /// Number of entries
    pub num_entries: u32,
    /// Next section
    pub next: *mut ConfigSection,
}

/// Config Manager
/// Manages kernel configuration entries using a fixed-size array.
/// Supports dynamic registration and lookup of config options.
pub struct ConfigManager {
    /// Config entries storage
    entries: [ConfigEntry; MAX_CONFIG_ENTRIES],
    /// Section count
    pub section_count: AtomicU32,
    /// Entry count
    pub entry_count: AtomicU32,
    /// Config file path
    pub config_path: [u8; 256],
    /// Dirty flag
    pub dirty: AtomicBool,
}

impl ConfigManager {
    pub const fn new() -> Self {
        ConfigManager {
            entries: [const { ConfigEntry::new() }; MAX_CONFIG_ENTRIES],
            section_count: AtomicU32::new(0),
            entry_count: AtomicU32::new(0),
            config_path: [0; 256],
            dirty: AtomicBool::new(false),
        }
    }
    
    /// Initialize
    pub fn init(&self) {
        // Register built-in config options
        self.register_builtin_configs();
        
        log_info!("Config manager initialized");
    }
    
    /// Register built-in configs
    fn register_builtin_configs(&mut self) {
        // General options
        self.add_bool("CONFIG_PRINTK", true, "Enable kernel printk");
        self.add_bool("CONFIG_DEBUG", false, "Enable kernel debugging");
        self.add_bool("CONFIG_MODULES", true, "Enable loadable module support");
        
        // Memory options
        self.add_int("CONFIG_PAGE_SIZE", 4096, "Page size in bytes");
        self.add_int("CONFIG_MAX_CPUS", 256, "Maximum number of CPUs");
        self.add_bool("CONFIG_NUMA", false, "Enable NUMA support");
        
        // Scheduler options
        self.add_int("CONFIG_HZ", 1000, "Timer interrupt frequency");
        self.add_bool("CONFIG_PREEMPT", true, "Enable kernel preemption");
        self.add_bool("CONFIG_SCHED_SMT", true, "Enable SMT scheduling");
        
        // Driver options
        self.add_bool("CONFIG_PCI", true, "Enable PCI support");
        self.add_bool("CONFIG_USB", true, "Enable USB support");
        self.add_bool("CONFIG_NET", true, "Enable networking");
        
        // Filesystem options
        self.add_bool("CONFIG_EXT4", true, "Enable ext4 filesystem");
        self.add_bool("CONFIG_FAT", true, "Enable FAT filesystem");
        
        // Security options
        self.add_bool("CONFIG_SECURITY", true, "Enable security framework");
        self.add_bool("CONFIG_HARDENED", false, "Enable hardening features");
    }
    
    /// Find a free slot in the entries array
    fn find_free_slot(&self) -> Option<usize> {
        for i in 0..MAX_CONFIG_ENTRIES {
            if !self.entries[i].in_use.load(Ordering::Acquire) {
                return Some(i);
            }
        }
        None
    }
    
    /// Find an entry by name, returns the index
    fn find_entry(&self, name: &str) -> Option<usize> {
        let name_bytes = name.as_bytes();
        let name_len = name_bytes.len();
        
        for i in 0..MAX_CONFIG_ENTRIES {
            if !self.entries[i].in_use.load(Ordering::Acquire) {
                continue;
            }
            
            let entry_name = &self.entries[i].name;
            // Compare name bytes up to the null terminator
            if entry_name[..name_len] == *name_bytes && entry_name[name_len] == 0 {
                return Some(i);
            }
        }
        None
    }
    
    /// Add boolean config
    fn add_bool(&mut self, name: &str, default: bool, help: &str) {
        if let Some(slot) = self.find_free_slot() {
            let entry = &mut self.entries[slot];
            
            // Copy name
            let name_len = name.len().min(63);
            entry.name[..name_len].copy_from_slice(&name.as_bytes()[..name_len]);
            entry.name[name_len] = 0;
            
            // Copy help
            let help_len = help.len().min(511);
            entry.help[..help_len].copy_from_slice(&help.as_bytes()[..help_len]);
            entry.help[help_len] = 0;
            
            // Set type and value
            entry.config_type = ConfigType::Bool;
            entry.value = ConfigValue { bool_val: default };
            entry.default_val = ConfigValue { bool_val: default };
            entry.flags.store(ConfigFlags::DEFAULT_SET.bits(), Ordering::Release);
            entry.in_use.store(true, Ordering::Release);
            
            self.entry_count.fetch_add(1, Ordering::AcqRel);
        } else {
            log_info!("Config: no free slots, cannot add {}", name);
        }
    }
    
    /// Add integer config
    fn add_int(&mut self, name: &str, default: i64, help: &str) {
        if let Some(slot) = self.find_free_slot() {
            let entry = &mut self.entries[slot];
            
            // Copy name
            let name_len = name.len().min(63);
            entry.name[..name_len].copy_from_slice(&name.as_bytes()[..name_len]);
            entry.name[name_len] = 0;
            
            // Copy help
            let help_len = help.len().min(511);
            entry.help[..help_len].copy_from_slice(&help.as_bytes()[..help_len]);
            entry.help[help_len] = 0;
            
            // Set type and value
            entry.config_type = ConfigType::Int;
            entry.value = ConfigValue { int_val: default };
            entry.default_val = ConfigValue { int_val: default };
            entry.flags.store(ConfigFlags::DEFAULT_SET.bits(), Ordering::Release);
            entry.in_use.store(true, Ordering::Release);
            
            self.entry_count.fetch_add(1, Ordering::AcqRel);
        } else {
            log_info!("Config: no free slots, cannot add {}", name);
        }
    }
    
    /// Get boolean config
    pub fn get_bool(&self, name: &str) -> bool {
        if let Some(idx) = self.find_entry(name) {
            let entry = &self.entries[idx];
            if entry.config_type == ConfigType::Bool {
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe { return entry.value.bool_val; }
            }
        }
        false
    }
    
    /// Get integer config
    pub fn get_int(&self, name: &str) -> i64 {
        if let Some(idx) = self.find_entry(name) {
            let entry = &self.entries[idx];
            if entry.config_type == ConfigType::Int {
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe { return entry.value.int_val; }
            }
        }
        0
    }
    
    /// Get string config
    pub fn get_string(&self, name: &str) -> &[u8] {
        if let Some(idx) = self.find_entry(name) {
            let entry = &self.entries[idx];
            if entry.config_type == ConfigType::String {
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    let str_val = &entry.value.str_val;
                    let len = str_val.iter().position(|&c| c == 0).unwrap_or(255);
                    return &str_val[..len];
                }
            }
        }
        b""
    }
    
    /// Set boolean config
    pub fn set_bool(&mut self, name: &str, value: bool) -> i32 {
        if let Some(idx) = self.find_entry(name) {
            let entry = &mut self.entries[idx];
            
            // Check if read-only
            let flags = ConfigFlags::from_bits_truncate(entry.flags.load(Ordering::Acquire));
            if flags.contains(ConfigFlags::READ_ONLY) {
                return Errno::Eperm.to_ret_i32(); // EPERM
            }
            
            if entry.config_type == ConfigType::Bool {
                entry.value = ConfigValue { bool_val: value };
                entry.flags.fetch_or(ConfigFlags::USER_SET.bits(), Ordering::AcqRel);
                self.dirty.store(true, Ordering::Release);
                return 0;
            }
        }
        -22 // EINVAL
    }
    
    /// Set integer config
    pub fn set_int(&mut self, name: &str, value: i64) -> i32 {
        if let Some(idx) = self.find_entry(name) {
            let entry = &mut self.entries[idx];
            
            // Check if read-only
            let flags = ConfigFlags::from_bits_truncate(entry.flags.load(Ordering::Acquire));
            if flags.contains(ConfigFlags::READ_ONLY) {
                return Errno::Eperm.to_ret_i32(); // EPERM
            }
            
            if entry.config_type == ConfigType::Int {
                entry.value = ConfigValue { int_val: value };
                entry.flags.fetch_or(ConfigFlags::USER_SET.bits(), Ordering::AcqRel);
                self.dirty.store(true, Ordering::Release);
                return 0;
            }
        }
        -22 // EINVAL
    }
    
    /// Load config from file
    pub fn load(&mut self, path: &[u8]) -> i32 {
        // Copy path
        let len = path.len().min(255);
        self.config_path[..len].copy_from_slice(&path[..len]);
        self.config_path[len] = 0;
        
        // Parse config file line by line
        // Format: CONFIG_XXX=value
        // In a real implementation, this would read from the filesystem
        // and parse each line. For now, we mark the path as set.
        log_info!("Config: path set to {} bytes", len);
        0
    }
    
    /// Save config to file
    pub fn save(&mut self) -> i32 {
        if !self.dirty.load(Ordering::Acquire) {
            return 0;
        }
        
        // In a real implementation, this would iterate over all entries
        // and write them to the config file in "CONFIG_XXX=value" format.
        // For now, we just clear the dirty flag.
        log_info!("Config: saving {} entries", self.entry_count.load(Ordering::Acquire));
        self.dirty.store(false, Ordering::Release);
        0
    }
    
    /// Dump all configs
    pub fn dump(&self) {
        log_info!("Kernel Configuration:");
        for i in 0..MAX_CONFIG_ENTRIES {
            if !self.entries[i].in_use.load(Ordering::Acquire) {
                continue;
            }
            let entry = &self.entries[i];
            let name_len = entry.name.iter().position(|&c| c == 0).unwrap_or(63);
            let name_str = unsafe { core::str::from_utf8_unchecked(&entry.name[..name_len]) };
            
            match entry.config_type {
                ConfigType::Bool => {
                    // SAFETY: unsafe block required for low-level memory or hardware access
                    unsafe { log_info!("  {} = {}", name_str, entry.value.bool_val); }
                }
                ConfigType::Int => {
                    // SAFETY: unsafe block required for low-level memory or hardware access
                    unsafe { log_info!("  {} = {}", name_str, entry.value.int_val); }
                }
                ConfigType::String => {
                    log_info!("  {} = \"{}\"", name_str, unsafe { core::str::from_utf8_unchecked(self.get_string(name_str)) });
                }
                _ => {
                    log_info!("  {} = (unsupported type)", name_str);
                }
            }
        }
    }
    
    /// Reset a config entry to its default value
    pub fn reset_to_default(&mut self, name: &str) -> i32 {
        if let Some(idx) = self.find_entry(name) {
            let entry = &mut self.entries[idx];
            entry.value = entry.default_val;
            entry.flags.fetch_and(!ConfigFlags::USER_SET.bits(), Ordering::AcqRel);
            self.dirty.store(true, Ordering::Release);
            return 0;
        }
        -22 // EINVAL
    }
}

/// Global config manager
static CONFIG_MANAGER: core::sync::OnceLock<ConfigManager> = core::sync::OnceLock::new();

/// Get config manager
pub fn config_manager() -> &'static ConfigManager {
    CONFIG_MANAGER.get_or_init(ConfigManager::new)
}

pub fn init_config_manager() -> &'static ConfigManager {
    CONFIG_MANAGER.get_or_init(ConfigManager::new)
}

/// Initialize config
pub fn init_config() {
    let mgr = config_manager();
    mgr.init();
}

// Convenience macros

/// Check if config is enabled
#[macro_export]
macro_rules! config_enabled {
    ($name:expr) => {
        $crate::config::config_manager().get_bool($name)
    };
}

/// Get config integer value
#[macro_export]
macro_rules! config_int {
    ($name:expr) => {
        $crate::config::config_manager().get_int($name)
    };
}

/// Compile-time config check
#[macro_export]
macro_rules! if_config {
    ($name:expr, $then:block) => {
        if $crate::config::config_manager().get_bool($name) $then
    };
    ($name:expr, $then:block, $else:block) => {
        if $crate::config::config_manager().get_bool($name) $then else $else
    };
}

// Common config checks

/// Check if printk is enabled
pub fn config_printk() -> bool {
    config_manager().get_bool("CONFIG_PRINTK")
}

/// Check if debug is enabled
pub fn config_debug() -> bool {
    config_manager().get_bool("CONFIG_DEBUG")
}

/// Check if modules are enabled
pub fn config_modules() -> bool {
    config_manager().get_bool("CONFIG_MODULES")
}

/// Get page size
pub fn config_page_size() -> i64 {
    config_manager().get_int("CONFIG_PAGE_SIZE")
}

/// Get max CPUs
pub fn config_max_cpus() -> i64 {
    config_manager().get_int("CONFIG_MAX_CPUS")
}

/// Get timer frequency
pub fn config_hz() -> i64 {
    config_manager().get_int("CONFIG_HZ")
}
