/*
 * Nuva OS - Kernel - Driver - Icc
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
 * Nuva OS - Kernel - Interconnect Framework
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Interconnect framework for bus bandwidth management.
 */

use crate::{pr_debug, pr_info};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Interconnect ID
pub type IccId = u32;

/// Bandwidth (bytes per second)
pub type Bandwidth = u64;

/// Interconnect Node
#[repr(C)]
pub struct IccNode {
    /// Node ID
    pub id: IccId,
    /// Node name
    pub name: [u8; 32],
    /// Number of links
    pub num_links: u16,
    /// Links to other nodes
    pub links: [IccId; 16],
    /// Current average bandwidth
    pub avg_bw: Bandwidth,
    /// Current peak bandwidth
    pub peak_bw: Bandwidth,
    /// Minimum average bandwidth
    pub min_avg_bw: Bandwidth,
    /// Minimum peak bandwidth
    pub min_peak_bw: Bandwidth,
    /// Maximum average bandwidth
    pub max_avg_bw: Bandwidth,
    /// Maximum peak bandwidth
    pub max_peak_bw: Bandwidth,
    /// Flags
    pub flags: IccNodeFlags,
}

/// Interconnect Node Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct IccNodeFlags: u32 {
        /// Upstream
        const UPSTREAM = 1 << 0;
        /// Downstream
        const DOWNSTREAM = 1 << 1;
        /// Active
        const ACTIVE = 1 << 2;
        /// Sleep
        const SLEEP = 1 << 3;
    }
}

/// Interconnect Path
#[repr(C)]
pub struct IccPath {
    /// Path ID
    pub id: u32,
    /// Source node
    pub src: IccId,
    /// Destination node
    pub dst: IccId,
    /// Nodes in path
    pub nodes: [IccId; 8],
    /// Number of nodes
    pub num_nodes: u8,
    /// Current average bandwidth
    pub avg_bw: Bandwidth,
    /// Current peak bandwidth
    pub peak_bw: Bandwidth,
    /// Reference count
    pub ref_count: AtomicU32,
}

/// Interconnect Request
#[repr(C)]
pub struct IccRequest {
    /// Path ID
    pub path_id: u32,
    /// Average bandwidth (Bps)
    pub avg_bw: Bandwidth,
    /// Peak bandwidth (Bps)
    pub peak_bw: Bandwidth,
    /// Tag
    pub tag: u32,
}

/// Interconnect Provider
pub struct IccProvider {
    /// Provider name
    pub name: [u8; 32],
    /// Provider ID
    pub id: u32,
    /// Number of nodes
    pub num_nodes: u32,
    /// Nodes
    pub nodes: *mut IccNode,
    /// Operations
    pub ops: IccProviderOps,
    /// Private data
    pub data: *mut core::ffi::c_void,
    /// Parent device
    pub parent: u32,
    /// Flags
    pub flags: IccProviderFlags,
}

/// Interconnect Provider Operations
pub struct IccProviderOps {
    /// Set bandwidth
    pub set: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *const IccNode) -> i32>,
    /// Aggregate
    pub aggregate:
        Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut IccNode, u32, u32, u32) -> i32>,
    /// Get bandwidth
    pub get_bw: Option<
        unsafe extern "C" fn(*const core::ffi::c_void, *mut IccNode, *mut u64, *mut u64) -> i32,
    >,
    /// Pre aggregate
    pub pre_aggregate: Option<unsafe extern "C" fn(*mut core::ffi::c_void, *mut IccNode) -> i32>,
}

/// Interconnect Provider Flags
bitflags::bitflags! {
    #[repr(transparent)]
    pub struct IccProviderFlags: u32 {
        /// Int aggregated
        const INT_AGGREGATE = 1 << 0;
        /// Xlate extended
        const XLATE_EXTENDED = 1 << 1;
    }
}

/// Interconnect Manager
pub struct IccManager {
    /// Provider count
    provider_count: AtomicU32,
    /// Path count
    path_count: AtomicU32,
    /// Statistics
    stats: IccStats,
}

