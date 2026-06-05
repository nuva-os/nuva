/*
 * Nuva OS - Kernel - Sched - Nvbalancer - LoadMetrics
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
 * Nuva OS - Kernel - NvBalancer Load Metrics
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Per-device real-time load metrics with atomic updates.
 */

use core::sync::atomic::{AtomicU32, Ordering};

/// DeviceLoadMetrics: per-device real-time load indicators
///
/// All fields use AtomicU32 for lock-free concurrent updates
/// from different CPU cores.
pub struct DeviceLoadMetrics {
    /// Device utilization (0-100 percentage)
    pub utilization: AtomicU32,
    /// Queue depth (number of pending operations)
    pub queue_depth: AtomicU32,
    /// Temperature (degrees Celsius, scaled by 10)
    pub temperature: AtomicU32,
    /// Power consumption (milliwatts)
    pub power_mw: AtomicU32,
    /// Data locality score (0-100, higher = better locality)
    pub data_locality: AtomicU32,
    /// Timestamp of last update (kernel ticks)
    pub last_update: AtomicU32,
}

impl DeviceLoadMetrics {
    /// Create zero-initialized metrics
    pub const fn new() -> Self {
        DeviceLoadMetrics {
            utilization: AtomicU32::new(0),
            queue_depth: AtomicU32::new(0),
            temperature: AtomicU32::new(250),
            power_mw: AtomicU32::new(0),
            data_locality: AtomicU32::new(50),
            last_update: AtomicU32::new(0),
        }
    }

    /// Update all metrics atomically
    pub fn update(&self, utilization: u32, queue_depth: u32, temperature: u32, power_mw: u32, data_locality: u32, timestamp: u32) {
        self.utilization.store(utilization.min(100), Ordering::Release);
        self.queue_depth.store(queue_depth, Ordering::Release);
        self.temperature.store(temperature, Ordering::Release);
        self.power_mw.store(power_mw, Ordering::Release);
        self.data_locality.store(data_locality.min(100), Ordering::Release);
        self.last_update.store(timestamp, Ordering::Release);
    }

    /// Get current utilization
    #[inline(always)]
    pub fn get_utilization(&self) -> u32 {
        self.utilization.load(Ordering::Acquire)
    }

    /// Get current queue depth
    #[inline(always)]
    pub fn get_queue_depth(&self) -> u32 {
        self.queue_depth.load(Ordering::Acquire)
    }

    /// Get temperature in degrees Celsius
    #[inline(always)]
    pub fn temperature_c(&self) -> u32 {
        self.temperature.load(Ordering::Acquire) / 10
    }

    /// Get power consumption in milliwatts
    #[inline(always)]
    pub fn power_mw(&self) -> u32 {
        self.power_mw.load(Ordering::Acquire)
    }

    /// Check if metrics are stale (not updated within threshold ticks)
    pub fn is_stale(&self, current_tick: u32, threshold_ticks: u32) -> bool {
        let last = self.last_update.load(Ordering::Acquire);
        current_tick.saturating_sub(last) > threshold_ticks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_update() {
        let m = DeviceLoadMetrics::new();
        m.update(75, 10, 450, 5000, 80, 1000);
        assert_eq!(m.get_utilization(), 75);
        assert_eq!(m.get_queue_depth(), 10);
        assert_eq!(m.temperature_c(), 45);
        assert_eq!(m.power_mw(), 5000);
    }

    #[test]
    fn test_utilization_clamped() {
        let m = DeviceLoadMetrics::new();
        m.update(150, 0, 250, 0, 50, 0);
        assert_eq!(m.get_utilization(), 100);
    }

    #[test]
    fn test_stale_check() {
        let m = DeviceLoadMetrics::new();
        m.update(50, 0, 250, 0, 50, 100);
        assert!(!m.is_stale(150, 100));
        assert!(m.is_stale(250, 100));
    }
}