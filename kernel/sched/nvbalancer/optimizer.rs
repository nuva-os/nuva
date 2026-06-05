/*
 * Nuva OS - Kernel - Sched - Nvbalancer - Optimizer
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
 * Nuva OS - Kernel - NvBalancer Balance Optimizer
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Core load balancing algorithm: evaluates task-device
 * matching, data locality, and power efficiency to
 * generate optimal migration plans.
 */

use super::topology::HeteroDeviceTopology;
use super::load_collector::LoadCollector;
use super::migration_entry::{MigrationEntry, BalanceDecision};
use super::device_types::NvHeteroDeviceType;

/// Task characteristics for matching
#[derive(Clone, Debug)]
pub struct TaskCharacteristics {
    /// Task ID
    pub task_id: u32,
    /// Compute ratio (0-100)
    pub compute_ratio: u32,
    /// Memory usage in MB
    pub memory_mb: u32,
    /// Whether task uses NPU
    pub has_npu_access: bool,
    /// Current device index
    pub current_device: usize,
}

impl TaskCharacteristics {
    /// Create new task characteristics
    pub const fn new(task_id: u32, compute_ratio: u32, memory_mb: u32, has_npu_access: bool, current_device: usize) -> Self {
        TaskCharacteristics { task_id, compute_ratio, memory_mb, has_npu_access, current_device }
    }
}

/// BalanceOptimizer: core balancing algorithm
///
/// Evaluates load imbalance and generates migration plans:
/// 1. Compute load deviation (max - min)
/// 2. If deviation > trigger threshold, initiate balancing
/// 3. Evaluate task-device matching + data locality + power efficiency
/// 4. Generate migration plan
/// 5. Output BalanceDecision
pub struct BalanceOptimizer;

impl BalanceOptimizer {
    /// Evaluate task-device matching score
    ///
    /// Higher score indicates better match between task
    /// characteristics and device capabilities.
    ///
    /// @param task: Task characteristics
    /// @param device_type: Target device type
    /// @param compute_score: Device compute score
    /// @param memory_bw: Device memory bandwidth
    /// @return: Matching score (0-100)
    pub fn task_device_match_score(
        task: &TaskCharacteristics,
        device_type: NvHeteroDeviceType,
        compute_score: u32,
        memory_bw: u32,
    ) -> u32 {
        let compute_match = if task.compute_ratio > 70 {
            match device_type {
                NvHeteroDeviceType::GpuRtxSpark | NvHeteroDeviceType::NpuDavinci => 100,
                NvHeteroDeviceType::CpuCluster => 40,
                _ => 30,
            }
        } else if task.compute_ratio > 40 {
            match device_type {
                NvHeteroDeviceType::CpuCluster => 80,
                NvHeteroDeviceType::NpuDavinci => 70,
                _ => 50,
            }
        } else {
            60
        };

        let npu_match = if task.has_npu_access {
            match device_type {
                NvHeteroDeviceType::NpuDavinci => 100,
                NvHeteroDeviceType::GpuRtxSpark => 60,
                _ => 20,
            }
        } else {
            50
        };

        let memory_match = if task.memory_mb > 100 && memory_bw > 100_000 {
            80
        } else if task.memory_mb > 50 {
            60
        } else {
            40
        };

        let capacity_factor = if compute_score > 0 { compute_score.min(1000) } else { 1 };

        (compute_match * 4 + npu_match * 3 + memory_match * 2 + capacity_factor / 10) / 10
    }

    /// Evaluate data locality score
    ///
    /// @param numa_src: Source NUMA node
    /// @param numa_dst: Destination NUMA node
    /// @param pcie_bw: PCIe bandwidth between devices (MB/s)
    /// @return: Locality score (0-100, higher = better)
    pub fn data_locality_score(numa_src: u32, numa_dst: u32, pcie_bw: u32) -> u32 {
        let numa_score = if numa_src == numa_dst {
            100
        } else {
            50
        };

        let bw_score = if pcie_bw > 500_000 {
            100
        } else if pcie_bw > 100_000 {
            70
        } else if pcie_bw > 10_000 {
            40
        } else {
            10
        };

        (numa_score + bw_score) / 2
    }

