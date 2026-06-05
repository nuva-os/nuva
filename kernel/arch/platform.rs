use crate::{pr_info};
/*
 * Nuva OS - Kernel - Arch
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


/// CPU ArchitectureType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    /// ARM64 (AArch64)
    ARM64,
    /// X86-64
    X64,
    /// LoongArch64
    LoongArch64,
    /// RISC-V 64-bit (RV64G)
    RiscV64,
}

/// SoC PlatformType
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// Huawei Kirin9020
    Kirin9020,
    /// Qualcomm Snapdragon 8 Gen 4
    Snapdragon8Gen4,
    /// General X64
    GenericX64,
    /// Intel Core
    IntelCore,
    /// AMD Ryzen
    AmdRyzen,
    /// Loongson 3A6000
    Loongson3A6000,
    /// Loongson 3C6000
    Loongson3C6000,
    /// QEMU virt machine (RISC-V)
    QemuVirtRiscV,
}

/// CPU Feature
pub struct CpuFeatures {
    /// Support NEON
    pub has_neon: bool,
    /// Support SVE
    pub has_sve: bool,
    /// Support SVE2
    pub has_sve2: bool,
    /// Support AES
    pub has_aes: bool,
    /// Support SHA
    pub has_sha: bool,
    /// Support CRC32
    pub has_crc32: bool,
    /// Support AVX
    pub has_avx: bool,
    /// Support AVX2
    pub has_avx2: bool,
    /// Support AVX-512
    pub has_avx512: bool,
    /// Support SSE
    pub has_sse: bool,
    /// Support SSE2
    pub has_sse2: bool,
    /// Support SSE3
    pub has_sse3: bool,
    /// Support SSSE3
    pub has_ssse3: bool,
    /// Support SSE4.1
    pub has_sse41: bool,
    /// Support SSE4.2
    pub has_sse42: bool,
    /// Support FMA
    pub has_fma: bool,
    /// Support BMI
    pub has_bmi: bool,
    /// Support BMI2
    pub has_bmi2: bool,
    /// Support RISC-V vector extension (V)
    pub has_v: bool,
    /// Support RV64G baseline
    pub has_rv64g: bool,
}

/// CPU Info
pub struct CpuInfo {
    /// Architecture
    pub arch: Arch,
    /// Platform
    pub platform: Platform,
    /// CPU Name
    pub name: &'static str,
    /// Number of cores
    pub cores: u32,
    /// Number of big cores
    pub big_cores: u32,
    /// Number of little cores
    pub little_cores: u32,
    /// Frequency (MHz)
    pub freq_mhz: u32,
    /// MaxFrequency (MHz)
    pub max_freq_mhz: u32,
    /// L1 InstructionCaching (KB)
    pub l1_icache: u32,
    /// L1 Data Caching (KB)
    pub l1_dcache: u32,
    /// L2 Caching (KB)
    pub l2_cache: u32,
    /// L3 Caching (KB)
    pub l3_cache: u32,
    /// Feature
    pub features: CpuFeatures,
}

/// Kirin9020 CPU Info
pub const KIRIN9020_INFO: CpuInfo = CpuInfo {
    arch: Arch::ARM64,
    platform: Platform::Kirin9020,
    name: "HiSilicon Kirin 9020",
    cores: 8,
    big_cores: 4,
    little_cores: 4,
    freq_mhz: 3100,
    max_freq_mhz: 3100,
    l1_icache: 64,
    l1_dcache: 64,
    l2_cache: 512,
    l3_cache: 4096,
    features: CpuFeatures {
        has_neon: true,
        has_sve: true,
        has_sve2: true,
        has_aes: true,
        has_sha: true,
        has_crc32: true,
        has_avx: false,
        has_avx2: false,
        has_avx512: false,
        has_sse: false,
        has_sse2: false,
        has_sse3: false,
        has_ssse3: false,
        has_sse41: false,
        has_sse42: false,
        has_fma: true,
        has_bmi: false,
        has_bmi2: false,
        has_v: false,
        has_rv64g: false,
    },
};

/// Snapdragon 8 Gen 4 CPU Info
pub const SNAPDRAGON8GEN4_INFO: CpuInfo = CpuInfo {
    arch: Arch::ARM64,
    platform: Platform::Snapdragon8Gen4,
    name: "Qualcomm Snapdragon 8 Gen 4",
    cores: 8,
    big_cores: 2,   // Oryon exceedlargekernel
    little_cores: 6, // Cortex-A720
    freq_mhz: 4090,
    max_freq_mhz: 4090,
    l1_icache: 128,
    l1_dcache: 128,
    l2_cache: 1024,
    l3_cache: 8192,
    features: CpuFeatures {
        has_neon: true,
        has_sve: true,
        has_sve2: true,
        has_aes: true,
        has_sha: true,
        has_crc32: true,
        has_avx: false,
        has_avx2: false,
        has_avx512: false,
        has_sse: false,
        has_sse2: false,
        has_sse3: false,
        has_ssse3: false,
        has_sse41: false,
        has_sse42: false,
        has_fma: true,
        has_bmi: false,
        has_bmi2: false,
        has_v: false,
        has_rv64g: false,
    },
};

/// General X64 CPU Info
pub const GENERIC_X64_INFO: CpuInfo = CpuInfo {
    arch: Arch::X64,
    platform: Platform::GenericX64,
    name: "Generic x86-64",
    cores: 4,
    big_cores: 4,
    little_cores: 0,
    freq_mhz: 3000,
    max_freq_mhz: 3000,
    l1_icache: 32,
    l1_dcache: 32,
    l2_cache: 256,
    l3_cache: 8192,
    features: CpuFeatures {
        has_neon: false,
        has_sve: false,
        has_sve2: false,
        has_aes: true,
        has_sha: false,
        has_crc32: false,
        has_avx: true,
        has_avx2: true,
        has_avx512: false,
        has_sse: true,
        has_sse2: true,
        has_sse3: true,
        has_ssse3: true,
        has_sse41: true,
        has_sse42: true,
        has_fma: true,
        has_bmi: true,
        has_bmi2: true,
        has_v: false,
        has_rv64g: false,
    },
};

/// QEMU virt RISC-V CPU Info
pub const QEMU_VIRT_RISCV_INFO: CpuInfo = CpuInfo {
    arch: Arch::RiscV64,
    platform: Platform::QemuVirtRiscV,
    name: "QEMU virt RISC-V 64",
    cores: 1,
    big_cores: 1,
    little_cores: 0,
    freq_mhz: 1000,
    max_freq_mhz: 1000,
    l1_icache: 32,
    l1_dcache: 32,
    l2_cache: 0,
    l3_cache: 0,
    features: CpuFeatures {
        has_neon: false,
        has_sve: false,
        has_sve2: false,
        has_aes: false,
        has_sha: false,
        has_crc32: false,
        has_avx: false,
        has_avx2: false,
        has_avx512: false,
        has_sse: false,
        has_sse2: false,
        has_sse3: false,
        has_ssse3: false,
        has_sse41: false,
        has_sse42: false,
        has_fma: false,
        has_bmi: false,
        has_bmi2: false,
        has_v: true,
        has_rv64g: true,
    },
};

/// PlatformManager
pub struct PlatformManager {
    /// CurrentPlatform
    pub current: Platform,
    /// CPU Info
    pub cpu_info: &'static CpuInfo,
}

impl PlatformManager {
    pub const fn new() -> Self {
        PlatformManager {
            current: Platform::Kirin9020,
            cpu_info: &KIRIN9020_INFO,
        }
    }
    
    /// InitializePlatform
    pub fn init(&self) {
        log_info!("Platform: {:?}", self.current);
        log_info!("CPU: {}", self.cpu_info.name);
        log_info!("Cores: {} ({} big + {} little)", 
            self.cpu_info.cores,
            self.cpu_info.big_cores,
            self.cpu_info.little_cores
        );
        log_info!("Frequency: {} MHz", self.cpu_info.freq_mhz);
        log_info!("Cache: L1={}KB L2={}KB L3={}KB",
            self.cpu_info.l1_icache + self.cpu_info.l1_dcache,
            self.cpu_info.l2_cache,
            self.cpu_info.l3_cache
        );
    }
    
    /// SetPlatform
    pub fn set_platform(&mut self, platform: Platform) {
        self.current = platform;
        self.cpu_info = match platform {
            Platform::Kirin9020 => &KIRIN9020_INFO,
            Platform::Snapdragon8Gen4 => &SNAPDRAGON8GEN4_INFO,
            Platform::GenericX64 => &GENERIC_X64_INFO,
            Platform::IntelCore => &GENERIC_X64_INFO,
            Platform::AmdRyzen => &GENERIC_X64_INFO,
            Platform::Loongson3A6000 => &GENERIC_X64_INFO,
            Platform::Loongson3C6000 => &GENERIC_X64_INFO,
            Platform::QemuVirtRiscV => &QEMU_VIRT_RISCV_INFO,
        };
    }
    
    /// GetArchitecture
    pub fn get_arch(&self) -> Arch {
        self.cpu_info.arch
    }
    
    /// Check if is ARM64
    pub fn is_arm64(&self) -> bool {
        self.cpu_info.arch == Arch::ARM64
    }
    
    /// Check if is X64
    pub fn is_x64(&self) -> bool {
        self.cpu_info.arch == Arch::X64
    }
    
    /// Support NEON
    pub fn has_neon(&self) -> bool {
        self.cpu_info.features.has_neon
    }
    
    /// Support AVX2
    pub fn has_avx2(&self) -> bool {
        self.cpu_info.features.has_avx2
    }
}

/// Global PlatformManager
static PLATFORM_MANAGER: core::sync::OnceLock<PlatformManager> = core::sync::OnceLock::new();

pub fn platform_manager() -> &'static PlatformManager {
    PLATFORM_MANAGER.get_or_init(PlatformManager::new)
}

pub fn init_platform_manager() -> &'static PlatformManager {
    PLATFORM_MANAGER.get_or_init(PlatformManager::new)
}

pub fn init_platform() {
    let manager = platform_manager();
    manager.init();
}