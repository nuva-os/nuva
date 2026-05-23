/*
 * Nuva OS - Kernel - Device Detection
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


//! Device detection module
/*!*/
//! Automatically detect hardware devices, identify CPU, platform, features, etc.

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt;

use super::{DeviceInfo, DeviceType, ArchType};
use crate::{pr_info};

// ============================================================================
// CPU Detection
// ============================================================================

/// CPU information
#[derive(Debug, Clone)]
pub struct CpuInfo {
    /// Vendor
    pub vendor: CpuVendor,
    /// Model
    pub model: String,
    /// Architecture type
    pub arch_type: ArchType,
    /// Core count
    pub cores: u32,
    /// Thread count
    pub threads: u32,
    /// Frequency (Hz)
    pub frequency: u64,
    /// Cache information
    pub cache: CacheInfo,
    /// Supported features
    pub features: CpuFeatures,
}

/// CPU vendor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuVendor {
    /// Intel
    Intel,
    /// AMD
    Amd,
    /// HiSilicon
    HiSilicon,
    /// Qualcomm
    Qualcomm,
    /// Loongson
    Loongson,
    /// Phytium
    Phytium,
    /// Zhaoxin
    Zhaoxin,
    /// RISC-V
    RiscV,
    /// Unknown
    Unknown,
}

impl fmt::Display for CpuVendor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CpuVendor::Intel => write!(f, "Intel"),
            CpuVendor::Amd => write!(f, "AMD"),
            CpuVendor::HiSilicon => write!(f, "HiSilicon"),
            CpuVendor::Qualcomm => write!(f, "Qualcomm"),
            CpuVendor::Loongson => write!(f, "Loongson"),
            CpuVendor::Phytium => write!(f, "Phytium"),
            CpuVendor::Zhaoxin => write!(f, "Zhaoxin"),
            CpuVendor::RiscV => write!(f, "RISC-V"),
            CpuVendor::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Cache information
#[derive(Debug, Clone, Default)]
pub struct CacheInfo {
    /// L1 instruction cache (KB)
    pub l1i: u32,
    /// L1 data cache (KB)
    pub l1d: u32,
    /// L2 cache (KB)
    pub l2: u32,
    /// L3 cache (KB)
    pub l3: u32,
}

/// CPU features
#[derive(Debug, Clone, Default)]
pub struct CpuFeatures {
    /// SIMD features
    pub simd: SimdFeatures,
    /// Extended features
    pub extensions: Vec<String>,
}

/// SIMD features
#[derive(Debug, Clone, Copy, Default)]
pub struct SimdFeatures {
    /// Supports NEON (ARM64)
    pub neon: bool,
    /// Supports SVE (ARM64)
    pub sve: bool,
    /// Supports SSE (x86)
    pub sse: bool,
    /// Supports SSE2
    pub sse2: bool,
    /// Supports AVX (x86)
    pub avx: bool,
    /// Supports AVX2
    pub avx2: bool,
    /// Supports AVX-512
    pub avx512: bool,
    /// Supports LSX (LoongArch)
    pub lsx: bool,
    /// Supports LASX (LoongArch)
    pub lasx: bool,
}

// ============================================================================
// Platform Detection
// ============================================================================

/// Platform information
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    /// Platform name
    pub name: String,
    /// Platform vendor
    pub vendor: PlatformVendor,
    /// Platform type
    pub platform_type: PlatformType,
    /// Board information
    pub board: Option<BoardInfo>,
}

/// Platform vendor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformVendor {
    /// Huawei
    Huawei,
    /// Qualcomm
    Qualcomm,
    /// Lenovo
    Lenovo,
    /// Dell
    Dell,
    /// HP
    HP,
    /// Loongson
    Loongson,
    /// Generic
    Generic,
}

/// Platform type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformType {
    /// Phone
    Phone,
    /// Tablet
    Tablet,
    /// Laptop
    Laptop,
    /// Desktop
    Desktop,
    /// Server
    Server,
    /// Development board
    DevBoard,
    /// Embedded
    Embedded,
}

/// Board information
#[derive(Debug, Clone)]
pub struct BoardInfo {
    /// Board name
    pub name: String,
    /// Board version
    pub version: String,
}

// ============================================================================
// Device Detector
// ============================================================================

/// Device detector
pub struct DeviceDetector;