/// Interconnect Statistics
pub struct IccStats {
    /// Set count
    pub set_count: AtomicU64,
    /// Get count
    pub get_count: AtomicU64,
    /// Paths created
    pub paths: AtomicU64,
}

impl IccStats {
    pub const fn new() -> Self {
        IccStats {
            set_count: AtomicU64::new(0),
            get_count: AtomicU64::new(0),
            paths: AtomicU64::new(0),
        }
    }
}

impl IccManager {
    pub const fn new() -> Self {
        IccManager {
            provider_count: AtomicU32::new(0),
            path_count: AtomicU32::new(0),
            stats: IccStats::new(),
        }
    }

    /// Initialize
    pub fn init(&self) {
        log_info!("Interconnect manager initialized");
    }

    /// Register provider
    pub fn register_provider(&mut self, _provider: &IccProvider) -> u32 {
        self.provider_count.fetch_add(1, Ordering::AcqRel)
    }

    /// Unregister provider
    pub fn unregister_provider(&mut self, provider_id: u32) {
        log_debug!("icc_unregister_provider: id={}", provider_id);
    }

    /// Get path
    pub fn get_path(&mut self, src: IccId, dst: IccId) -> u32 {
        self.stats.paths.fetch_add(1, Ordering::AcqRel);
        let path_id = self.path_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("icc_get_path: src={}, dst={}, path={}", src, dst, path_id);
        path_id
    }

    /// Put path
    pub fn put_path(&mut self, path_id: u32) {
        log_debug!("icc_put_path: path={}", path_id);
    }

    /// Set bandwidth
    pub fn set_bw(&mut self, path_id: u32, avg_bw: Bandwidth, peak_bw: Bandwidth) -> i32 {
        self.stats.set_count.fetch_add(1, Ordering::AcqRel);
        log_debug!(
            "icc_set_bw: path={}, avg={}, peak={}",
            path_id,
            avg_bw,
            peak_bw
        );
        0
    }

    /// Get bandwidth
    pub fn get_bw(&mut self, path_id: u32) -> (Bandwidth, Bandwidth) {
        self.stats.get_count.fetch_add(1, Ordering::AcqRel);
        log_debug!("icc_get_bw: path={}", path_id);
        (0, 0)
    }

    /// Set tag
    pub fn set_tag(&mut self, path_id: u32, tag: u32) -> i32 {
        log_debug!("icc_set_tag: path={}, tag={}", path_id, tag);
        0
    }
}

/// Global interconnect manager
static ICC_MANAGER: crate::sync_oncelock::OnceLock<IccManager> = crate::sync_oncelock::OnceLock::new();

/// Get interconnect manager
pub fn icc_manager() -> &'static IccManager {
    ICC_MANAGER.get_or_init(IccManager::new)
}

/// Initialize interconnect manager
pub fn init_icc_manager() {
    let mgr = icc_manager();
    mgr.init();
}

// Convenience functions

/// Get interconnect path
pub fn icc_get(src: IccId, dst: IccId) -> u32 {
    icc_manager().get_path(src, dst)
}

/// Put interconnect path
pub fn icc_put(path_id: u32) {
    icc_manager().put_path(path_id);
}

/// Set bandwidth
pub fn icc_set_bw(path_id: u32, avg_bw: Bandwidth, peak_bw: Bandwidth) -> i32 {
    icc_manager().set_bw(path_id, avg_bw, peak_bw)
}

/// Get bandwidth
pub fn icc_get_bw(path_id: u32) -> (Bandwidth, Bandwidth) {
    icc_manager().get_bw(path_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icc_node_flags() {
        let flags = IccNodeFlags::UPSTREAM | IccNodeFlags::ACTIVE;
        assert!(flags.contains(IccNodeFlags::UPSTREAM));
        assert!(flags.contains(IccNodeFlags::ACTIVE));
    }

    #[test]
    fn test_icc_provider_flags() {
        let flags = IccProviderFlags::INT_AGGREGATE;
        assert!(flags.contains(IccProviderFlags::INT_AGGREGATE));
    }
}
