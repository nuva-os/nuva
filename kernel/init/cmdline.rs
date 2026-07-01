/*
 * Nuva OS - Kernel - Init - Cmdline
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
use crate::pr_info;
/*
 * Nuva OS - Kernel - Command Line Parser
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Kernel command line parsing and parameter handling.
 */

use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

/// Maximum command line length
pub const CMDLINE_MAX: usize = 2048;

/// Maximum number of parameters
pub const PARAM_MAX: usize = 256;

/// Parameter Type
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamType {
    /// Boolean
    Bool = 0,
    /// Integer
    Int = 1,
    /// String
    String = 2,
    /// Callback
    Callback = 3,
}

/// Parameter Value
#[repr(C)]
pub union ParamValue {
    pub bool_val: bool,
    pub int_val: i64,
    pub str_val: [u8; 256],
}

/// Parameter Callback
pub type ParamCallback = unsafe extern "C" fn(*const u8, usize) -> i32;

/// Kernel Parameter
pub struct KernelParam {
    /// Parameter name
    pub name: [u8; 64],
    /// Parameter type
    pub param_type: ParamType,
    /// Value
    pub value: ParamValue,
    /// Callback
    pub callback: Option<ParamCallback>,
    /// Flags
    pub flags: AtomicU32,
    /// Set flag
    pub set: AtomicBool,
    /// Whether this slot is in use
    pub in_use: AtomicBool,
}

impl KernelParam {
    /// Create an empty unused parameter slot
    pub const fn new() -> Self {
        KernelParam {
            name: [0; 64],
            param_type: ParamType::Bool,
            value: ParamValue { bool_val: false },
            callback: None,
            flags: AtomicU32::new(0),
            set: AtomicBool::new(false),
            in_use: AtomicBool::new(false),
        }
    }

    pub fn new_bool(name: &[u8], default: bool) -> Self {
        let mut name_arr = [0u8; 64];
        let len = name.len().min(63);
        name_arr[..len].copy_from_slice(&name[..len]);

        KernelParam {
            name: name_arr,
            param_type: ParamType::Bool,
            value: ParamValue { bool_val: default },
            callback: None,
            flags: AtomicU32::new(0),
            set: AtomicBool::new(false),
            in_use: AtomicBool::new(true),
        }
    }

    pub fn new_int(name: &[u8], default: i64) -> Self {
        let mut name_arr = [0u8; 64];
        let len = name.len().min(63);
        name_arr[..len].copy_from_slice(&name[..len]);

        KernelParam {
            name: name_arr,
            param_type: ParamType::Int,
            value: ParamValue { int_val: default },
            callback: None,
            flags: AtomicU32::new(0),
            set: AtomicBool::new(false),
            in_use: AtomicBool::new(true),
        }
    }

    pub fn new_string(name: &[u8], default: &[u8]) -> Self {
        let mut name_arr = [0u8; 64];
        let mut str_arr = [0u8; 256];

        let name_len = name.len().min(63);
        let str_len = default.len().min(255);

        name_arr[..name_len].copy_from_slice(&name[..name_len]);
        str_arr[..str_len].copy_from_slice(&default[..str_len]);

        KernelParam {
            name: name_arr,
            param_type: ParamType::String,
            value: ParamValue { str_val: str_arr },
            callback: None,
            flags: AtomicU32::new(0),
            set: AtomicBool::new(false),
            in_use: AtomicBool::new(true),
        }
    }

    pub fn new_callback(name: &[u8], callback: ParamCallback) -> Self {
        let mut name_arr = [0u8; 64];
        let len = name.len().min(63);
        name_arr[..len].copy_from_slice(&name[..len]);

        KernelParam {
            name: name_arr,
            param_type: ParamType::Callback,
            value: ParamValue { int_val: 0 },
            callback: Some(callback),
            flags: AtomicU32::new(0),
            set: AtomicBool::new(false),
            in_use: AtomicBool::new(true),
        }
    }
}

