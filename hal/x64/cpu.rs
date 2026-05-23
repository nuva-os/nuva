/*
 * Nuva OS - HAL - X64
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



use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// X64 MSR registers
pub mod msr {
    /// APIC base address
    pub const IA32_APIC_BASE: u32 = 0x1B;
    /// Feature control
    pub const IA32_FEATURE_CONTROL: u32 = 0x3A;
    /// Temperature target
    pub const IA32_TEMPERATURE_TARGET: u32 = 0x1A2;
    /// Performance status
    pub const IA32_PERF_STATUS: u32 = 0x198;
    pub const IA32_PERF_CTL: u32 = 0x199;
    /// Energy performance preference
    pub const IA32_ENERGY_PERF_BIAS: u32 = 0x1B0;
}

/// CPU vendor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuVendor {
    /// Intel
    Intel,
    /// AMD
    Amd,
    /// Unknown
    Unknown,
}

/// CPU feature flags
pub struct CpuFeaturesX64 {
    /// If FPU supported
    pub has_fpu: bool,
    /// If VME supported
    pub has_vme: bool,
    /// If DE supported
    pub has_de: bool,
    /// If PSE supported
    pub has_pse: bool,
    /// If PAE supported
    pub has_pae: bool,
    /// If APIC supported
    pub has_apic: bool,
    /// If SSE supported
    pub has_sse: bool,
    /// If SSE2 supported
    pub has_sse2: bool,
    /// If SSE3 supported
    pub has_sse3: bool,
    /// If SSSE3 supported
    pub has_ssse3: bool,
    /// If SSE4.1 supported
    pub has_sse41: bool,
    /// If SSE4.2 supported
    pub has_sse42: bool,
    /// If AVX supported
    pub has_avx: bool,
    /// If AVX2 supported
    pub has_avx2: bool,
    /// If AVX-512F supported
    pub has_avx512f: bool,
    /// If AVX-512DQ supported
    pub has_avx512dq: bool,
    /// If AVX-512BW supported
    pub has_avx512bw: bool,
    /// If AVX-512VL supported
    pub has_avx512vl: bool,
    /// If FMA supported
    pub has_fma: bool,
    /// If BMI supported
    pub has_bmi: bool,
    /// If BMI2 supported
    pub has_bmi2: bool,
    /// If AES supported
    pub has_aes: bool,
    /// If SHA supported
    pub has_sha: bool,
    /// If RDRAND supported
    pub has_rdrand: bool,
    /// If RDSEED supported
    pub has_rdseed: bool,
    /// If FSGSBASE supported
    pub has_fsgsbase: bool,
    /// If PCID supported
    pub has_pcid: bool,
    /// If XSAVE supported
    pub has_xsave: bool,
    /// If OSXSAVE supported
    pub has_osxsave: bool,
    /// If NX supported
    pub has_nx: bool,
    /// If SMEP supported
    pub has_smep: bool,
    /// If SMAP supported
    pub has_smap: bool,
}

/// CPU core
pub struct X64CpuCore {
    /// Core ID
    pub id: u32,
    /// APIC ID
    pub apic_id: u32,
    /// Current frequency (MHz)
    pub freq_mhz: AtomicU32,
    /// Minimum frequency (MHz)
    pub min_freq_mhz: u32,
    /// Maximum frequency (MHz)
    pub max_freq_mhz: u32,
    /// If online
    pub online: AtomicU32,
    /// Temperature (millidegrees)
    pub temp_mc: AtomicU32,
}

impl X64CpuCore {
    pub fn new(id: u32, apic_id: u32) -> Self {
        X64CpuCore {
            id,
            apic_id,
            freq_mhz: AtomicU32::new(800),
            min_freq_mhz: 800,
            max_freq_mhz: 5000,
            online: AtomicU32::new(1),
            temp_mc: AtomicU32::new(25000),
        }
    }

    /// Set frequency
    pub fn set_freq(&self, freq_mhz: u32) -> bool {
        if freq_mhz < self.min_freq_mhz || freq_mhz > self.max_freq_mhz {
            return false;
        }

        // Write MSR to set frequency
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let perf_ctl = (freq_mhz as u64) << 8;
            msr_write(msr::IA32_PERF_CTL, perf_ctl);
        }
        self.freq_mhz.store(freq_mhz, Ordering::Release);
        true
    }

    /// Get frequency
    pub fn get_freq(&self) -> u32 {
        self.freq_mhz.load(Ordering::Acquire)
    }
}

/// X64 CPU HAL
pub struct X64CpuHal {
    /// Vendor
    pub vendor: CpuVendor,
    /// CPU name
    pub name: &'static str,
    /// Number of cores
    pub cores: u32,
    /// Core array
    pub core_list: [Option<X64CpuCore>; 64],
    /// Features
    pub features: CpuFeaturesX64,
    /// Total temperature
    pub total_temp_mc: AtomicU32,
    /// Total power consumption (mW)
    pub total_power_mw: AtomicU32,
}

impl X64CpuHal {
    pub fn new() -> Self {
        X64CpuHal {
            vendor: CpuVendor::Unknown,
            name: "Unknown x86-64",
            cores: 4,
            core_list: [None; 64],
            features: CpuFeaturesX64 {
                has_fpu: true,
                has_vme: true,
                has_de: true,
                has_pse: true,
                has_pae: true,
                has_apic: true,
                has_sse: true,
                has_sse2: true,
                has_sse3: true,
                has_ssse3: true,
                has_sse41: true,
                has_sse42: true,
                has_avx: true,
                has_avx2: true,
                has_avx512f: false,
                has_avx512dq: false,
                has_avx512bw: false,
                has_avx512vl: false,
                has_fma: true,
                has_bmi: true,
                has_bmi2: true,
                has_aes: true,
                has_sha: false,
                has_rdrand: true,
                has_rdseed: true,
                has_fsgsbase: true,
                has_pcid: true,
                has_xsave: true,
                has_osxsave: true,
                has_nx: true,
                has_smep: true,
                has_smap: true,
            },
            total_temp_mc: AtomicU32::new(25000),
            total_power_mw: AtomicU32::new(0),
        }
    }

    /// Initialize
    pub fn init(&mut self) {
        // CPUID detection
        self.detect_cpu();

        log_info!("X64 CPU HAL initialized");
        log_info!("  Vendor: {:?}", self.vendor);
        log_info!("  CPU: {}", self.name);
        log_info!("  Cores: {}", self.cores);
        log_info!("  Features: SSE, SSE2, SSE3, SSSE3, SSE4.1, SSE4.2");
        if self.features.has_avx {
            log_info!("           AVX");
        }
        if self.features.has_avx2 {
            log_info!("           AVX2");
        }
        if self.features.has_avx512f {
            log_info!("           AVX-512");
        }
    }

    /// Detect CPU
    fn detect_cpu(&mut self) {
        // Use CPUID instruction to detect CPU information
        let (vendor, name, features) = self.cpuid_detect();

        self.vendor = vendor;
        self.name = name;
        self.features = features;

        // Detect core count
        self.cores = self.detect_core_count();

        // Initialize core list
        for i in 0..self.cores.min(64) {
            let apic_id = if i == 0 {
                self.get_local_apic_id()
            } else {
                i // Simplified handling, actual should get from ACPI/MP tables
            };
            self.core_list[i as usize] = Some(X64CpuCore::new(i, apic_id));
        }
    }

    /// CPUID detection
    fn cpuid_detect(&self) -> (CpuVendor, &'static str, CpuFeaturesX64) {
        // Read vendor ID
        let vendor_str = self.cpuid_vendor_string();

        let vendor = if vendor_str.iter().take(12).copied().collect::<Vec<u8>>().windows(12).any(|w| w == b"GenuineIntel") {
            CpuVendor::Intel
        } else if vendor_str.iter().take(12).copied().collect::<Vec<u8>>().windows(12).any(|w| w == b"AuthenticAMD") {
            CpuVendor::Amd
        } else {
            CpuVendor::Unknown
        };

        // Read CPU name
        let name = self.cpuid_processor_name();

        // Detect features
        let features = self.cpuid_features();

        (vendor, name, features)
    }

    /// CPUID read vendor string
    fn cpuid_vendor_string(&self) -> [u8; 12] {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let mut vendor = [0u8; 12];
            let result = cpuid(0);

            // EBX, EDX, ECX contain vendor string
            let ebx = result.ebx;
            let edx = result.edx;
            let ecx = result.ecx;

            vendor[0..4].copy_from_slice(&(ebx as u32).to_le_bytes());
            vendor[4..8].copy_from_slice(&(edx as u32).to_le_bytes());
            vendor[8..12].copy_from_slice(&(ecx as u32).to_le_bytes());

            vendor
        }
    }

    /// CPUID read processor name
    fn cpuid_processor_name(&self) -> &'static str {
        // Simplified handling here, actual should read from CPUID 0x80000002-0x80000004
        match self.vendor {
            CpuVendor::Intel => "Intel x86-64 Processor",
            CpuVendor::Amd => "AMD x86-64 Processor",
            CpuVendor::Unknown => "Unknown x86-64 Processor",
        }
    }

    /// CPUID detect features
    fn cpuid_features(&self) -> CpuFeaturesX64 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let result1 = cpuid(1);
            let result7 = cpuid(7);

            CpuFeaturesX64 {
                has_fpu: (result1.edx & (1 << 0)) != 0,
                has_vme: (result1.edx & (1 << 1)) != 0,
                has_de: (result1.edx & (1 << 2)) != 0,
                has_pse: (result1.edx & (1 << 3)) != 0,
                has_pae: (result1.edx & (1 << 6)) != 0,
                has_apic: (result1.edx & (1 << 9)) != 0,
                has_sse: (result1.edx & (1 << 25)) != 0,
                has_sse2: (result1.edx & (1 << 26)) != 0,
                has_sse3: (result1.ecx & (1 << 0)) != 0,
                has_ssse3: (result1.ecx & (1 << 9)) != 0,
                has_sse41: (result1.ecx & (1 << 19)) != 0,
                has_sse42: (result1.ecx & (1 << 20)) != 0,
                has_avx: (result1.ecx & (1 << 28)) != 0,
                has_avx2: (result7.ebx & (1 << 5)) != 0,
                has_avx512f: (result7.ebx & (1 << 16)) != 0,
                has_avx512dq: (result7.ebx & (1 << 17)) != 0,
                has_avx512bw: (result7.ebx & (1 << 30)) != 0,
                has_avx512vl: (result7.ecx & (1 << 1)) != 0,
                has_fma: (result1.ecx & (1 << 12)) != 0,
                has_bmi: (result7.ebx & (1 << 3)) != 0,
                has_bmi2: (result7.ebx & (1 << 8)) != 0,
                has_aes: (result1.ecx & (1 << 25)) != 0,
                has_sha: (result7.ebx & (1 << 29)) != 0,
                has_rdrand: (result1.ecx & (1 << 30)) != 0,
                has_rdseed: (result7.ebx & (1 << 18)) != 0,
                has_fsgsbase: (result7.ebx & (1 << 0)) != 0,
                has_pcid: (result1.ecx & (1 << 17)) != 0,
                has_xsave: (result1.ecx & (1 << 26)) != 0,
                has_osxsave: (result1.ecx & (1 << 27)) != 0,
                has_nx: (result1.edx & (1 << 20)) != 0,
                has_smep: (result7.ebx & (1 << 7)) != 0,
                has_smap: (result7.ebx & (1 << 20)) != 0,
            }
        }
    }

    /// Detect core count
    fn detect_core_count(&self) -> u32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let result = cpuid(0xb);
            if result.eax != 0 {
                // Use CPUID 0xB to get topology information
                let level_type = (result.ecx & 0xFF00) >> 8;
                if level_type == 1 {
                    // SMT level
                    result.ebx as u32
                } else if level_type == 2 {
                    // Core level
                    result.ebx as u32
                } else {
                    4 // Default value
                }
            } else {
                // Use CPUID 1 fallback method
                let result = cpuid(1);
                let logical_processors = (result.ebx >> 16) & 0xFF;
                if logical_processors > 0 {
                    logical_processors as u32
                } else {
                    4 // Default value
                }
            }
        }
    }

    /// Get local APIC ID
    fn get_local_apic_id(&self) -> u32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            let result = cpuid(1);
            ((result.ebx >> 24) & 0xFF) as u32
        }
    }

    /// Get core
    pub fn get_core(&self, id: u32) -> Option<&X64CpuCore> {
        if (id as usize) < self.core_list.len() {
            self.core_list[id as usize].as_ref()
        } else {
            None
        }
    }

    /// Get online core count
    pub fn get_online_count(&self) -> u32 {
        self.core_list.iter()
            .filter(|c| c.is_some())
            .filter(|c| c.as_ref().unwrap().online.load(Ordering::Acquire) != 0)
            .count() as u32
    }

    /// DVFS update
    pub fn dvfs_update(&mut self, load: u32) {
        for slot in self.core_list.iter_mut() {
            if let Some(ref core) = slot {
                if core.online.load(Ordering::Acquire) == 0 {
                    continue;
                }

                let target_freq = if load > 80 {
                    core.max_freq_mhz
                } else if load > 50 {
                    (core.max_freq_mhz + core.min_freq_mhz) / 2
                } else if load > 20 {
                    core.min_freq_mhz + (core.max_freq_mhz - core.min_freq_mhz) / 4
                } else {
                    core.min_freq_mhz
                };

                core.set_freq(target_freq);
            }
        }
    }

    /// Thermal management
    pub fn thermal_update(&mut self) {
        let temp = self.read_thermal();
        self.total_temp_mc.store(temp, Ordering::Release);

        if temp > 95000 {  // 95°C
            for slot in self.core_list.iter_mut() {
                if let Some(ref core) = slot {
                    let current = core.get_freq();
                    if current > core.min_freq_mhz {
                        core.set_freq(current - 200);
                    }
                }
            }
        }
    }

    /// Read temperature
    fn read_thermal(&self) -> u32 {
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            // Read MSR temperature register
            let temp_target = msr_read(msr::IA32_TEMPERATURE_TARGET);
            let temp_offset = (temp_target >> 16) & 0x3F; // Temperature offset

            // Read digital temperature sensor
            let pkg_temp = msr_read(0x1A6); // IA32_PACKAGE_THERM_STATUS

            if (pkg_temp & (1 << 31)) != 0 {
                // Temperature read valid
                let temp = (pkg_temp >> 16) & 0x7F;
                // Convert to millidegrees Celsius
                ((temp - temp_offset) * 1000) as u32 + 25000
            } else {
                25000 // Default 25°C
            }
        }
    }

    /// Get temperature
    pub fn get_temp(&self) -> u32 {
        self.total_temp_mc.load(Ordering::Acquire)
    }

    /// Get power consumption
    pub fn get_power(&self) -> u32 {
        self.total_power_mw.load(Ordering::Acquire)
    }
}

/// Global CPU HAL
static mut CPU_HAL: Option<X64CpuHal> = None;

pub fn get_cpu_hal() -> &'static mut X64CpuHal {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        if CPU_HAL.is_none() {
            CPU_HAL = Some(X64CpuHal::new());
        }
        CPU_HAL.as_mut().unwrap()
    }
}

pub fn init_cpu_hal() {
    let hal = get_cpu_hal();
    hal.init();
}

/// CPUID result structure
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CpuidResult {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

/// Execute CPUID instruction
#[inline]
pub unsafe fn cpuid(leaf: u32) -> CpuidResult {
    let mut eax: u32;
    let mut ebx: u32;
    let mut ecx: u32;
    let mut edx: u32;

    core::arch::asm!(
        "cpuid",
        inlateout("eax") leaf => eax,
        out("ebx") ebx,
        out("ecx") ecx,
        out("edx") edx,
        options(nomem, nostack)
    );

    CpuidResult { eax, ebx, ecx, edx }
}

/// Read MSR register
#[inline]
pub unsafe fn msr_read(msr: u32) -> u64 {
    let mut high: u32;
    let mut low: u32;

    core::arch::asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") low,
        out("edx") high,
        options(nomem, nostack)
    );

    ((high as u64) << 32) | (low as u64)
}

/// Write MSR register
#[inline]
pub unsafe fn msr_write(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;

    core::arch::asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") low,
        in("edx") high,
        options(nomem, nostack)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_vendor() {
        assert_eq!(CpuVendor::Intel as i32, 0);
        assert_eq!(CpuVendor::Amd as i32, 1);
        assert_eq!(CpuVendor::Unknown as i32, 2);
    }

    #[test]
    fn test_x64_cpu_core() {
        let core = X64CpuCore::new(0, 0);
        assert_eq!(core.id, 0);
        assert_eq!(core.apic_id, 0);
        assert_eq!(core.min_freq_mhz, 800);
        assert_eq!(core.max_freq_mhz, 5000);
    }

    #[test]
    fn test_x64_cpu_hal() {
        let hal = X64CpuHal::new();
        assert_eq!(hal.cores, 4);
        assert!(hal.features.has_sse);
        assert!(hal.features.has_avx);
    }

    #[test]
    fn test_cpuid_result() {
        let result = CpuidResult {
            eax: 0,
            ebx: 0,
            ecx: 0,
            edx: 0,
        };
        assert_eq!(result.eax, 0);
    }
}
