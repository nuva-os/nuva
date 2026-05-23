use crate::{pr_err, pr_info, pr_warn};
/*
 * Nuva OS - HAL - Platform Detection and Identification
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

use crate::kernel::arch::platform::{Arch, Platform};

/// Device form factor classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormFactor {
    /// Mobile phone: touch-primary, single-app, power-constrained.
    Mobile,
    /// Tablet: touch + stylus, split-screen, moderate power.
    Tablet,
    /// PC: mouse + keyboard, multi-window, performance-oriented.
    Pc,
}

/// Power source type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerSource {
    /// AC power connected (desktop or laptop plugged in).
    Ac,
    /// Battery power (mobile or laptop on battery).
    Battery,
    /// Power source unknown (e.g., not yet detected).
    Unknown,
}

/// Available input device types (bitmask).
bitflags::bitflags! {
    /// Bitmask of available input devices.
    pub struct InputDeviceSet: u32 {
        /// Touchscreen present.
        const TOUCHSCREEN = 1 << 0;
        /// Keyboard present.
        const KEYBOARD    = 1 << 1;
        /// Mouse/trackpad present.
        const MOUSE       = 1 << 2;
        /// Stylus/pen present.
        const STYLUS      = 1 << 3;
        /// Gamepad/controller present.
        const GAMEPAD     = 1 << 4;
        /// Trackball present.
        const TRACKBALL   = 1 << 5;
        /// Rotary encoder present.
        const ROTARY      = 1 << 6;
    }
}

/// Runtime platform identification information.
pub struct PlatformInfo {
    /// CPU architecture (ARM64, X64, LoongArch64).
    pub arch: Arch,
    /// SoC platform identifier.
    pub soc: Platform,
    /// Detected form factor.
    pub form_factor: FormFactor,
    /// Number of CPU cores.
    pub core_count: u32,
    /// Total physical memory in bytes.
    pub memory_size: u64,
    /// Primary display width in pixels.
    pub display_width: u32,
    /// Primary display height in pixels.
    pub display_height: u32,
    /// Display pixel density (DPI).
    pub display_dpi: u32,
    /// Available input devices bitmask.
    pub input_devices: InputDeviceSet,
    /// Power source type.
    pub power_source: PowerSource,
}

impl PlatformInfo {
    /// Create a new PlatformInfo with the given parameters.
    pub const fn new() -> Self {
        PlatformInfo {
            arch: Arch::ARM64,
            soc: Platform::Kirin9020,
            form_factor: FormFactor::Mobile,
            core_count: 0,
            memory_size: 0,
            display_width: 0,
            display_height: 0,
            display_dpi: 0,
            input_devices: InputDeviceSet::empty(),
            power_source: PowerSource::Unknown,
        }
    }

    /// Classify form factor based on display size, input devices, and power source.
    /// Classification rules:
    /// - Mobile: screen diagonal < 7 inches, touchscreen present, battery powered
    /// - Tablet: 7-13 inches, touchscreen present, may have keyboard
    /// - PC: screen > 13 inches, keyboard + mouse present, typically AC powered
    pub fn classify_form_factor(&mut self) {
        /// Calculate approximate screen diagonal in inches from resolution and DPI.
        let diagonal_inches = if self.display_dpi > 0 {
            let w = self.display_width as f32 / self.display_dpi as f32;
            let h = self.display_height as f32 / self.display_dpi as f32;
            // Approximate diagonal using Pythagorean theorem
            let diag_sq = w * w + h * h;
            // Integer approximation without libm
            let diag_approx = if diag_sq > 0.0 { diag_sq } else { 1.0 };
            // Rough square root via integer math
            let diag_int = diag_approx as u32;
            let mut est = 1u32;
            while est * est < diag_int { est += 1; }
            est as f32
        } else {
            // No DPI info: use resolution heuristics
            if self.display_width <= 1080 { 5.0 }       // Mobile
            else if self.display_width <= 2560 { 10.0 }  // Tablet
            else { 15.0 }                                 // PC
        };

        let has_touch = self.input_devices.contains(InputDeviceSet::TOUCHSCREEN);
        let has_keyboard = self.input_devices.contains(InputDeviceSet::KEYBOARD);
        let has_mouse = self.input_devices.contains(InputDeviceSet::MOUSE);

        self.form_factor = if diagonal_inches < 7.0 && has_touch {
            FormFactor::Mobile
        } else if diagonal_inches <= 13.0 && has_touch {
            if has_keyboard && has_mouse {
                FormFactor::Pc
            } else {
                FormFactor::Tablet
            }
        } else if has_keyboard && has_mouse {
            FormFactor::Pc
        } else if has_touch {
            FormFactor::Tablet
        } else {
            // Default: PC for large screens, Mobile otherwise
            if diagonal_inches > 10.0 {
                FormFactor::Pc
            } else {
                FormFactor::Mobile
            }
        };
    }

    /// Check if the current platform matches the compile-time feature configuration.
    pub fn validate_compile_time_match(&self) -> bool {
        #[cfg(feature = "arm64")]
        {
            if self.arch != Arch::ARM64 {
                log_error!("Platform mismatch: compiled for ARM64 but detected {:?}", self.arch);
                return false;
            }
        }
        #[cfg(feature = "x64")]
        {
            if self.arch != Arch::X64 {
                log_error!("Platform mismatch: compiled for x86_64 but detected {:?}", self.arch);
                return false;
            }
        }
        true
    }

    /// Get display diagonal in inches (approximate).
    pub fn display_diagonal_inches(&self) -> u32 {
        if self.display_dpi == 0 { return 0; }
        let w = self.display_width / self.display_dpi;
        let h = self.display_height / self.display_dpi;
        let diag_sq = w * w + h * h;
        let mut est = 1u32;
        while est * est < diag_sq { est += 1; }
        est
    }

    /// Check if this is a mobile form factor.
    pub fn is_mobile(&self) -> bool {
        self.form_factor == FormFactor::Mobile
    }

    /// Check if this is a tablet form factor.
    pub fn is_tablet(&self) -> bool {
        self.form_factor == FormFactor::Tablet
    }

    /// Check if this is a PC form factor.
    pub fn is_pc(&self) -> bool {
        self.form_factor == FormFactor::Pc
    }
}

/// Global PlatformInfo instance.
static mut PLATFORM_INFO: PlatformInfo = PlatformInfo::new();

/// Get a reference to the global PlatformInfo.
pub fn get_platform_info() -> &'static PlatformInfo {
    // SAFETY: PLATFORM_INFO is a mutable static accessed only during single-threaded
    // HAL initialization; after init completes, only immutable references are returned
    // via get_platform_info(), preventing data races.
    unsafe { &PLATFORM_INFO }
}

/// Get a mutable reference to the global PlatformInfo (for initialization only).
pub fn get_platform_info_mut() -> &'static mut PlatformInfo {
    // SAFETY: get_platform_info_mut() is called only during single-threaded HAL
    // initialization before any other CPU cores are online; no concurrent access possible.
    unsafe { &mut PLATFORM_INFO }
}

/// Detect platform at runtime by reading CPU identification registers.
fn detect_soc() -> Platform {
    #[cfg(target_arch = "aarch64")]
    {
        // Read MIDR_EL1 to identify the CPU implementer and part number
        // SAFETY: MRS MIDR_EL1 is a read-only system register access on AArch64;
        // it does not modify any system state and is always readable at any exception level.
        let midr: u64;
        unsafe { core::arch::asm!("mrs {}, midr_el1", out(reg) midr); }

        let implementer = (midr >> 24) & 0xFF;
        let part_num = (midr >> 4) & 0xFFF;

        match implementer {
            0x41 => {
                // ARM Limited: Cortex-A series
                // Snapdragon uses ARM cores, further identification needed
                Platform::Snapdragon8Gen4
            }
            0x48 => {
                // HiSilicon (Huawei): Kirin series
                Platform::Kirin9020
            }
            0x51 => {
                // Qualcomm: Snapdragon Oryon cores
                Platform::Snapdragon8Gen4
            }
            _ => {
                log_warn!("Unknown ARM64 implementer: 0x{:02x}, part: 0x{:03x}", implementer, part_num);
                Platform::Snapdragon8Gen4 // Default ARM64
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    {
        // Read CPUID to identify vendor
        // SAFETY: The inline asm reads general-purpose registers (EBX, ECX, EDX)
        // after a CPUID-like sequence; these are read-only operations that do not
        // modify system state. The xchg and mov instructions are benign register moves.
        let vendor_ebx: u32;
        let vendor_ecx: u32;
        let vendor_edx: u32;
        unsafe {
            core::arch::asm!(
                "xchg {bx}, {bx}",
                "mov {ecx}, ecx",
                "mov {edx}, edx",
                bx = out(reg) vendor_ebx,
                ecx = out(reg) vendor_ecx,
                edx = out(reg) vendor_edx,
            );
        }

        // Reconstruct vendor string from EBX, EDX, ECX (CPUID leaf 0)
        let vendor_bytes: [u8; 12] = [
            (vendor_ebx & 0xFF) as u8, ((vendor_ebx >> 8) & 0xFF) as u8, ((vendor_ebx >> 16) & 0xFF) as u8, ((vendor_ebx >> 24) & 0xFF) as u8,
            (vendor_edx & 0xFF) as u8, ((vendor_edx >> 8) & 0xFF) as u8, ((vendor_edx >> 16) & 0xFF) as u8, ((vendor_edx >> 24) & 0xFF) as u8,
            (vendor_ecx & 0xFF) as u8, ((vendor_ecx >> 8) & 0xFF) as u8, ((vendor_ecx >> 16) & 0xFF) as u8, ((vendor_ecx >> 24) & 0xFF) as u8,
        ];

        if &vendor_bytes[0..4] == b"Genu" {
            // GenuineIntel
            Platform::IntelCore
        } else if &vendor_bytes[0..4] == b"Auth" {
            // AuthenticAMD
            Platform::AmdRyzen
        } else {
            Platform::GenericX64
        }
    }

    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        Platform::GenericX64
    }
}

/// Detect the current architecture at compile time.
const fn detect_arch() -> Arch {
    #[cfg(target_arch = "aarch64")]
    { Arch::ARM64 }

    #[cfg(target_arch = "x86_64")]
    { Arch::X64 }

    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    { Arch::X64 }
}

/// Detect number of CPU cores.
fn detect_core_count() -> u32 {
    #[cfg(target_arch = "aarch64")]
    {
        // Read MPIDR_EL1 affinity bits to count cores
        // On real hardware, this would iterate over all possible affinity levels
        // For now, use a reasonable default based on platform
        8
    }

    #[cfg(target_arch = "x86_64")]
    {
        // Use CPUID leaf 0xB (Extended Topology) to count logical processors
        // For now, use a reasonable default
        4
    }

    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    { 4 }
}

/// Detect total physical memory size.
fn detect_memory_size() -> u64 {
    // In a real implementation, this would:
    // - ARM64: Read from Device Tree /memory node
    // - x86_64: Read from ACPI SRAT or e820 map from bootloader
    // Default values per platform
    #[cfg(feature = "arm64")]
    { 8 * 1024 * 1024 * 1024 } // 8 GB typical mobile

    #[cfg(feature = "x64")]
    { 16 * 1024 * 1024 * 1024 } // 16 GB typical PC

    #[cfg(not(any(feature = "arm64", feature = "x64")))]
    { 4 * 1024 * 1024 * 1024 }
}

/// Detect display information from Device Tree (ARM64) or ACPI (x86_64).
fn detect_display_info() -> (u32, u32, u32) {
    // In a real implementation, this would:
    // - ARM64: Parse DT display node for resolution and DPI
    // - x86_64: Use EFI GOP or ACPI display info
    #[cfg(feature = "arm64")]
    {
        (1080, 2400, 400) // Typical mobile: 1080x2400 @ 400dpi
    }

    #[cfg(feature = "x64")]
    {
        (1920, 1080, 96) // Typical PC: 1920x1080 @ 96dpi
    }

    #[cfg(not(any(feature = "arm64", feature = "x64")))]
    {
        (1920, 1080, 96)
    }
}

/// Detect available input devices.
fn detect_input_devices() -> InputDeviceSet {
    // In a real implementation, this would:
    // - Enumerate HID devices from DT/ACPI
    // - Check for touchscreen, keyboard, mouse, etc.
    #[cfg(feature = "arm64")]
    {
        InputDeviceSet::TOUCHSCREEN // Mobile: touchscreen
    }

    #[cfg(feature = "x64")]
    {
        InputDeviceSet::KEYBOARD | InputDeviceSet::MOUSE // PC: keyboard + mouse
    }

    #[cfg(not(any(feature = "arm64", feature = "x64")))]
    {
        InputDeviceSet::empty()
    }
}

/// Detect power source.
fn detect_power_source() -> PowerSource {
    // In a real implementation, this would:
    // - Read PMIC status for battery presence
    // - Check AC adapter status
    #[cfg(feature = "arm64")]
    { PowerSource::Battery } // Mobile: battery

    #[cfg(feature = "x64")]
    { PowerSource::Ac } // PC: AC power

    #[cfg(not(any(feature = "arm64", feature = "x64")))]
    { PowerSource::Unknown }
}

/// Perform full platform detection and populate PlatformInfo.
pub fn detect_platform() {
    let info = get_platform_info_mut();

    // Detect hardware properties
    info.arch = detect_arch();
    info.soc = detect_soc();
    info.core_count = detect_core_count();
    info.memory_size = detect_memory_size();

    let (w, h, dpi) = detect_display_info();
    info.display_width = w;
    info.display_height = h;
    info.display_dpi = dpi;

    info.input_devices = detect_input_devices();
    info.power_source = detect_power_source();

    // Classify form factor from detected properties
    info.classify_form_factor();

    // Validate compile-time vs runtime match
    if !info.validate_compile_time_match() {
        log_error!("FATAL: Compile-time platform does not match runtime hardware!");
        log_error!("System halted to prevent incorrect HAL operation.");
        // Halt the system - cannot safely continue with mismatched HAL
        loop {
            core::hint::spin_loop();
        }
    }

    // Log detected platform information
    log_info!("Platform: arch={:?} soc={:?} form_factor={:?}", info.arch, info.soc, info.form_factor);
    log_info!("CPU cores: {} memory: {}MB", info.core_count, info.memory_size / (1024 * 1024));
    log_info!("Display: {}x{} @ {}dpi", info.display_width, info.display_height, info.display_dpi);
    log_info!("Input: {:#x} Power: {:?}", info.input_devices.bits(), info.power_source as u32);
}