impl DeviceDetector {
    /// Detect CPU information
    pub fn detect_cpu() -> CpuInfo {
        #[cfg(target_arch = "aarch64")]
        {
            Self::detect_arm64_cpu()
        }

        #[cfg(target_arch = "x86_64")]
        {
            Self::detect_x64_cpu()
        }

        #[cfg(target_arch = "loongarch64")]
        {
            Self::detect_loongarch_cpu()
        }

        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64", target_arch = "loongarch64")))]
        {
            CpuInfo::default()
        }
    }

    /// Detect ARM64 CPU
    #[cfg(target_arch = "aarch64")]
    fn detect_arm64_cpu() -> CpuInfo {
        let mut info = CpuInfo {
            vendor: CpuVendor::Unknown,
            model: String::new(),
            arch_type: ArchType::Arm64,
            cores: 0,
            threads: 0,
            frequency: 0,
            cache: CacheInfo::default(),
            features: CpuFeatures::default(),
        };

        // Read MIDR register
        let midr: u64;
        // SAFETY: inline assembly required for hardware instruction
        unsafe {
            core::arch::asm!(
                "mrs {}, midr_el1",
                out(reg) midr,
            );
        }

        // Parse vendor (Bits 31-24)
        let implementer = (midr >> 24) & 0xFF;
        info.vendor = match implementer {
            0x41 => CpuVendor::HiSilicon, // ARM (possibly Huawei)
            0x51 => CpuVendor::Qualcomm,
            _ => CpuVendor::Unknown,
        };

        // Detect SIMD features
        info.features.simd.neon = true; // ARM64 always supports NEON

        // TODO: Detect SVE
        // TODO: Detect core count

        info
    }

    /// Detect x64 CPU
    #[cfg(target_arch = "x86_64")]
    fn detect_x64_cpu() -> CpuInfo {
        let mut info = CpuInfo {
            vendor: CpuVendor::Unknown,
            model: String::new(),
            arch_type: ArchType::X64,
            cores: 0,
            threads: 0,
            frequency: 0,
            cache: CacheInfo::default(),
            features: CpuFeatures::default(),
        };

        // Use CPUID instruction to detect
        let vendor_str: [u8; 12] = [0; 12];

        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let eax: u32;
            let ebx: u32;
            let ecx: u32;
            let edx: u32;

            // CPUID 0: Vendor string
            core::arch::asm!(
                "cpuid",
                inout("eax") 0 => eax,
                out("ebx") ebx,
                out("ecx") ecx,
                out("edx") edx,
            );

            // Parse vendor
            let vendor_bytes: [u8; 12] = [
                (ebx & 0xFF) as u8,
                ((ebx >> 8) & 0xFF) as u8,
                ((ebx >> 16) & 0xFF) as u8,
                ((ebx >> 24) & 0xFF) as u8,
                (edx & 0xFF) as u8,
                ((edx >> 8) & 0xFF) as u8,
                ((edx >> 16) & 0xFF) as u8,
                ((edx >> 24) & 0xFF) as u8,
                (ecx & 0xFF) as u8,
                ((ecx >> 8) & 0xFF) as u8,
                ((ecx >> 16) & 0xFF) as u8,
                ((ecx >> 24) & 0xFF) as u8,
            ];

            let vendor_str = core::str::from_utf8_unchecked(&vendor_bytes);
            info.vendor = match vendor_str {
                "GenuineIntel" => CpuVendor::Intel,
                "AuthenticAMD" => CpuVendor::Amd,
                "CentaurHauls" => CpuVendor::Zhaoxin,
                _ => CpuVendor::Unknown,
            };

            // CPUID 1: Features
            core::arch::asm!(
                "cpuid",
                inout("eax") 1 => eax,
                out("ebx") ebx,
                out("ecx") ecx,
                out("edx") edx,
            );

            info.features.simd.sse = (edx & (1 << 25)) != 0;
            info.features.simd.sse2 = (edx & (1 << 26)) != 0;
            info.features.simd.avx = (ecx & (1 << 28)) != 0;

            // CPUID 7: AVX2, AVX-512
            core::arch::asm!(
                "cpuid",
                inout("eax") 7 => eax,
                inout("ebx") 0 => ebx,
                out("ecx") ecx,
                out("edx") edx,
            );

            info.features.simd.avx2 = (ebx & (1 << 5)) != 0;
            info.features.simd.avx512 = (ebx & (1 << 16)) != 0;
        }