    /// Generate balance decision for a set of tasks
    ///
    /// @param tasks: Tasks to consider for migration
    /// @param topology: Current device topology
    /// @param collector: Current load metrics
    /// @param trigger_pct: Imbalance trigger threshold
    /// @param balance_pct: Balance target threshold
    /// @return: Balance decision with migration plan
    pub fn optimize(
        tasks: &[TaskCharacteristics],
        topology: &HeteroDeviceTopology,
        collector: &LoadCollector,
        trigger_pct: u32,
        balance_pct: u32,
    ) -> BalanceDecision {
        let max_load = topology.max_load();
        let min_load = topology.min_load();

        let deviation = if max_load > 0 {
            ((max_load - min_load) * 100) / max_load
        } else {
            0
        };

        if deviation < trigger_pct {
            return BalanceDecision::balanced(deviation);
        }

        let mut migrations = alloc::vec::Vec::new();
        let mut step = 0u32;

        for task in tasks {
            let best_device = Self::find_best_device(task, topology, collector);
            if best_device != task.current_device && best_device < super::MAX_HETERO_DEVICES {
                let overhead = Self::estimate_migration_overhead(task, task.current_device, best_device, topology);
                migrations.push(MigrationEntry {
                    task_id: task.task_id,
                    source_device: task.current_device,
                    target_device: best_device,
                    estimated_overhead_us: overhead,
                });
                step += 1;
            }

            if step >= 5 {
                break;
            }
        }

        let quality = if deviation > 0 { (100 - deviation).max(balance_pct) } else { 100 };

        BalanceDecision {
            device_assignments: alloc::vec::Vec::new(),
            migration_plan: migrations,
            convergence_step: step,
            balance_quality: quality,
            confidence: if deviation > 50 { 80 } else { 60 },
        }
    }

    /// Find best device for a task
    fn find_best_device(
        task: &TaskCharacteristics,
        topology: &HeteroDeviceTopology,
        _collector: &LoadCollector,
    ) -> usize {
        let mut best = task.current_device;
        let mut best_score = 0u32;

        for i in 0..super::MAX_HETERO_DEVICES {
            if let Some(dev) = topology.get_device(i) {
                if !dev.is_usable() {
                    continue;
                }
                let score = Self::task_device_match_score(
                    task, dev.device_type, dev.compute_score, dev.memory_bandwidth_mbps,
                );
                let locality = if task.current_device < super::MAX_HETERO_DEVICES {
                    Self::data_locality_score(
                        topology.numa_map[task.current_device],
                        topology.numa_map[i],
                        topology.pcie_bandwidth(task.current_device, i),
                    )
                } else {
                    50
                };
                let combined = (score * 3 + locality * 2) / 5;
                let load_penalty = dev.load;
                let final_score = if combined > load_penalty { combined - load_penalty } else { 0 };

                if final_score > best_score {
                    best_score = final_score;
                    best = i;
                }
            }
        }

        best
    }

    /// Estimate migration overhead in microseconds
    fn estimate_migration_overhead(
        task: &TaskCharacteristics,
        src: usize,
        dst: usize,
        topology: &HeteroDeviceTopology,
    ) -> u32 {
        let base_overhead = 100u32;
        let memory_factor = task.memory_mb / 10;
        let latency = topology.latency(src, dst).min(10000);
        base_overhead + memory_factor + latency / 100
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_device_match_npu_task() {
        let task = TaskCharacteristics::new(1, 80, 50, true, 0);
        let score = BalanceOptimizer::task_device_match_score(
            &task, NvHeteroDeviceType::NpuDavinci, 800, 400_000,
        );
        assert!(score > 50);
    }

    #[test]
    fn test_data_locality_same_numa() {
        let score = BalanceOptimizer::data_locality_score(0, 0, 500_000);
        assert_eq!(score, 100);
    }

    #[test]
    fn test_data_locality_cross_numa() {
        let score = BalanceOptimizer::data_locality_score(0, 1, 100_000);
        assert!(score < 100);
        assert!(score > 20);
    }

    #[test]
    fn test_optimize_no_imbalance() {
        let mut topo = HeteroDeviceTopology::new();
        let mut dev = super::super::topology::HeteroDeviceNode::new(0, NvHeteroDeviceType::CpuCluster, 0);
        dev.load = 50;
        dev.state = super::super::device_types::HeteroDeviceState::Active;
        let _ = topo.register_device(dev);
        let collector = LoadCollector::new();
        let tasks: [TaskCharacteristics; 0] = [];
        let decision = BalanceOptimizer::optimize(&tasks, &topo, &collector, 30, 10);
        assert!(decision.migration_plan.is_empty());
    }
}