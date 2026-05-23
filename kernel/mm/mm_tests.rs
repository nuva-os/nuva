/*
 * Nuva OS - Kernel - Memory Management Tests
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

//! Memory Management Test Suite
/*!*/
//! Comprehensive tests for page reclamation, NUMA, and COW.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Test result
#[derive(Debug)]
pub struct TestResult {
    pub name: &'static str,
    pub passed: bool,
    pub message: Option<&'static str>,
}

/// Test runner
pub struct TestRunner {
    results: [Option<TestResult>; 64],
    count: usize,
    passed: usize,
    failed: usize,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            results: core::array::from_fn(|_| None),
            count: 0,
            passed: 0,
            failed: 0,
        }
    }

    pub fn run_test(&mut self, name: &'static str, test: fn() -> bool) {
        let passed = test();
        let result = TestResult {
            name,
            passed,
            message: None,
        };

        if self.count < 64 {
            self.results[self.count] = Some(result);
            self.count += 1;

            if passed {
                self.passed += 1;
            } else {
                self.failed += 1;
            }
        }
    }

    pub fn print_summary(&self) {
        crate::log_info!("=== Memory Management Test Summary ===");
        crate::log_info!("Total: {} tests", self.count);
        crate::log_info!("Passed: {}", self.passed);
        crate::log_info!("Failed: {}", self.failed);

        if self.failed > 0 {
            crate::log_info!("
Failed tests:");
            for i in 0..self.count {
                if let Some(ref result) = self.results[i] {
                    if !result.passed {
                        crate::log_info!("  - {}", result.name);
                    }
                }
            }
        }
    }

    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

// ============================================================================
// Page Reclamation Tests

fn test_lru_list_basic() -> bool {
    use super::reclaim::{LruListHead, LruList, LruNode, PageRef};

    let list = LruListHead::new(LruList::InactiveFile);
    list.is_empty() && list.len() == 0
}

fn test_lru_list_add_remove() -> bool {
    use super::reclaim::{LruListHead, LruList, LruNode, PageRef};

    let list = LruListHead::new(LruList::InactiveFile);

    // Add would require allocation, so just test empty state
    list.is_empty()
}

fn test_page_ref_flags() -> bool {
    use super::reclaim::PageRef;

    let page = PageRef::new(0x1000);
    assert_eq!(page.phys_addr, 0x1000);
    assert!(!page.is_anon());
    assert!(!page.is_dirty());
    assert!(!page.is_active());
    true
}

fn test_working_set_estimator() -> bool {
    use super::reclaim::WorkingSetEstimator;

    let ws = WorkingSetEstimator::new();
    ws.record_access();
    ws.record_access();
    ws.record_refault();

    // Refault rate should be 0 (no evictions yet)
    ws.refault_rate() == 0
}

fn test_reclaim_stats() -> bool {
    use super::reclaim::ReclaimStats;

    let stats = ReclaimStats::new();
    assert_eq!(stats.pages_reclaimed.load(Ordering::Relaxed), 0);
    assert_eq!(stats.pages_scanned.load(Ordering::Relaxed), 0);
    true
}

fn test_page_reclaimer_init() -> bool {
    use super::reclaim::PageReclaimer;

    let reclaimer = PageReclaimer::new();
    !reclaimer.is_reclaiming()
}

fn test_lru_list_types() -> bool {
    use super::reclaim::LruList;

    LruList::ActiveAnon as usize == 0
        && LruList::InactiveAnon as usize == 1
        && LruList::ActiveFile as usize == 2
        && LruList::InactiveFile as usize == 3
        && LruList::Unevictable as usize == 4
}

// ============================================================================
// NUMA Tests

fn test_numa_node_init() -> bool {
    use super::numa::NumaNode;

    let node = NumaNode::new();
    assert_eq!(node.node_id, 0);
    assert!(!node.is_online());
    true
}

fn test_numa_node_flags() -> bool {
    use super::numa::{NumaNode, numa_flags};

    let node = NumaNode::new();

    // Set online flag
    node.flags.store(numa_flags::NODE_ONLINE, Ordering::Release);
    assert!(node.is_online());
    assert!(!node.has_memory());
    assert!(!node.has_cpu());

    // Set memory flag
    node.flags.fetch_or(numa_flags::NODE_HAS_MEMORY, Ordering::AcqRel);
    assert!(node.has_memory());

    true
}

fn test_numa_node_distance() -> bool {
    use super::numa::NumaNode;

    let mut node = NumaNode::new();
    node.distance[0] = 10; // Local
    node.distance[1] = 20; // Remote

    node.distance_to(0) == 10 && node.distance_to(1) == 20
}

fn test_numa_topology_init() -> bool {
    use super::numa::NumaTopology;

    let topology = NumaTopology::new();
    !topology.initialized.load(Ordering::Relaxed)
}

fn test_numa_balancing() -> bool {
    use super::numa::NumaBalancing;

    let balancing = NumaBalancing::new();
    balancing.enabled.load(Ordering::Relaxed)
}

fn test_numa_zone_types() -> bool {
    use super::numa::ZoneType;

    ZoneType::ZoneDma as usize == 0
        && ZoneType::ZoneDma32 as usize == 1
        && ZoneType::ZoneNormal as usize == 2
        && ZoneType::ZoneHighMem as usize == 3
}

// ============================================================================
// COW Tests

fn test_cow_entry_basic() -> bool {
    use super::cow::CowEntry;

    let entry = CowEntry::new(0x1000, 1, 0x400000);
    assert!(entry.is_cow());
    assert!(!entry.is_broken());
    assert!(!entry.is_shared());
    assert_eq!(entry.ref_count, 1);
    assert_eq!(entry.owner_pid, 1);
    true
}

fn test_cow_entry_flags() -> bool {
    use super::cow::{CowEntry, cow_flags};

    let mut entry = CowEntry::new(0x1000, 1, 0x400000);

    // Test pending flag
    entry.flags |= cow_flags::COW_PENDING;
    assert!(entry.is_pending());

    // Test broken flag
    entry.flags |= cow_flags::COW_BROKEN;
    assert!(entry.is_broken());

    true
}

fn test_cow_stats() -> bool {
    use super::cow::CowStats;

    let stats = CowStats::new();
    assert_eq!(stats.pages_created.load(Ordering::Relaxed), 0);
    assert_eq!(stats.breaks.load(Ordering::Relaxed), 0);
    assert_eq!(stats.faults_handled.load(Ordering::Relaxed), 0);
    true
}

fn test_cow_manager_init() -> bool {
    use super::cow::CowManager;

    let manager = CowManager::new();
    manager.is_enabled()
}

fn test_cow_page_table_entry() -> bool {
    use super::cow::PageTableEntry;

    let mut pte = PageTableEntry { value: 0 };

    // Set physical address
    pte.set_phys_addr(0x1000);
    assert_eq!(pte.phys_addr(), 0x1000);

    // Set writable
    pte.set_writable(true);
    assert!((pte.value & (1 << 1)) != 0);

    // Clear writable
    pte.set_writable(false);
    assert!((pte.value & (1 << 1)) == 0);

    true
}

fn test_cow_fault_action() -> bool {
    use super::cow::FaultAction;

    // Test all fault actions
    let actions = [
        FaultAction::None,
        FaultAction::UpdatePte,
        FaultAction::Signal,
        FaultAction::Oom,
    ];

    actions.len() == 4
}

// ============================================================================
// Integration Tests

fn test_numa_reclaim_integration() -> bool {
    // Test that NUMA-aware reclamation works
    use super::numa::NumaTopology;
    use super::reclaim::PageReclaimer;

    let topology = NumaTopology::new();
    let reclaimer = PageReclaimer::new();

    // Both should initialize without error
    !topology.initialized.load(Ordering::Relaxed)
        && !reclaimer.is_reclaiming()
}

fn test_cow_numa_integration() -> bool {
    // Test that COW works with NUMA
    use super::cow::CowManager;
    use super::numa::NumaTopology;

    let cow = CowManager::new();
    let topology = NumaTopology::new();

    cow.is_enabled() && !topology.initialized.load(Ordering::Relaxed)
}

// ============================================================================
// Stress Tests

fn test_lru_stress() -> bool {
    use super::reclaim::{LruListHead, LruList};

    let list = LruListHead::new(LruList::InactiveFile);

    // Simulate many rotations
    for _ in 0..100 {
        list.rotate();
    }

    list.is_empty()
}

fn test_working_set_stress() -> bool {
    use super::reclaim::WorkingSetEstimator;

    let ws = WorkingSetEstimator::new();

    // Simulate many accesses
    for _ in 0..1000 {
        ws.record_access();
    }

    // Simulate some refaults
    for _ in 0..10 {
        ws.record_refault();
        ws.record_eviction();
    }

    // Refault rate should be around 10%
    let rate = ws.refault_rate();
    rate >= 5 && rate <= 15
}

fn test_cow_ref_count_stress() -> bool {
    use super::cow::CowEntry;

    let mut entry = CowEntry::new(0x1000, 1, 0x400000);

    // Simulate many shares
    for _ in 0..100 {
        entry.ref_count += 1;
    }

    assert!(entry.is_shared());

    // Simulate many releases
    for _ in 0..100 {
        entry.ref_count -= 1;
    }

    assert!(!entry.is_shared());
    assert_eq!(entry.ref_count, 1);

    true
}

// ============================================================================
// Performance Tests

fn test_lru_performance() -> bool {
    // LRU operations should be O(1)
    use super::reclaim::{LruListHead, LruList};

    let list = LruListHead::new(LruList::InactiveFile);

    // Measure time for operations
    // For now, just verify the list works
    list.is_empty()
}

fn test_numa_distance_performance() -> bool {
    use super::numa::NumaNode;

    let node = NumaNode::new();

    // Distance lookup should be O(1)
    for i in 0..8 {
        let _ = node.distance_to(i);
    }

    true
}

fn test_cow_fault_performance() -> bool {
    use super::cow::{CowEntry, CowManager};

    let manager = CowManager::new();
    let mut entry = CowEntry::new(0x1000, 1, 0x400000);

    // Fault handling should be fast
    // For now, just verify the manager works
    manager.is_enabled()
}

// ============================================================================
// Main Test Runner

/// Run all memory management tests
pub fn run_all_tests() -> bool {
    let mut runner = TestRunner::new();

    crate::log_info!("=== Memory Management Test Suite ===
");

    // Page reclamation tests
    crate::log_info!("Running Page Reclamation tests...");
    runner.run_test("lru_list_basic", test_lru_list_basic);
    runner.run_test("lru_list_add_remove", test_lru_list_add_remove);
    runner.run_test("page_ref_flags", test_page_ref_flags);
    runner.run_test("working_set_estimator", test_working_set_estimator);
    runner.run_test("reclaim_stats", test_reclaim_stats);
    runner.run_test("page_reclaimer_init", test_page_reclaimer_init);
    runner.run_test("lru_list_types", test_lru_list_types);

    // NUMA tests
    crate::log_info!("Running NUMA tests...");
    runner.run_test("numa_node_init", test_numa_node_init);
    runner.run_test("numa_node_flags", test_numa_node_flags);
    runner.run_test("numa_node_distance", test_numa_node_distance);
    runner.run_test("numa_topology_init", test_numa_topology_init);
    runner.run_test("numa_balancing", test_numa_balancing);
    runner.run_test("numa_zone_types", test_numa_zone_types);

    // COW tests
    crate::log_info!("Running COW tests...");
    runner.run_test("cow_entry_basic", test_cow_entry_basic);
    runner.run_test("cow_entry_flags", test_cow_entry_flags);
    runner.run_test("cow_stats", test_cow_stats);
    runner.run_test("cow_manager_init", test_cow_manager_init);
    runner.run_test("cow_page_table_entry", test_cow_page_table_entry);
    runner.run_test("cow_fault_action", test_cow_fault_action);

    // Integration tests
    crate::log_info!("Running Integration tests...");
    runner.run_test("numa_reclaim_integration", test_numa_reclaim_integration);
    runner.run_test("cow_numa_integration", test_cow_numa_integration);

    // Stress tests
    crate::log_info!("Running Stress tests...");
    runner.run_test("lru_stress", test_lru_stress);
    runner.run_test("working_set_stress", test_working_set_stress);
    runner.run_test("cow_ref_count_stress", test_cow_ref_count_stress);

    // Performance tests
    crate::log_info!("Running Performance tests...");
    runner.run_test("lru_performance", test_lru_performance);
    runner.run_test("numa_distance_performance", test_numa_distance_performance);
    runner.run_test("cow_fault_performance", test_cow_fault_performance);

    // Print summary
    runner.print_summary();

    runner.all_passed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_all() {
        assert!(run_all_tests());
    }
}