/// Parameter Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct ParamFlags: u32 {
        /// Read only
        const READ_ONLY = 1 << 0;
        /// Write only
        const WRITE_ONLY = 1 << 1;
        /// Hidden
        const HIDDEN = 1 << 2;
        /// Experimental
        const EXPERIMENTAL = 1 << 3;
        /// Deprecated
        const DEPRECATED = 1 << 4;
    }
}

/// Command Line Manager
/// Manages kernel command line parameters using a fixed-size array.
pub struct CmdlineManager {
    /// Command line buffer
    pub cmdline: [u8; CMDLINE_MAX],
    /// Command line length
    pub cmdline_len: usize,
    /// Parameters storage (fixed-size array instead of linked list)
    params: [KernelParam; PARAM_MAX],
    /// Parameter count
    pub param_count: AtomicU32,
    /// Parsed flag
    pub parsed: AtomicBool,
}

impl CmdlineManager {
    pub const fn new() -> Self {
        CmdlineManager {
            cmdline: [0; CMDLINE_MAX],
            cmdline_len: 0,
            params: [const { KernelParam::new() }; PARAM_MAX],
            param_count: AtomicU32::new(0),
            parsed: AtomicBool::new(false),
        }
    }

    /// Initialize
    pub fn init(&self) {
        // Register built-in parameters
        self.register_builtin_params();

        log_info!("Cmdline manager initialized");
    }

    /// Register built-in parameters
    fn register_builtin_params(&mut self) {
        // Common kernel parameters
        self.add_param_bool(b"debug", false);
        self.add_param_bool(b"quiet", false);
        self.add_param_bool(b"loglevel", false);
        self.add_param_int(b"maxcpus", 0);
        self.add_param_int(b"mem", 0);
        self.add_param_string(b"root", b"/dev/sda1");
        self.add_param_string(b"rootfstype", b"ext4");
        self.add_param_bool(b"ro", false);
        self.add_param_bool(b"rw", false);
        self.add_param_string(b"init", b"/sbin/init");
        self.add_param_bool(b"nosmp", false);
        self.add_param_bool(b"nohz", false);
        self.add_param_bool(b"preempt", true);
        self.add_param_int(b"hz", 1000);
        self.add_param_string(b"console", b"ttyS0,115200");
        self.add_param_bool(b"earlycon", false);
        self.add_param_bool(b"nokaslr", false);
        self.add_param_bool(b"nospec", false);
        self.add_param_bool(b"mitigations", true);
    }

    /// Find a free slot in the params array
    fn find_free_slot(&self) -> Option<usize> {
        for i in 0..PARAM_MAX {
            if !self.params[i].in_use.load(Ordering::Acquire) {
                return Some(i);
            }
        }
        None
    }

    /// Add boolean parameter
    fn add_param_bool(&mut self, name: &[u8], default: bool) {
        if let Some(slot) = self.find_free_slot() {
            self.params[slot] = KernelParam::new_bool(name, default);
            self.param_count.fetch_add(1, Ordering::AcqRel);
        }
    }

    /// Add integer parameter
    fn add_param_int(&mut self, name: &[u8], default: i64) {
        if let Some(slot) = self.find_free_slot() {
            self.params[slot] = KernelParam::new_int(name, default);
            self.param_count.fetch_add(1, Ordering::AcqRel);
        }
    }

    /// Add string parameter
    fn add_param_string(&mut self, name: &[u8], default: &[u8]) {
        if let Some(slot) = self.find_free_slot() {
            self.params[slot] = KernelParam::new_string(name, default);
            self.param_count.fetch_add(1, Ordering::AcqRel);
        }
    }

    /// Set command line
    pub fn set_cmdline(&mut self, cmdline: &[u8]) {
        let len = cmdline.len().min(CMDLINE_MAX - 1);
        self.cmdline[..len].copy_from_slice(&cmdline[..len]);
        self.cmdline[len] = 0;
        self.cmdline_len = len;
    }

