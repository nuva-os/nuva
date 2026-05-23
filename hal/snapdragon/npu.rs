/*
 * Nuva OS - HAL - Snapdragon
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

/// Hexagon NPU register addresses
pub mod regs {
    /// NPU base address
    pub const NPU_BASE: u64 = 0x0C00_0000;

    /// NPU control registers
    pub const NPU_CTRL: u64 = NPU_BASE + 0x0000;
    pub const NPU_STATUS: u64 = NPU_BASE + 0x0004;
    pub const NPU_FREQ: u64 = NPU_BASE + 0x0008;
    pub const NPU_VOLTAGE: u64 = NPU_BASE + 0x000C;

    /// NPU command queue
    pub const NPU_CMD_BASE: u64 = NPU_BASE + 0x1000;
    pub const NPU_CMD_WRITE: u64 = NPU_CMD_BASE + 0x0000;
    pub const NPU_CMD_READ: u64 = NPU_CMD_BASE + 0x0004;

    /// NPU memory
    pub const NPU_MEM_BASE: u64 = NPU_BASE + 0x2000;
    pub const NPU_MEM_SIZE: u64 = NPU_BASE + 0x2004;

    /// NPU performance counters
    pub const NPU_PERF_BASE: u64 = NPU_BASE + 0x3000;
    pub const NPU_PERF_CYCLES: u64 = NPU_PERF_BASE + 0x0000;
    pub const NPU_PERF_OPS: u64 = NPU_PERF_BASE + 0x0004;
}

/// NPU state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpuState {
    /// Idle
    Idle = 0,
    /// Active
    Active = 1,
    /// Suspended
    Suspended = 2,
    /// Error
    Error = 3,
}

/// NPU precision mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrecisionMode {
    /// INT8
    Int8 = 0,
    /// INT16
    Int16 = 1,
    /// FP16
    Fp16 = 2,
    /// BF16
    Bf16 = 3,
    /// Mixed precision
    Mixed = 4,
}

/// NPU features
pub struct NpuFeatures {
    /// TOPS (INT8)
    pub tops_int8: u32,
    /// TOPS (FP16)
    pub tops_fp16: u32,
    /// If transformer supported
    pub transformer_support: bool,
    /// If mixed precision supported
    pub mixed_precision: bool,
    /// If sparse computation supported
    pub sparse_support: bool,
    /// Maximum tensor dimension
    pub max_tensor_dims: u32,
    /// Local memory (MB)
    pub local_mem_mb: u32,
}

/// Hexagon NPU HAL
pub struct HexagonNpuHal {
    /// Current state
    pub state: AtomicU32,
    /// Current frequency (MHz)
    pub freq_mhz: AtomicU32,
    /// Minimum frequency (MHz)
    pub min_freq_mhz: u32,
    /// Maximum frequency (MHz)
    pub max_freq_mhz: u32,
    /// Current voltage (mV)
    pub voltage_mv: AtomicU32,
    /// Temperature (millidegrees)
    pub temp_mc: AtomicU32,
    /// Power consumption (mW)
    pub power_mw: AtomicU32,
    /// Precision mode
    pub precision: AtomicU32,
    /// Features
    pub features: NpuFeatures,
    /// Inference count
    pub inference_count: AtomicU64,
    /// Total TOPS usage
    pub total_tops: AtomicU64,
}

impl HexagonNpuHal {
    pub fn new() -> Self {
        HexagonNpuHal {
            state: AtomicU32::new(NpuState::Idle as u32),
            freq_mhz: AtomicU32::new(500),
            min_freq_mhz: 300,
            max_freq_mhz: 1500,
            voltage_mv: AtomicU32::new(700),
            temp_mc: AtomicU32::new(25000),
            power_mw: AtomicU32::new(0),
            precision: AtomicU32::new(PrecisionMode::Mixed as u32),
            features: NpuFeatures {
                tops_int8: 75,
                tops_fp16: 37,
                transformer_support: true,
                mixed_precision: true,
                sparse_support: true,
                max_tensor_dims: 8,
                local_mem_mb: 8,
            },
            inference_count: AtomicU64::new(0),
            total_tops: AtomicU64::new(0),
        }
    }

    /// Initialize
    pub fn init(&mut self) {
        log_info!("Hexagon NPU HAL initialized");
        log_info!("  TOPS: {} (INT8), {} (FP16)",
            self.features.tops_int8,
            self.features.tops_fp16
        );
        log_info!("  Transformer: supported");
        log_info!("  Mixed precision: supported");
        log_info!("  Max freq: {} MHz", self.max_freq_mhz);
    }

    /// Get state
    pub fn get_state(&self) -> NpuState {
        match self.state.load(Ordering::Acquire) {
            0 => NpuState::Idle,
            1 => NpuState::Active,
            2 => NpuState::Suspended,
            3 => NpuState::Error,
            _ => NpuState::Idle,
        }
    }

    /// Set state
    pub fn set_state(&self, state: NpuState) {
        self.state.store(state as u32, Ordering::Release);
    }

    /// Set frequency
    pub fn set_freq(&self, freq_mhz: u32) -> bool {
        if freq_mhz < self.min_freq_mhz || freq_mhz > self.max_freq_mhz {
            return false;
        }

        self.freq_mhz.store(freq_mhz, Ordering::Release);
        true
    }

    /// Set precision mode
    pub fn set_precision(&self, mode: PrecisionMode) {
        self.precision.store(mode as u32, Ordering::Release);
    }

    /// Get precision mode
    pub fn get_precision(&self) -> PrecisionMode {
        match self.precision.load(Ordering::Acquire) {
            0 => PrecisionMode::Int8,
            1 => PrecisionMode::Int16,
            2 => PrecisionMode::Fp16,
            3 => PrecisionMode::Bf16,
            4 => PrecisionMode::Mixed,
            _ => PrecisionMode::Mixed,
        }
    }

    /// Begin inference
    pub fn begin_inference(&self) {
        self.set_state(NpuState::Active);
    }

    /// End inference
    pub fn end_inference(&self, tops_used: u64) {
        self.inference_count.fetch_add(1, Ordering::AcqRel);
        self.total_tops.fetch_add(tops_used, Ordering::AcqRel);
        self.set_state(NpuState::Idle);
    }

    /// Get inference count
    pub fn get_inference_count(&self) -> u64 {
        self.inference_count.load(Ordering::Acquire)
    }

    /// Get utilization
    pub fn get_utilization(&self) -> f32 {
        let tops = self.total_tops.load(Ordering::Acquire);
        let max_tops = self.features.tops_int8 as u64;
        if max_tops == 0 {
            return 0.0;
        }
        (tops as f32) / (max_tops as f32) * 100.0
    }

    /// DVFS update
    pub fn dvfs_update(&mut self, load: u32) {
        let target_freq = if load > 80 {
            self.max_freq_mhz
        } else if load > 50 {
            (self.max_freq_mhz + self.min_freq_mhz) / 2
        } else if load > 20 {
            self.min_freq_mhz + (self.max_freq_mhz - self.min_freq_mhz) / 4
        } else {
            self.min_freq_mhz
        };

        self.set_freq(target_freq);
    }

    /// Thermal management
    pub fn thermal_update(&mut self) {
        let temp = self.read_thermal();
        self.temp_mc.store(temp, Ordering::Release);

        if temp > 85000 {
            let current = self.freq_mhz.load(Ordering::Acquire);
            if current > self.min_freq_mhz {
                self.set_freq(current - 100);
            }
        }
    }

    /// Read temperature
    fn read_thermal(&self) -> u32 {
        25000
    }

    /// Suspend
    pub fn suspend(&self) {
        self.set_state(NpuState::Suspended);
        self.set_freq(self.min_freq_mhz);
    }

    /// Resume
    pub fn resume(&self) {
        self.set_state(NpuState::Idle);
    }
}

/// Global NPU HAL
static mut NPU_HAL: Option<HexagonNpuHal> = None;

pub fn get_npu_hal() -> &'static mut HexagonNpuHal {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        if NPU_HAL.is_none() {
            NPU_HAL = Some(HexagonNpuHal::new());
        }
        NPU_HAL.as_mut().unwrap()
    }
}

pub fn init_npu_hal() {
    let hal = get_npu_hal();
    hal.init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npu_state() {
        assert_eq!(NpuState::Idle as i32, 0);
        assert_eq!(NpuState::Active as i32, 1);
        assert_eq!(NpuState::Suspended as i32, 2);
        assert_eq!(NpuState::Error as i32, 3);
    }

    #[test]
    fn test_precision_mode() {
        assert_eq!(PrecisionMode::Int8 as i32, 0);
        assert_eq!(PrecisionMode::Int16 as i32, 1);
        assert_eq!(PrecisionMode::Fp16 as i32, 2);
        assert_eq!(PrecisionMode::Bf16 as i32, 3);
        assert_eq!(PrecisionMode::Mixed as i32, 4);
    }

    #[test]
    fn test_npu_features() {
        let features = NpuFeatures {
            tops_int8: 75,
            tops_fp16: 37,
            transformer_support: true,
            mixed_precision: true,
            sparse_support: true,
            max_tensor_dims: 8,
            local_mem_mb: 8,
        };

        assert_eq!(features.tops_int8, 75);
        assert_eq!(features.tops_fp16, 37);
        assert!(features.transformer_support);
        assert_eq!(features.local_mem_mb, 8);
    }

    #[test]
    fn test_hexagon_npu_hal() {
        let hal = HexagonNpuHal::new();
        assert_eq!(hal.min_freq_mhz, 300);
        assert_eq!(hal.max_freq_mhz, 1500);
        assert_eq!(hal.features.tops_int8, 75);
    }

    #[test]
    fn test_npu_frequency() {
        let hal = HexagonNpuHal::new();
        assert!(hal.set_freq(800));
        assert_eq!(hal.get_freq(), 800);
        assert!(!hal.set_freq(200));  // Below minimum
    }

    #[test]
    fn test_npu_precision() {
        let hal = HexagonNpuHal::new();
        hal.set_precision(PrecisionMode::Int8);
        assert_eq!(hal.get_precision(), PrecisionMode::Int8);
    }
}
