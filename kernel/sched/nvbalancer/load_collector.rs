/*
 * Nuva OS - Kernel - Sched - Nvbalancer - LoadCollector
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
 * Nuva OS - Kernel - NvBalancer Load Collector
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Collects real-time load metrics from all heterogeneous
 * devices. Handles collection timeout with degraded fallback.
 */

use core::sync::atomic::{AtomicU32, AtomicU8, AtomicBool, Ordering};

use super::load_metrics::DeviceLoadMetrics;
use super::MAX_HETERO_DEVICES;

/// Load collection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CollectionState {
    /// Normal collection
    Normal = 0,
    /// Degraded (using cached values)
    Degraded = 1,
    /// Failed (all devices unreachable)
    Failed = 2,
}

/// LoadCollector: real-time device load collection
///
/// Periodically collects load metrics from all registered
/// heterogeneous devices. Falls back to last valid snapshot
/// on collection timeout.
pub struct LoadCollector {
    /// Per-device metrics array
    device_metrics: [DeviceLoadMetrics; MAX_HETERO_DEVICES],
    /// Collection period in milliseconds
    collection_period_ms: AtomicU32,
    /// Collection timeout in milliseconds
    collection_timeout_ms: AtomicU32,
    /// Current collection state
    state: AtomicU8,
    /// Last successful collection timestamp
    last_collection_tick: AtomicU32,
}

impl LoadCollector {
    /// Create a new load collector
    pub const fn new() -> Self {
        LoadCollector {
            device_metrics: [
                DeviceLoadMetrics::new(), DeviceLoadMetrics::new(),
                DeviceLoadMetrics::new(), DeviceLoadMetrics::new(),
                DeviceLoadMetrics::new(), DeviceLoadMetrics::new(),
                DeviceLoadMetrics::new(), DeviceLoadMetrics::new(),
                DeviceLoadMetrics::new(), DeviceLoadMetrics::new(),
                DeviceLoadMetrics::new(), DeviceLoadMetrics::new(),
                DeviceLoadMetrics::new(), DeviceLoadMetrics::new(),
                DeviceLoadMetrics::new(), DeviceLoadMetrics::new(),
            ],
            collection_period_ms: AtomicU32::new(10),
            collection_timeout_ms: AtomicU32::new(50),
            state: AtomicU8::new(CollectionState::Normal as u8),
            last_collection_tick: AtomicU32::new(0),
        }
    }

    /// Get metrics for a specific device
    pub fn get_metrics(&self, device_index: usize) -> Option<&DeviceLoadMetrics> {
        if device_index < MAX_HETERO_DEVICES {
            Some(&self.device_metrics[device_index])
        } else {
            None
        }
    }

    /// Update metrics for a device
    pub fn update_device(
        &self,
        device_index: usize,
        utilization: u32,
        queue_depth: u32,
        temperature: u32,
        power_mw: u32,
        data_locality: u32,
        timestamp: u32,
    ) {
        if device_index < MAX_HETERO_DEVICES {
            self.device_metrics[device_index].update(
                utilization, queue_depth, temperature, power_mw, data_locality, timestamp,
            );
        }
    }

    /// Get collection state
    pub fn state(&self) -> CollectionState {
        match self.state.load(Ordering::Acquire) {
            0 => CollectionState::Normal,
            1 => CollectionState::Degraded,
            _ => CollectionState::Failed,
        }
    }

    /// Set collection state (e.g., on timeout)
    pub fn set_state(&self, state: CollectionState) {
        self.state.store(state as u8, Ordering::Release);
    }

    /// Get collection period
    pub fn collection_period_ms(&self) -> u32 {
        self.collection_period_ms.load(Ordering::Acquire)
    }

    /// Compute average utilization across all devices
    pub fn avg_utilization(&self) -> u32 {
        let mut total = 0u32;
        let mut count = 0u32;
        for i in 0..MAX_HETERO_DEVICES {
            let util = self.device_metrics[i].get_utilization();
            if util > 0 || self.device_metrics[i].last_update.load(Ordering::Acquire) > 0 {
                total += util;
                count += 1;
            }
        }
        if count > 0 { total / count } else { 0 }
    }

    /// Compute max utilization across all devices
    pub fn max_utilization(&self) -> u32 {
        let mut max = 0u32;
        for i in 0..MAX_HETERO_DEVICES {
            let util = self.device_metrics[i].get_utilization();
            if util > max {
                max = util;
            }
        }
        max
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use core::sync::atomic::AtomicU8;

    #[test]
    fn test_update_and_get() {
        let lc = LoadCollector::new();
        lc.update_device(0, 80, 5, 450, 3000, 90, 100);
        let m = lc.get_metrics(0).unwrap();
        assert_eq!(m.get_utilization(), 80);
    }

    #[test]
    fn test_avg_utilization() {
        let lc = LoadCollector::new();
        lc.update_device(0, 60, 0, 250, 0, 50, 100);
        lc.update_device(1, 80, 0, 250, 0, 50, 100);
        assert_eq!(lc.avg_utilization(), 70);
    }

    #[test]
    fn test_collection_state() {
        let lc = LoadCollector::new();
        assert_eq!(lc.state(), CollectionState::Normal);
        lc.set_state(CollectionState::Degraded);
        assert_eq!(lc.state(), CollectionState::Degraded);
    }
}