    /// Parse command line
    pub fn parse(&mut self) {
        if self.parsed.load(Ordering::Acquire) {
            return;
        }

        // Parse each token
        let mut start = 0;
        let mut i = 0;

        while i < self.cmdline_len {
            // Find end of token
            while i < self.cmdline_len && self.cmdline[i] != b' ' {
                i += 1;
            }

            if i > start {
                let token = self.cmdline[start..i].as_ptr();
                let token_len = i - start;
                // SAFETY: token points into self.cmdline which is a valid buffer,
                // and the slice [start..i] is within bounds.
                let token: &[u8] = unsafe { core::slice::from_raw_parts(token, token_len) };
                self.parse_token(token);
            }

            // Skip spaces
            while i < self.cmdline_len && self.cmdline[i] == b' ' {
                i += 1;
            }

            start = i;
        }

        self.parsed.store(true, Ordering::Release);
    }

    /// Parse single token
    fn parse_token(&mut self, token: &[u8]) {
        // Check for key=value format
        if let Some(eq_pos) = self.find_char(token, b'=') {
            let key = &token[..eq_pos];
            let value = &token[eq_pos + 1..];
            self.set_param(key, value);
        } else {
            // Boolean flag
            self.set_param_bool(token, true);
        }
    }

    /// Find character in slice
    fn find_char(&self, slice: &[u8], c: u8) -> Option<usize> {
        for (i, &ch) in slice.iter().enumerate() {
            if ch == c {
                return Some(i);
            }
        }
        None
    }

    /// Set parameter
    fn set_param(&mut self, key: &[u8], value: &[u8]) {
        // Find parameter
        if let Some(idx) = self.find_param_index(key) {
            match self.params[idx].param_type {
                ParamType::Bool => {
                    let bool_val = self.parse_bool(value);
                    self.params[idx].value = ParamValue { bool_val };
                    self.params[idx].set.store(true, Ordering::Release);
                }
                ParamType::Int => {
                    let int_val = self.parse_int(value);
                    self.params[idx].value = ParamValue { int_val };
                    self.params[idx].set.store(true, Ordering::Release);
                }
                ParamType::String => {
                    let mut str_arr = [0u8; 256];
                    let len = value.len().min(255);
                    str_arr[..len].copy_from_slice(&value[..len]);
                    self.params[idx].value = ParamValue { str_val: str_arr };
                    self.params[idx].set.store(true, Ordering::Release);
                }
                ParamType::Callback => {
                    if let Some(cb) = self.params[idx].callback {
                        // SAFETY: The callback function pointer was provided
                        // during parameter registration and is expected to be
                        // a valid function that handles the value pointer safely.
                        unsafe {
                            cb(value.as_ptr(), value.len());
                        }
                    }
                }
            }
        }
    }

    /// Set boolean parameter
    fn set_param_bool(&mut self, key: &[u8], value: bool) {
        if let Some(idx) = self.find_param_index(key) {
            if self.params[idx].param_type == ParamType::Bool {
                self.params[idx].value = ParamValue { bool_val: value };
                self.params[idx].set.store(true, Ordering::Release);
            }
        }
    }

    /// Find parameter index by name
    fn find_param_index(&self, name: &[u8]) -> Option<usize> {
        let name_len = name.len();
        for i in 0..PARAM_MAX {
            if !self.params[i].in_use.load(Ordering::Acquire) {
                continue;
            }
            let param_name = &self.params[i].name;
            if param_name[..name_len] == *name && param_name[name_len] == 0 {
                return Some(i);
            }
        }
        None
    }

    /// Find parameter by name (returns pointer for backward compatibility)
    fn find_param(&self, name: &[u8]) -> *mut KernelParam {
        if let Some(idx) = self.find_param_index(name) {
            &self.params[idx] as *const KernelParam as *mut KernelParam
        } else {
            core::ptr::null_mut()
        }
    }