        info
    }

    /// Detect LoongArch CPU
    #[cfg(target_arch = "loongarch64")]
    fn detect_loongarch_cpu() -> CpuInfo {
        let mut info = CpuInfo {
            vendor: CpuVendor::Loongson,
            model: String::new(),
            arch_type: ArchType::LoongArch64,
            cores: 0,
            threads: 0,
            frequency: 0,
            cache: CacheInfo::default(),
            features: CpuFeatures::default(),
        };

        // Use CPUCFG instruction to detect
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // CPUCFG 0: Vendor information
            let cfg0: u32;
            core::arch::asm!(
                "cpucfg {}, $r0",
                out(reg) cfg0,
            );

            // CPUCFG 2: Feature information
            let cfg2: u32;
            core::arch::asm!(
                "cpucfg {}, $r2",
                out(reg) cfg2,
            );

            // Detect LSX/LASX
            info.features.simd.lsx = (cfg2 & (1 << 6)) != 0;
            info.features.simd.lasx = (cfg2 & (1 << 7)) != 0;
        }

        info
    }

    /// Detect platform information
    pub fn detect_platform() -> PlatformInfo {
        // TODO: Read platform information from device tree or ACPI
        PlatformInfo {
            name: "Generic".to_string(),
            vendor: PlatformVendor::Generic,
            platform_type: PlatformType::Desktop,
            board: None,
        }
    }

    /// Detect complete device information
    pub fn detect_device() -> DeviceInfo {
        let cpu = Self::detect_cpu();
        let platform = Self::detect_platform();

        let device_type = match platform.platform_type {
            PlatformType::Phone | PlatformType::Tablet => DeviceType::Mobile,
            PlatformType::Laptop | PlatformType::Desktop => DeviceType::Desktop,
            PlatformType::Server => DeviceType::Server,
            PlatformType::DevBoard | PlatformType::Embedded => DeviceType::Embedded,
        };

        let mut features = Vec::new();
        if cpu.features.simd.neon { features.push("neon".to_string()); }
        if cpu.features.simd.sve { features.push("sve".to_string()); }
        if cpu.features.simd.sse { features.push("sse".to_string()); }
        if cpu.features.simd.avx { features.push("avx".to_string()); }
        if cpu.features.simd.avx2 { features.push("avx2".to_string()); }
        if cpu.features.simd.lsx { features.push("lsx".to_string()); }
        if cpu.features.simd.lasx { features.push("lasx".to_string()); }

        DeviceInfo {
            name: platform.name,
            device_type,
            cpu_vendor: cpu.vendor.to_string(),
            cpu_model: cpu.model,
            cpu_cores: cpu.cores,
            features,
            memory_size: 0, // TODO: Detect memory
        }
    }
}

impl Default for CpuInfo {
    fn default() -> Self {
        Self {
            vendor: CpuVendor::Unknown,
            model: String::new(),
            arch_type: ArchType::Arm64,
            cores: 1,
            threads: 1,
            frequency: 0,
            cache: CacheInfo::default(),
            features: CpuFeatures::default(),
        }
    }
}

// ============================================================================
// Global Device Information
// ============================================================================

use spin::Once;

/// Global CPU information
static CPU_INFO: Once<CpuInfo> = Once::new();

/// Global platform information
static PLATFORM_INFO: Once<PlatformInfo> = Once::new();

/// Initialize device detection
pub fn init_device_detection() {
    let cpu = DeviceDetector::detect_cpu();
    CPU_INFO.call_once(|| cpu);

    let platform = DeviceDetector::detect_platform();
    PLATFORM_INFO.call_once(|| platform);

    let cpu = CPU_INFO.get().unwrap();
    log_info!("Device detection initialized");
    log_info!("  CPU: {} {}", cpu.vendor, cpu.model);
    log_info!("  Cores: {} threads: {}", cpu.cores, cpu.threads);
}

/// Get CPU information
pub fn get_cpu_info() -> Option<&'static CpuInfo> {
    CPU_INFO.get()
}

/// Get platform information
pub fn get_platform_info() -> Option<&'static PlatformInfo> {
    PLATFORM_INFO.get()
}
