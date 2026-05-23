/*
 * Nuva OS - Kernel - NUMA Support
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

use core::ptr::read_unaligned;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

/// NUMA configuration
pub mod numa_config {
    /// Maximum number of NUMA nodes
    pub const MAX_NUMA_NODES: usize = 8;

    /// Maximum CPUs per node
    pub const MAX_CPUS_PER_NODE: usize = 64;

    /// Maximum zones per node
    pub const MAX_ZONES_PER_NODE: usize = 4;

    /// NUMA balancing scan period (ms)
    pub const BALANCE_SCAN_PERIOD_MS: u64 = 1000;

    /// NUMA balancing scan delay (ms)
    pub const BALANCE_SCAN_DELAY_MS: u64 = 100;
}

/// NUMA node ID type
pub type NumaNodeId = u32;

/// CPU ID type
pub type CpuId = u32;

/// NUMA node flags
pub mod numa_flags {
    /// Node is online
    pub const NODE_ONLINE: u32 = 1 << 0;

    /// Node has memory
    pub const NODE_HAS_MEMORY: u32 = 1 << 1;

    /// Node has CPU
    pub const NODE_HAS_CPU: u32 = 1 << 2;

    /// Node is possible
    pub const NODE_POSSIBLE: u32 = 1 << 3;
}

/// NUMA node statistics
pub struct NumaNodeStats {
    /// Total pages in this node
    pub total_pages: AtomicU64,

    /// Free pages in this node
    pub free_pages: AtomicU64,

    /// Pages used by kernel
    pub kernel_pages: AtomicU64,

    /// Pages used by user
    pub user_pages: AtomicU64,

    /// Page migrations in
    pub migrations_in: AtomicU64,

    /// Page migrations out
    pub migrations_out: AtomicU64,
}

impl NumaNodeStats {
    pub const fn new() -> Self {
        NumaNodeStats {
            total_pages: AtomicU64::new(0),
            free_pages: AtomicU64::new(0),
            kernel_pages: AtomicU64::new(0),
            user_pages: AtomicU64::new(0),
            migrations_in: AtomicU64::new(0),
            migrations_out: AtomicU64::new(0),
        }
    }
}

/// NUMA memory zone
pub struct NumaZone {
    /// Zone type
    pub zone_type: ZoneType,

    /// Start physical address
    pub start_paddr: u64,

    /// End physical address
    pub end_paddr: u64,

    /// Total pages
    pub total_pages: u64,

    /// Free pages
    pub free_pages: AtomicU64,
}

/// Zone types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneType {
    /// DMA zone (< 16MB)
    ZoneDma = 0,

    /// DMA32 zone (< 4GB)
    ZoneDma32 = 1,

    /// Normal zone
    ZoneNormal = 2,

    /// High memory zone
    ZoneHighMem = 3,
}

impl NumaZone {
    pub const fn new() -> Self {
        NumaZone {
            zone_type: ZoneType::ZoneNormal,
            start_paddr: 0,
            end_paddr: 0,
            total_pages: 0,
            free_pages: AtomicU64::new(0),
        }
    }
}

/// NUMA node
pub struct NumaNode {
    /// Node ID
    pub node_id: NumaNodeId,

    /// Node flags
    pub flags: AtomicU32,

    /// CPUs in this node
    pub cpus: [AtomicU32; numa_config::MAX_CPUS_PER_NODE],

    /// Number of CPUs
    pub nr_cpus: AtomicU32,

    /// Memory zones
    pub zones: [NumaZone; numa_config::MAX_ZONES_PER_NODE],

    /// Number of zones
    pub nr_zones: u32,

    /// Statistics
    pub stats: NumaNodeStats,

    /// Distance to other nodes
    pub distance: [u32; numa_config::MAX_NUMA_NODES],
}

impl NumaNode {
    pub const fn new() -> Self {
        NumaNode {
            node_id: 0,
            flags: AtomicU32::new(0),
            cpus: [const { AtomicU32::new(0) }; numa_config::MAX_CPUS_PER_NODE],
            nr_cpus: AtomicU32::new(0),
            zones: [const { NumaZone::new() }; numa_config::MAX_ZONES_PER_NODE],
            nr_zones: 0,
            stats: NumaNodeStats::new(),
            distance: [0; numa_config::MAX_NUMA_NODES],
        }
    }

    /// Check if node is online
    #[inline]
    pub fn is_online(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & numa_flags::NODE_ONLINE) != 0
    }

    /// Check if node has memory
    #[inline]
    pub fn has_memory(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & numa_flags::NODE_HAS_MEMORY) != 0
    }

    /// Check if node has CPU
    #[inline]
    pub fn has_cpu(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & numa_flags::NODE_HAS_CPU) != 0
    }

    /// Get distance to another node
    #[inline]
    pub fn distance_to(&self, other: NumaNodeId) -> u32 {
        if (other as usize) < numa_config::MAX_NUMA_NODES {
            self.distance[other as usize]
        } else {
            u32::MAX
        }
    }

    /// Add CPU to this node
    pub fn add_cpu(&self, cpu: CpuId) {
        let idx = self.nr_cpus.fetch_add(1, Ordering::AcqRel);
        if (idx as usize) < numa_config::MAX_CPUS_PER_NODE {
            self.cpus[idx as usize].store(cpu, Ordering::Release);
        }
    }
}

/// NUMA balancing statistics
pub struct NumaBalancingStats {
    /// Pages migrated
    pub pages_migrated: AtomicU64,

    /// Pages placed correctly
    pub pages_placed: AtomicU64,

    /// Scan count
    pub scan_count: AtomicU64,

    /// Scan periods skipped
    pub scan_periods_skipped: AtomicU64,
}

impl NumaBalancingStats {
    pub const fn new() -> Self {
        NumaBalancingStats {
            pages_migrated: AtomicU64::new(0),
            pages_placed: AtomicU64::new(0),
            scan_count: AtomicU64::new(0),
            scan_periods_skipped: AtomicU64::new(0),
        }
    }
}

/// NUMA balancing context
pub struct NumaBalancing {
    /// Balancing enabled
    pub enabled: AtomicBool,

    /// Scan period (ms)
    pub scan_period_ms: AtomicU64,

    /// Scan delay (ms)
    pub scan_delay_ms: AtomicU64,

    /// Statistics
    pub stats: NumaBalancingStats,
}

impl NumaBalancing {
    pub const fn new() -> Self {
        NumaBalancing {
            enabled: AtomicBool::new(true),
            scan_period_ms: AtomicU64::new(numa_config::BALANCE_SCAN_PERIOD_MS),
            scan_delay_ms: AtomicU64::new(numa_config::BALANCE_SCAN_DELAY_MS),
            stats: NumaBalancingStats::new(),
        }
    }
}

/// NUMA topology manager
pub struct NumaTopology {
    /// NUMA nodes
    pub nodes: [NumaNode; numa_config::MAX_NUMA_NODES],

    /// Number of online nodes
    pub nr_online_nodes: AtomicU32,

    /// Total number of nodes
    pub nr_nodes: u32,

    /// NUMA balancing
    pub balancing: NumaBalancing,

    /// Initialized flag
    pub initialized: AtomicBool,
}

impl NumaTopology {
    pub const fn new() -> Self {
        NumaTopology {
            nodes: [const { NumaNode::new() }; numa_config::MAX_NUMA_NODES],
            nr_online_nodes: AtomicU32::new(0),
            nr_nodes: 0,
            balancing: NumaBalancing::new(),
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize NUMA topology
    /// Detects NUMA topology from firmware (ACPI SRAT/Device Tree).
    /// Falls back to single-node UMA if no topology found.
    pub fn init(&mut self) {
        // Try to detect NUMA topology from firmware
        let detected_nodes = self.detect_topology();

        if detected_nodes == 0 {
            // No NUMA topology found, assume UMA (single node)
            self.nodes[0].node_id = 0;
            self.nodes[0].flags.store(
                numa_flags::NODE_ONLINE | numa_flags::NODE_HAS_MEMORY | numa_flags::NODE_HAS_CPU,
                Ordering::Release,
            );
            self.nodes[0].distance[0] = 10;
            self.nr_nodes = 1;
            self.nr_online_nodes.store(1, Ordering::Release);
        } else {
            self.nr_nodes = detected_nodes as u32;
            self.nr_online_nodes
                .store(detected_nodes as u32, Ordering::Release);
        }

        // Initialize default distance matrix if not set by firmware
        self.init_distance_matrix();

        self.initialized.store(true, Ordering::Release);
    }

    /// Detect NUMA topology from firmware
    /// @return Number of nodes detected (0 if none)
    fn detect_topology(&mut self) -> usize {
        #[cfg(target_arch = "x86_64")]
        {
            if let Some(nodes) = self.detect_from_acpi() {
                self.apply_topology(&nodes);
                return nodes.len();
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            if let Some(nodes) = self.detect_from_fdt() {
                self.apply_topology(&nodes);
                return nodes.len();
            }
        }

        0
    }

    /// Initialize default distance matrix
    /// Local = 10, Remote = 20 (if not set by firmware)
    fn init_distance_matrix(&mut self) {
        for i in 0..self.nr_nodes as usize {
            for j in 0..self.nr_nodes as usize {
                if self.nodes[i].distance[j] == 0 {
                    self.nodes[i].distance[j] = if i == j { 10 } else { 20 };
                }
            }
        }
    }

    /// Detect NUMA topology from ACPI SRAT (x86_64)
    #[cfg(target_arch = "x86_64")]
    fn detect_from_acpi(&self) -> Option<alloc::vec::Vec<(u32, u64, u64)>> {
        // Parse ACPI SRAT (System Resource Affinity Table) for NUMA topology.
        // The SRAT contains memory affinity entries that map address ranges
        // to proximity domains (NUMA nodes).
        let mut nodes: alloc::vec::Vec<(u32, u64, u64)> = alloc::vec::Vec::new();

        // Use the ACPI parser to find and parse the SRAT
        let mut acpi = crate::hal::acpi::AcpiParser::new();
        if !acpi.find_rsdp() {
            return None;
        }

        // Find SRAT table
        let srat_entry = acpi.find_table(b"SRAT");
        if srat_entry.is_none() {
            return None;
        }

        let srat_entry = match srat_entry {
            Some(e) => e,
            None => return None,
        };
        let srat_paddr = srat_entry.address;

        // Parse SRAT entries
        // SAFETY: srat_paddr is a valid physical address of the SRAT table
        // returned by the ACPI parser.
        unsafe {
            let srat = srat_paddr as *const u8;
            // Read SRAT length from its header (offset 4, 4 bytes)
            let srat_len = read_unaligned(srat.add(4) as *const u32) as usize;
            let mut offset: usize = 48; // Skip SRAT header

            while offset + 8 <= srat_len {
                let entry_type = read_unaligned(srat.add(offset) as *const u8);
                let entry_len = read_unaligned(srat.add(offset + 1) as *const u8) as usize;

                if entry_type == 1 && entry_len >= 40 {
                    // Memory affinity entry
                    let proximity_domain = read_unaligned(srat.add(offset + 2) as *const u32);
                    let base_addr = read_unaligned(srat.add(offset + 8) as *const u64);
                    let length = read_unaligned(srat.add(offset + 16) as *const u64);
                    let flags = read_unaligned(srat.add(offset + 28) as *const u32);

                    // Check if enabled (bit 0 of flags)
                    if (flags & 1) != 0 && length > 0 {
                        nodes.push((proximity_domain, base_addr, base_addr + length));
                    }
                }

                if entry_len == 0 {
                    break;
                }
                offset += entry_len;
            }
        }

        if nodes.is_empty() {
            None
        } else {
            Some(nodes)
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn detect_from_acpi(&self) -> Option<alloc::vec::Vec<(u32, u64, u64)>> {
        None
    }

    /// Detect NUMA topology from Device Tree (ARM64)
    #[cfg(target_arch = "aarch64")]
    fn detect_from_fdt(&self) -> Option<alloc::vec::Vec<(u32, u64, u64)>> {
        // Parse FDT numa-node-id properties to determine NUMA topology.
        // Each memory node in the device tree has a numa-node-id property
        // that indicates its proximity domain.
        let mut nodes: alloc::vec::Vec<(u32, u64, u64)> = alloc::vec::Vec::new();

        // Get FDT address from platform info
        let platform_info = crate::kernel::platform::detect_platform_info(core::ptr::null());
        let fdt_paddr = platform_info.memory_base;
        if fdt_paddr == 0 {
            return None;
        }

        // Parse FDT memory nodes
        // SAFETY: fdt_paddr is the physical address of the FDT passed
        // by the bootloader, mapped to a virtual address by early init.
        unsafe {
            let fdt = fdt_paddr as *const u8;
            // Check FDT magic number (0xD00DFEED)
            let magic = read_unaligned(fdt as *const u32);
            if magic != 0xD00DFEED {
                return None;
            }

            // Walk FDT structure block looking for memory@* nodes
            // For each memory node, read:
            //   - reg property: base address and size
            //   - numa-node-id property: proximity domain
            let totalsize = read_unaligned(fdt.add(4) as *const u32) as usize;
            let _off_dt_struct = read_unaligned(fdt.add(8) as *const u32) as usize;
            let _off_dt_strings = read_unaligned(fdt.add(12) as *const u32) as usize;

            // Simplified: assume single node if FDT parsing finds no
            // numa-node-id properties. Real implementation would walk
            // the FDT tree properly.
            let _ = totalsize;

            // Attempt to find numa-node-id in memory nodes
            // If found, populate nodes; otherwise return None
        }

        if nodes.is_empty() {
            None
        } else {
            Some(nodes)
        }
    }

    #[cfg(not(target_arch = "aarch64"))]
    fn detect_from_fdt(&self) -> Option<alloc::vec::Vec<(u32, u64, u64)>> {
        None
    }

    /// Apply detected topology to node structures
    fn apply_topology(&mut self, nodes: &[(u32, u64, u64)]) {
        for (idx, &(node_id, start_paddr, end_paddr)) in nodes.iter().enumerate() {
            if idx >= numa_config::MAX_NUMA_NODES {
                break;
            }

            self.nodes[idx].node_id = node_id;
            self.nodes[idx].flags.store(
                numa_flags::NODE_ONLINE | numa_flags::NODE_HAS_MEMORY | numa_flags::NODE_HAS_CPU,
                Ordering::Release,
            );
            self.nodes[idx].zones[0] = NumaZone {
                zone_type: ZoneType::ZoneNormal,
                start_paddr,
                end_paddr,
                total_pages: (end_paddr - start_paddr) / 4096,
                free_pages: AtomicU64::new((end_paddr - start_paddr) / 4096),
            };
            self.nodes[idx].nr_zones = 1;
        }
    }

    /// Allocate pages on a specific NUMA node
    /// @param node: Target NUMA node ID
    /// @param order: Allocation order
    /// @return Pointer to Page, or null on failure
    pub fn alloc_pages_node(&self, node: NumaNodeId, order: usize) -> *mut super::Page {
        if (node as usize) >= self.nr_nodes as usize {
            return core::ptr::null_mut();
        }

        let n = &self.nodes[node as usize];
        if !n.is_online() || !n.has_memory() {
            return core::ptr::null_mut();
        }

        let page = super::alloc_pages(order);
        if !page.is_null() {
            n.stats
                .user_pages
                .fetch_add(1u64 << order, Ordering::Relaxed);
            n.stats
                .free_pages
                .fetch_sub(1u64 << order, Ordering::Relaxed);
        }
        page
    }

    /// Allocate pages on the preferred node for current CPU
    /// Falls back to nearest node if preferred node is full.
    pub fn alloc_pages_preferred(&self, order: usize) -> *mut super::Page {
        let cpu = crate::hal::cpu::smp_processor_id();
        let preferred = self.cpu_to_node(cpu);

        let page = self.alloc_pages_node(preferred, order);
        if !page.is_null() {
            return page;
        }

        // Try nearest node
        let nearest = self.find_nearest_node(preferred);
        if nearest != preferred {
            let page = self.alloc_pages_node(nearest, order);
            if !page.is_null() {
                return page;
            }
        }

        // Try all nodes in distance order
        let mut distances: [(u32, NumaNodeId); numa_config::MAX_NUMA_NODES] =
            [(u32::MAX, 0); numa_config::MAX_NUMA_NODES];
        for i in 0..self.nr_nodes as usize {
            distances[i] = (
                self.nodes[preferred as usize].distance_to(i as u32),
                i as u32,
            );
        }

        distances.sort_unstable_by_key(|&(d, _)| d);

        for i in 0..self.nr_nodes as usize {
            let (_, node_id) = distances[i];
            if node_id == preferred || node_id == nearest {
                continue;
            }
            let page = self.alloc_pages_node(node_id, order);
            if !page.is_null() {
                return page;
            }
        }

        core::ptr::null_mut()
    }

    /// Free pages on a specific NUMA node
    pub fn free_pages_node(&self, node: NumaNodeId, page: *mut super::Page, order: usize) {
        if page.is_null() {
            return;
        }

        super::free_pages(page, order);

        if (node as usize) < self.nr_nodes as usize {
            let n = &self.nodes[node as usize];
            n.stats
                .free_pages
                .fetch_add(1u64 << order, Ordering::Relaxed);
            n.stats
                .user_pages
                .fetch_sub(1u64 << order, Ordering::Relaxed);
        }
    }

    /// Migrate a page from one NUMA node to another
    /// @return Ok(true) if migrated, Ok(false) if already on target, Err on failure
    pub fn migrate_page(
        &self,
        page: *mut super::Page,
        from_node: NumaNodeId,
        to_node: NumaNodeId,
    ) -> Result<bool, NumaMigrateError> {
        if from_node == to_node {
            return Ok(false);
        }

        let src_page = self.alloc_pages_node(to_node, 0);
        if src_page.is_null() {
            return Err(NumaMigrateError::AllocFailed);
        }

        // SAFETY: both pages are valid from allocator
        unsafe {
            let dst_vaddr = (*src_page).phys_addr + 0xFFFF_0000_0000_0000;
            let src_vaddr = (*page).phys_addr + 0xFFFF_0000_0000_0000;
            core::ptr::copy_nonoverlapping(src_vaddr as *const u8, dst_vaddr as *mut u8, 4096);
        }

        // Free old page
        self.free_pages_node(from_node, page, 0);

        // Update statistics
        self.nodes[from_node as usize]
            .stats
            .migrations_out
            .fetch_add(1, Ordering::Relaxed);
        self.nodes[to_node as usize]
            .stats
            .migrations_in
            .fetch_add(1, Ordering::Relaxed);

        Ok(true)
    }

    /// Scan and balance pages across NUMA nodes
    /// Called periodically by the NUMA balancing daemon.
    pub fn balance(&self) {
        if !self.balancing.enabled.load(Ordering::Relaxed) {
            return;
        }

        self.balancing
            .stats
            .scan_count
            .fetch_add(1, Ordering::Relaxed);

        // NUMA balancing: scan task memory access patterns and migrate
        // pages from remote nodes to the local node.
        //
        // Algorithm:
        // 1. Sample the current task's memory accesses via PTE accessed bits
        // 2. Identify "hot" pages that are frequently accessed from a remote node
        // 3. Migrate those pages to the node where the accessing CPU resides
        // 4. Rate-limit migrations to avoid excessive memory bandwidth usage
        let current_cpu = crate::hal::cpu::smp_processor_id();
        let local_node = self.cpu_to_node(current_cpu);

        // Walk pages and check for remote accesses
        let mut pages_scanned: u64 = 0;
        let mut pages_migrated: u64 = 0;
        const MAX_SCAN_PAGES: u64 = 256;
        const MIGRATION_THRESHOLD: u32 = 3; // Minimum access count to trigger migration

        // In a full implementation, we would:
        // - Walk the current task's page table
        // - Check PTE accessed bits for each page
        // - Compare the page's node with the current CPU's node
        // - If a page is on a remote node and accessed frequently,
        //   call migrate_page() to move it to the local node
        //
        // For now, we implement the scan loop framework:
        for node_idx in 0..self.nr_nodes as usize {
            let node = &self.nodes[node_idx];
            if !node.is_online() || !node.has_memory() {
                continue;
            }

            let remote_node = node.node_id;
            if remote_node == local_node {
                continue;
            }

            // Check if this remote node has pages that the local CPU
            // accesses frequently (would be determined by PTE scan)
            let distance = self.nodes[local_node as usize].distance_to(remote_node);
            if distance <= 10 {
                continue; // Local or same distance, no benefit from migration
            }

            // Migrate hot pages from remote to local
            // Rate limit: migrate at most a few pages per scan
            let max_migrate = 4u64;
            if pages_migrated >= max_migrate {
                break;
            }

            if pages_scanned >= MAX_SCAN_PAGES {
                break;
            }

            pages_scanned += 1;
        }

        self.balancing
            .stats
            .pages_migrated
            .fetch_add(pages_migrated, Ordering::Relaxed);
    }

    /// Get node for a given CPU
    pub fn cpu_to_node(&self, cpu: CpuId) -> NumaNodeId {
        for i in 0..self.nr_nodes {
            let node = &self.nodes[i as usize];
            if !node.is_online() {
                continue;
            }

            let nr_cpus = node.nr_cpus.load(Ordering::Acquire);
            for j in 0..nr_cpus {
                if node.cpus[j as usize].load(Ordering::Acquire) == cpu {
                    return node.node_id;
                }
            }
        }

        0 // Default to node 0
    }

    /// Get node for a given physical address
    pub fn paddr_to_node(&self, paddr: u64) -> NumaNodeId {
        for i in 0..self.nr_nodes {
            let node = &self.nodes[i as usize];
            if !node.is_online() || !node.has_memory() {
                continue;
            }

            for j in 0..node.nr_zones {
                let zone = &node.zones[j as usize];
                if paddr >= zone.start_paddr && paddr < zone.end_paddr {
                    return node.node_id;
                }
            }
        }

        0 // Default to node 0
    }

    /// Get preferred node for allocation
    /// @param current_node: Current node of the process
    /// @param flags: Allocation flags
    /// @return Preferred node ID
    pub fn preferred_node(&self, current_node: NumaNodeId, _flags: u32) -> NumaNodeId {
        // Prefer local node
        if (current_node as usize) < self.nr_nodes as usize {
            let node = &self.nodes[current_node as usize];
            if node.is_online() && node.has_memory() {
                return current_node;
            }
        }

        // Fall back to first online node with memory
        for i in 0..self.nr_nodes {
            let node = &self.nodes[i as usize];
            if node.is_online() && node.has_memory() {
                return node.node_id;
            }
        }

        0
    }

    /// Find nearest node with memory
    pub fn find_nearest_node(&self, from: NumaNodeId) -> NumaNodeId {
        let mut min_dist = u32::MAX;
        let mut nearest = from;

        for i in 0..self.nr_nodes {
            let node = &self.nodes[i as usize];
            if !node.is_online() || !node.has_memory() {
                continue;
            }

            let dist = self.nodes[from as usize].distance_to(node.node_id);
            if dist < min_dist {
                min_dist = dist;
                nearest = node.node_id;
            }
        }

        nearest
    }

    /// Get total free pages across all nodes
    pub fn total_free_pages(&self) -> u64 {
        let mut total = 0u64;

        for i in 0..self.nr_nodes {
            let node = &self.nodes[i as usize];
            if node.is_online() {
                total += node.stats.free_pages.load(Ordering::Relaxed);
            }
        }

        total
    }
}

/// Global NUMA topology
static NUMA_TOPOLOGY: core::sync::OnceLock<NumaTopology> = core::sync::OnceLock::new();

/// Get NUMA topology
pub fn numa_topology() -> &'static NumaTopology {
    NUMA_TOPOLOGY.get_or_init(NumaTopology::new)
}

/// Initialize NUMA support
pub fn init_numa() {
    get_numa_topology().init();
}

/// Get node for CPU
pub fn cpu_to_node(cpu: CpuId) -> NumaNodeId {
    get_numa_topology().cpu_to_node(cpu)
}

/// Get node for physical address
pub fn paddr_to_node(paddr: u64) -> NumaNodeId {
    get_numa_topology().paddr_to_node(paddr)
}

/// Check if NUMA is available
pub fn numa_available() -> bool {
    get_numa_topology().initialized.load(Ordering::Acquire) && get_numa_topology().nr_nodes > 1
}

/// NUMA page migration error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumaMigrateError {
    /// Allocation on target node failed
    AllocFailed,
    /// Source page not valid
    InvalidPage,
    /// Page already on target node
    AlreadyLocal,
}

/// Allocate pages on preferred NUMA node
pub fn alloc_pages_preferred(order: usize) -> *mut super::Page {
    get_numa_topology().alloc_pages_preferred(order)
}

/// Allocate pages on specific NUMA node
pub fn alloc_pages_node(node: NumaNodeId, order: usize) -> *mut super::Page {
    get_numa_topology().alloc_pages_node(node, order)
}

/// Free pages on specific NUMA node
pub fn free_pages_node(node: NumaNodeId, page: *mut super::Page, order: usize) {
    get_numa_topology().free_pages_node(node, page, order)
}

/// Run NUMA balancing scan
pub fn numa_balance() {
    get_numa_topology().balance();
}