    /// Parse boolean value
    fn parse_bool(&self, value: &[u8]) -> bool {
        match value {
            b"1" | b"true" | b"yes" | b"on" | b"y" => true,
            b"0" | b"false" | b"no" | b"off" | b"n" => false,
            _ => !value.is_empty(),
        }
    }

    /// Parse integer value
    fn parse_int(&self, value: &[u8]) -> i64 {
        let mut result: i64 = 0;
        let mut negative = false;
        let mut i = 0;

        if i < value.len() && value[i] == b'-' {
            negative = true;
            i += 1;
        }

        // Check for hex prefix
        if i + 1 < value.len() && value[i] == b'0' && (value[i + 1] == b'x' || value[i + 1] == b'X')
        {
            i += 2;
            while i < value.len() {
                let digit = if value[i] >= b'0' && value[i] <= b'9' {
                    value[i] - b'0'
                } else if value[i] >= b'a' && value[i] <= b'f' {
                    value[i] - b'a' + 10
                } else if value[i] >= b'A' && value[i] <= b'F' {
                    value[i] - b'A' + 10
                } else {
                    break;
                };
                result = result.wrapping_mul(16).wrapping_add(digit as i64);
                i += 1;
            }
        } else {
            while i < value.len() && value[i] >= b'0' && value[i] <= b'9' {
                result = result
                    .wrapping_mul(10)
                    .wrapping_add((value[i] - b'0') as i64);
                i += 1;
            }
        }

        if negative {
            result = -result;
        }

        result
    }

    /// Get boolean parameter
    pub fn get_bool(&self, name: &[u8]) -> bool {
        if let Some(idx) = self.find_param_index(name) {
            if self.params[idx].param_type == ParamType::Bool {
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    return self.params[idx].value.bool_val;
                }
            }
        }
        false
    }

    /// Get integer parameter
    pub fn get_int(&self, name: &[u8]) -> i64 {
        if let Some(idx) = self.find_param_index(name) {
            if self.params[idx].param_type == ParamType::Int {
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    return self.params[idx].value.int_val;
                }
            }
        }
        0
    }

    /// Get string parameter
    pub fn get_string(&self, name: &[u8]) -> &[u8] {
        if let Some(idx) = self.find_param_index(name) {
            if self.params[idx].param_type == ParamType::String {
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    let str_val = &self.params[idx].value.str_val;
                    let len = str_val.iter().position(|&c| c == 0).unwrap_or(255);
                    return &str_val[..len];
                }
            }
        }
        b""
    }

    /// Check if parameter was set
    pub fn is_set(&self, name: &[u8]) -> bool {
        if let Some(idx) = self.find_param_index(name) {
            return self.params[idx].set.load(Ordering::Acquire);
        }
        false
    }

    /// Dump command line
    pub fn dump(&self) {
        log_info!("Kernel command line:");
        // SAFETY: cmdline contains only valid UTF-8 from the bootloader
        unsafe {
            log_info!(
                "  {}",
                core::str::from_utf8_unchecked(&self.cmdline[..self.cmdline_len])
            );
        }
    }

    /// Dump all parameters
    pub fn dump_params(&self) {
        log_info!("Kernel parameters:");
        for i in 0..PARAM_MAX {
            if !self.params[i].in_use.load(Ordering::Acquire) {
                continue;
            }
            let param = &self.params[i];
            let name_len = param.name.iter().position(|&c| c == 0).unwrap_or(63);
            // SAFETY: param name contains only valid UTF-8
            let name_str = unsafe { core::str::from_utf8_unchecked(&param.name[..name_len]) };

            match param.param_type {
                ParamType::Bool => {
                    // SAFETY: atomic memory operation on shared state
                    unsafe {
                        log_info!(
                            "  {} = {} (set={})",
                            name_str,
                            param.value.bool_val,
                            param.set.load(Ordering::Acquire)
                        );
                    }
                }
                ParamType::Int => {
                    // SAFETY: atomic memory operation on shared state
                    unsafe {
                        log_info!(
                            "  {} = {} (set={})",
                            name_str,
                            param.value.int_val,
                            param.set.load(Ordering::Acquire)
                        );
                    }
                }
                ParamType::String => {
                    let val = self.get_string(&param.name[..name_len]);
                    // SAFETY: string parameter value contains only valid UTF-8
                    unsafe {
                        log_info!(
                            "  {} = \"{}\" (set={})",
                            name_str,
                            core::str::from_utf8_unchecked(val),
                            param.set.load(Ordering::Acquire)
                        );
                    }
                }
                ParamType::Callback => {
                    log_info!("  {} = <callback>", name_str);
                }
            }
        }
    }
}

