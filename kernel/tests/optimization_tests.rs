/*
 * Nuva OS - Kernel - Tests - OptimizationTests
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
 * Nuva OS - Kernel - Optimization Module Tests
 * 
 * Copyright (C) 2026 Nuva OS Team
 * 
 * Unit tests for optimization modules
 */

#![cfg(test)]

mod percpu_cache_tests {
    use crate::mm::percpu_cache::{PerCpuPageCache, PerCpuPageCacheSet, PcpStats};
    use core::sync::atomic::Ordering;

    #[test]
    fn test_pcp_stats_new() {
        let stats = PcpStats::new();
        assert_eq!(stats.alloc_count.load(Ordering::Relaxed), 0);
        assert_eq!(stats.free_count.load(Ordering::Relaxed), 0);
        assert_eq!(stats.hit_count.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_pcp_cache_new() {
        let cache = PerCpuPageCache::new();
        assert_eq!(cache.count.load(Ordering::Relaxed), 0);
    }
}

mod huge_page_tests {
    use crate::mm::huge_page::{HugePageSize, HugePagePool};

    #[test]
    fn test_huge_page_size_values() {
        assert_eq!(HugePageSize::Huge2MB as u32, 21);
        assert_eq!(HugePageSize::Huge1GB as u32, 30);
    }

    #[test]
    fn test_huge_page_pool_new() {
        let pool = HugePagePool::new();
        assert_eq!(pool.nr_huge_pages, 0);
    }
}

mod rbtree_tests {
    use crate::sched::rbtree::{RbTree, RbNode};

    #[test]
    fn test_rb_node_new() {
        let node = RbNode::new(0);
        assert_eq!(node.key, 0);
        assert!(node.is_red());
    }

    #[test]
    fn test_rb_tree_new() {
        let tree = RbTree::new();
        assert_eq!(tree.count, 0);
        assert!(tree.root.is_null());
    }
}

mod sched_domain_tests {
    use crate::sched::sched_domain::{SchedDomain, SchedGroup, CpuMask};

    #[test]
    fn test_cpu_mask_new() {
        let mask = CpuMask::new();
        assert_eq!(mask.bits.load(core::sync::atomic::Ordering::Relaxed), 0);
    }

    #[test]
    fn test_sched_domain_new() {
        let sd = SchedDomain::new();
        assert_eq!(sd.level, 0);
    }
}

mod eas_tests {
    use crate::sched::eas::{EnergyModel, PerfDomain, PerfState};

    #[test]
    fn test_perf_state_new() {
        let state = PerfState::new();
        assert_eq!(state.frequency, 0);
        assert_eq!(state.power, 0);
    }

    #[test]
    fn test_perf_domain_new() {
        let pd = PerfDomain::new();
        assert_eq!(pd.nr_perf_states, 0);
    }
}

mod page_cache_tests {
    use crate::fs::page_cache::{PageCache, PageCacheKey, PageCacheEntry};

    #[test]
    fn test_page_cache_key_new() {
        let key = PageCacheKey::new(1, 0);
        assert_eq!(key.ino, 1);
        assert_eq!(key.index, 0);
    }

    #[test]
    fn test_page_cache_new() {
        let cache = PageCache::new();
        assert_eq!(cache.nr_pages.load(core::sync::atomic::Ordering::Relaxed), 0);
    }
}

mod dcache_tests {
    use crate::fs::dcache::{DentryCache, Dentry, Qstr};

    #[test]
    fn test_qstr_new() {
        let qstr = Qstr::new();
        assert_eq!(qstr.hash, 0);
        assert_eq!(qstr.len, 0);
    }

    #[test]
    fn test_dentry_new() {
        let dentry = Dentry::new();
        assert!(!dentry.is_valid());
    }
}

mod tcp_fastpath_tests {
    use crate::net::tcp_fastpath::{TcpConnection, TcpState, TcpFastPathProcessor};

    #[test]
    fn test_tcp_connection_new() {
        let conn = TcpConnection::new();
        let state = conn.state.load(core::sync::atomic::Ordering::Relaxed);
        assert_eq!(state, TcpState::Closed as u32);
    }

    #[test]
    fn test_tcp_fast_path_processor_new() {
        let processor = TcpFastPathProcessor::new();
        assert!(processor.enabled.load(core::sync::atomic::Ordering::Relaxed));
    }
}

// Binder tests removed - using NuvaIPC instead
// NuvaIPC provides better performance and security

mod aslr_tests {
    use crate::security::aslr::{AslrState, aslr_config};

    #[test]
    fn test_aslr_state_new() {
        let state = AslrState::new();
        assert!(state.is_enabled());
    }

    #[test]
    fn test_aslr_config_values() {
        assert_eq!(aslr_config::STACK_RND_BITS, 18);
        assert_eq!(aslr_config::MMAP_RND_BITS, 28);
        assert_eq!(aslr_config::BRK_RND_BITS, 23);
    }
}

mod stack_canary_tests {
    use crate::security::stack_canary::{StackCanary, TaskStackCanary};

    #[test]
    fn test_stack_canary_new() {
        let canary = StackCanary::new();
        assert!(!canary.valid.load(core::sync::atomic::Ordering::Relaxed));
    }

    #[test]
    fn test_task_stack_canary_new() {
        let task_canary = TaskStackCanary::new();
        assert_eq!(task_canary.task_id, 0);
    }
}

mod numa_tests {
    use crate::mm::numa::{NumaNode, NumaTopology, NumaNodeStats};

    #[test]
    fn test_numa_node_stats_new() {
        let stats = NumaNodeStats::new();
        assert_eq!(stats.total_pages.load(core::sync::atomic::Ordering::Relaxed), 0);
    }

    #[test]
    fn test_numa_node_new() {
        let node = NumaNode::new();
        assert_eq!(node.node_id, 0);
    }
}

mod compaction_tests {
    use crate::mm::compaction::{MemoryCompactor, CompactResult, CompactStats};

    #[test]
    fn test_compact_stats_new() {
        let stats = CompactStats::new();
        assert_eq!(stats.total_compactions.load(core::sync::atomic::Ordering::Relaxed), 0);
    }

    #[test]
    fn test_compact_result_values() {
        assert_eq!(CompactResult::Success as u32, 0);
        assert_eq!(CompactResult::Partial as u32, 1);
        assert_eq!(CompactResult::NoSuitablePages as u32, 2);
    }
}

mod io_uring_tests {
    use crate::fs::io_uring::{IoUring, IoSqe, IoCqe, IoOpCode};

    #[test]
    fn test_io_sqe_new() {
        let sqe = IoSqe::new();
        assert_eq!(sqe.opcode, 0);
        assert_eq!(sqe.fd, 0);
    }

    #[test]
    fn test_io_cqe_new() {
        let cqe = IoCqe::new();
        assert_eq!(cqe.user_data, 0);
        assert_eq!(cqe.res, 0);
    }

    #[test]
    fn test_io_opcode_values() {
        assert_eq!(IoOpCode::Nop as u8, 0);
        assert_eq!(IoOpCode::Read as u8, 1);
        assert_eq!(IoOpCode::Write as u8, 2);
    }
}