/// Global cmdline manager
static CMDLINE_MANAGER: crate::sync_oncelock::OnceLock<CmdlineManager> = crate::sync_oncelock::OnceLock::new();

/// Get cmdline manager
pub fn cmdline_manager() -> &'static CmdlineManager {
    CMDLINE_MANAGER.get_or_init(CmdlineManager::new)
}

pub fn init_cmdline_manager() -> &'static CmdlineManager {
    CMDLINE_MANAGER.get_or_init(CmdlineManager::new)
}

/// Initialize cmdline
pub fn init_cmdline() {
    let mgr = cmdline_manager();
    mgr.init();
}

/// Parse command line
pub fn parse_cmdline(cmdline: &[u8]) {
    let mgr = cmdline_manager();
    mgr.set_cmdline(cmdline);
    mgr.parse();
}

// Convenience functions

/// Get debug flag
pub fn cmdline_debug() -> bool {
    cmdline_manager().get_bool(b"debug")
}

/// Get quiet flag
pub fn cmdline_quiet() -> bool {
    cmdline_manager().get_bool(b"quiet")
}

/// Get max CPUs
pub fn cmdline_maxcpus() -> i64 {
    cmdline_manager().get_int(b"maxcpus")
}

/// Get root device
pub fn cmdline_root() -> &'static [u8] {
    cmdline_manager().get_string(b"root")
}

/// Get root filesystem type
pub fn cmdline_rootfstype() -> &'static [u8] {
    cmdline_manager().get_string(b"rootfstype")
}

/// Get init path
pub fn cmdline_init() -> &'static [u8] {
    cmdline_manager().get_string(b"init")
}

/// Get console device
pub fn cmdline_console() -> &'static [u8] {
    cmdline_manager().get_string(b"console")
}

/// Check if read-only root
pub fn cmdline_ro() -> bool {
    cmdline_manager().get_bool(b"ro")
}

/// Check if SMP disabled
pub fn cmdline_nosmp() -> bool {
    cmdline_manager().get_bool(b"nosmp")
}

/// Check if preemption enabled
pub fn cmdline_preempt() -> bool {
    cmdline_manager().get_bool(b"preempt")
}

/// Get timer frequency
pub fn cmdline_hz() -> i64 {
    cmdline_manager().get_int(b"hz")
}

/// Module parameter macro helper
#[macro_export]
macro_rules! module_param {
    ($name:ident, $type:ty, $default:expr) => {
        static $name: $crate::cmdline::KernelParam = {
            let mut param =
                $crate::cmdline::KernelParam::new(stringify!($name).as_bytes(), $default);
            param
        };
    };
}

/// Module parameter description
#[macro_export]
macro_rules! module_param_desc {
    ($name:ident, $desc:expr) => {
        #[used]
        #[link_section = ".module_params"]
        static __PARAM_DESC_ $name: (&'static [u8], &'static [u8]) = (
            stringify!($name).as_bytes(),
            $desc.as_bytes(),
        );
    };
}
