/*
 * Nuva OS - Kernel - Scheduler Domain Support
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

#![no_std]
#![allow(dead_code)]

use core::sync::atomic::{AtomicU32, AtomicU64, AtomicBool, Ordering};

use crate::syslib::posix::errno::Errno;
use crate::kernel::error::Errno;
/// Maximum number of CPUs
pub const MAX_NR_CPUS: usize = 16;

/// Maximum number of scheduling domains
pub const MAX_NR_DOMAINS: usize = 4;

// CPU mask
pub struct CpuMask {
    pub bits: AtomicU64,
}

impl Clone for CpuMask {
    fn clone(&self) -> Self {
        CpuMask { bits: AtomicU64::new(self.bits.load(Ordering::Relaxed)) }
    }
}


impl CpuMask {
    pub const fn new() -> Self {
        CpuMask {
            bits: AtomicU64::new(0),
        }
    }
    
    pub fn set_cpu(&self, cpu: usize) {
        if cpu < 64 {
            self.bits.fetch_or(1 << cpu, Ordering::AcqRel);
        }
    }
    
    pub fn clear_cpu(&self, cpu: usize) {
        if cpu < 64 {
            self.bits.fetch_and(!(1 << cpu), Ordering::AcqRel);
        }
    }
    
    pub fn test_cpu(&self, cpu: usize) -> bool {
        if cpu < 64 {
            (self.bits.load(Ordering::Acquire) & (1 << cpu)) != 0
        } else {
            false
        }
    }
    
    pub fn weight(&self) -> u32 {
        let bits = self.bits.load(Ordering::Acquire);
        bits.count_ones()
    }
    
    pub fn first_cpu(&self) -> i32 {
        let bits = self.bits.load(Ordering::Acquire);
        if bits == 0 {
            return Errno::Eperm.to_ret_i32();
        }
        bits.trailing_zeros() as i32
    }
    
    pub fn next_cpu(&self, cpu: usize) -> i32 {
        let bits = self.bits.load(Ordering::Acquire);
        if cpu >= 63 {
            return Errno::Eperm.to_ret_i32();
        }
        let mask = bits & (!((1 << (cpu + 1)) - 1));
        if mask == 0 {
            return Errno::Eperm.to_ret_i32();
        }
        mask.trailing_zeros() as i32
    }
}

/// Scheduling group
pub struct SchedGroup {
    /// CPUs in this group
    pub cpus: CpuMask,
    
    /// Group capacity
    pub capacity: AtomicU32,
    
    /// Group load
    pub load: AtomicU32,
    
    /// Next group in domain
    pub next: *mut SchedGroup,
}

impl SchedGroup {
    pub const fn new() -> Self {
        SchedGroup {
            cpus: CpuMask::new(),
            capacity: AtomicU32::new(1024),  // Default capacity
            load: AtomicU32::new(0),
            next: core::ptr::null_mut(),
        }
    }
}

/// Scheduling domain flags
pub mod sd_flags {
    /// Load balancing enabled
    pub const SD_LOAD_BALANCE: u32 = 1 << 0;
    
    /// Balance on fork
    pub const SD_BALANCE_FORK: u32 = 1 << 1;
    
    /// Balance on exec
    pub const SD_BALANCE_EXEC: u32 = 1 << 2;
    
    /// Balance on wake
    pub const SD_BALANCE_WAKE: u32 = 1 << 3;
    
    /// Wake affine
    pub const SD_WAKE_AFFINE: u32 = 1 << 4;
    
    /// Prefer sibling
    pub const SD_PREFER_SIBLING: u32 = 1 << 5;
    
    /// Share CPU capacity
    pub const SD_SHARE_CPUCAPACITY: u32 = 1 << 6;
    
    /// Share power domain
    pub const SD_SHARE_POWERDOMAIN: u32 = 1 << 7;
    
    /// Overlap allowed
    pub const SD_OVERLAP: u32 = 1 << 8;
}

/// Scheduling domain
/// Represents a hierarchical level of CPUs for load balancing.
/// Examples: SMT (hyperthreading) -> Core -> Socket -> NUMA node
pub struct SchedDomain {
    /// Domain level (0 = lowest, e.g., SMT)
    pub level: u32,
    
    /// CPUs in this domain
    pub span: CpuMask,
    
    /// Domain name
    pub name: [u8; 16],
    
    /// Domain flags
    pub flags: AtomicU32,
    
    /// Parent domain (higher level)
    pub parent: *mut SchedDomain,
    
    /// Child domain (lower level)
    pub child: *mut SchedDomain,
    
    /// Scheduling groups in this domain
    pub groups: *mut SchedGroup,
    
    /// Number of groups
    pub nr_groups: u32,
    
    /// Load imbalance percentage threshold
    pub imbalance_pct: u32,
    
    /// Cache hot time (nanoseconds)
    pub cache_hot_time: AtomicU64,
    
    /// Maximum interval between balances (milliseconds)
    pub max_interval: u32,
    
    /// Minimum interval between balances (milliseconds)
    pub min_interval: u32,
    
    /// Last balance time
    pub last_balance: AtomicU64,
    
    /// Balance interval
    pub balance_interval: AtomicU32,
    
    /// Number of balance attempts
    pub nr_balance_failed: AtomicU32,
    
    /// Domain statistics
    pub stats: SchedDomainStats,
}

impl Clone for SchedDomain {
    fn clone(&self) -> Self {
        Self {
            level: self.level.clone(),
            span: self.span.clone(),
            name: self.name.clone(),
            flags: AtomicU32::new(self.flags.load(core::sync::atomic::Ordering::Relaxed)),
            parent: self.parent.clone(),
            child: self.child.clone(),
            groups: self.groups.clone(),
            nr_groups: self.nr_groups.clone(),
            imbalance_pct: self.imbalance_pct.clone(),
            cache_hot_time: AtomicU64::new(self.cache_hot_time.load(core::sync::atomic::Ordering::Relaxed)),
            max_interval: self.max_interval.clone(),
            min_interval: self.min_interval.clone(),
            last_balance: AtomicU64::new(self.last_balance.load(core::sync::atomic::Ordering::Relaxed)),
            balance_interval: AtomicU32::new(self.balance_interval.load(core::sync::atomic::Ordering::Relaxed)),
            nr_balance_failed: AtomicU32::new(self.nr_balance_failed.load(core::sync::atomic::Ordering::Relaxed)),
            stats: self.stats.clone(),
        }
    }
}

// Scheduling domain statistics
pub struct SchedDomainStats {
    /// Number of load balances
    pub lb_count: AtomicU64,
    
    /// Number of successful balances
    pub lb_balanced: AtomicU64,
    
    /// Number of failed balances
    pub lb_failed: AtomicU64,
    
    /// Number of tasks moved
    pub lb_moved: AtomicU64,
    
    /// Number of tasks pushed
    pub lb_pushed: AtomicU64,
    
    /// Number of tasks pulled
    pub lb_pulled: AtomicU64,
}

impl Clone for SchedDomainStats {
    fn clone(&self) -> Self {
        SchedDomainStats {
            lb_count: AtomicU64::new(self.lb_count.load(Ordering::Relaxed)),
            lb_balanced: AtomicU64::new(self.lb_balanced.load(Ordering::Relaxed)),
            lb_failed: AtomicU64::new(self.lb_failed.load(Ordering::Relaxed)),
            lb_moved: AtomicU64::new(self.lb_moved.load(Ordering::Relaxed)),
            lb_pushed: AtomicU64::new(self.lb_pushed.load(Ordering::Relaxed)),
            lb_pulled: AtomicU64::new(self.lb_pulled.load(Ordering::Relaxed)),
        }
    }
}


impl SchedDomain {
    pub const fn new() -> Self {
        SchedDomain {
            level: 0,
            span: CpuMask::new(),
            name: [0; 16],
            flags: AtomicU32::new(sd_flags::SD_LOAD_BALANCE),
            parent: core::ptr::null_mut(),
            child: core::ptr::null_mut(),
            groups: core::ptr::null_mut(),
            nr_groups: 0,
            imbalance_pct: 125,  // 25% imbalance threshold
            cache_hot_time: AtomicU64::new(500_000),  // 500us
            max_interval: 32,
            min_interval: 1,
            last_balance: AtomicU64::new(0),
            balance_interval: AtomicU32::new(1),
            nr_balance_failed: AtomicU32::new(0),
            stats: SchedDomainStats {
                lb_count: AtomicU64::new(0),
                lb_balanced: AtomicU64::new(0),
                lb_failed: AtomicU64::new(0),
                lb_moved: AtomicU64::new(0),
                lb_pushed: AtomicU64::new(0),
                lb_pulled: AtomicU64::new(0),
            },
        }
    }
    
    /// Check if load balancing is enabled
    pub fn is_load_balance_enabled(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & sd_flags::SD_LOAD_BALANCE) != 0
    }
    
    /// Check if should balance on fork
    pub fn should_balance_fork(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & sd_flags::SD_BALANCE_FORK) != 0
    }
    
    /// Check if should balance on exec
    pub fn should_balance_exec(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & sd_flags::SD_BALANCE_EXEC) != 0
    }
    
    /// Check if should balance on wake
    pub fn should_balance_wake(&self) -> bool {
        (self.flags.load(Ordering::Acquire) & sd_flags::SD_BALANCE_WAKE) != 0
    }
    
    /// Get the busiest group in this domain
    pub fn find_busiest_group(&self) -> *mut SchedGroup {
        if self.groups.is_null() {
            return core::ptr::null_mut();
        }
        
        let mut busiest = self.groups;
        let mut max_load = 0u32;
        let mut group = self.groups;
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            loop {
                let load = (*group).load.load(Ordering::Acquire);
                if load > max_load {
                    max_load = load;
                    busiest = group;
                }
                
                if (*group).next.is_null() {
                    break;
                }
                group = (*group).next;
            }
        }
        
        busiest
    }
    
    /// Calculate domain load
    pub fn calculate_load(&self) -> u32 {
        if self.groups.is_null() {
            return 0;
        }
        
        let mut total_load = 0u32;
        let mut group = self.groups;
        
        // SAFETY: unsafe block required for low-level memory or hardware access
        unsafe {
            loop {
                total_load += (*group).load.load(Ordering::Acquire);
                
                if (*group).next.is_null() {
                    break;
                }
                group = (*group).next;
            }
        }
        
        total_load
    }
}

/// Scheduling domain topology
pub struct SchedDomainTopology {
    /// Domains for each CPU
    pub cpu_domains: [[Option<SchedDomain>; MAX_NR_DOMAINS]; MAX_NR_CPUS],
    
    /// Number of CPUs
    pub nr_cpus: u32,
    
    /// Number of domain levels
    pub nr_levels: u32,
    
    /// Initialized flag
    pub initialized: AtomicBool,
}

impl SchedDomainTopology {
    pub const fn new() -> Self {
        SchedDomainTopology {
            cpu_domains: [const { [const { None }; MAX_NR_DOMAINS] }; MAX_NR_CPUS],
            nr_cpus: 0,
            nr_levels: 0,
            initialized: AtomicBool::new(false),
        }
    }
    
    /// Initialize scheduling domain topology
    pub fn init(&mut self, nr_cpus: u32) {
        self.nr_cpus = nr_cpus;
        
        // Create domain hierarchy based on CPU topology
        // For simplicity, we create a flat topology (all CPUs in one domain)
        
        // Level 0: All CPUs in one domain (MC level)
        for cpu in 0..nr_cpus as usize {
            let mut domain = SchedDomain::new();
            domain.level = 0;
            domain.name = *b"MC\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
            
            // Add all CPUs to this domain
            for c in 0..nr_cpus as usize {
                domain.span.set_cpu(c);
            }
            
            // Create one group per CPU
            // (simplified - in reality would be based on cache topology)
            self.cpu_domains[cpu][0] = Some(domain);
        }
        
        self.nr_levels = 1;
        self.initialized.store(true, Ordering::Release);
    }
    
    /// Get the domain for a CPU at a given level
    pub fn get_domain(&self, cpu: usize, level: usize) -> Option<&SchedDomain> {
        if cpu >= MAX_NR_CPUS || level >= MAX_NR_DOMAINS {
            return None;
        }
        
        self.cpu_domains[cpu][level].as_ref()
    }
    
    /// Get the highest level domain for a CPU
    pub fn get_highest_domain(&self, cpu: usize) -> Option<&SchedDomain> {
        if cpu >= MAX_NR_CPUS {
            return None;
        }
        
        for level in (0..MAX_NR_DOMAINS).rev() {
            if self.cpu_domains[cpu][level].is_some() {
                return self.cpu_domains[cpu][level].as_ref();
            }
        }
        
        None
    }
}

/// Load balancer
pub struct LoadBalancer {
    /// Topology
    pub topology: SchedDomainTopology,
    
    /// Balance interval (milliseconds)
    pub interval: AtomicU32,
    
    /// Last balance time
    pub last_balance: AtomicU64,
    
    /// Statistics
    pub stats: LoadBalancerStats,
}

/// Load balancer statistics
pub struct LoadBalancerStats {
    pub balance_count: AtomicU64,
    pub migration_count: AtomicU64,
    pub failed_count: AtomicU64,
}

impl LoadBalancer {
    pub const fn new() -> Self {
        LoadBalancer {
            topology: SchedDomainTopology::new(),
            interval: AtomicU32::new(1),
            last_balance: AtomicU64::new(0),
            stats: LoadBalancerStats {
                balance_count: AtomicU64::new(0),
                migration_count: AtomicU64::new(0),
                failed_count: AtomicU64::new(0),
            },
        }
    }
    
    /// Initialize load balancer
    pub fn init(&mut self, nr_cpus: u32) {
        self.topology.init(nr_cpus);
    }
    
    /// Perform load balancing for a CPU
    pub fn balance(&mut self, _cpu: usize, _current_time: u64) -> bool {
        // TODO: Implement load balancing algorithm
        // 1. Find the domain for this CPU
        // 2. Find the busiest group in the domain
        // 3. Find the busiest CPU in the busiest group
        // 4. Move tasks from busiest to current CPU
        
        self.stats.balance_count.fetch_add(1, Ordering::Relaxed);
        false
    }
    
    /// Check if load balancing is needed
    pub fn should_balance(&self, cpu: usize, current_time: u64) -> bool {
        let last = self.last_balance.load(Ordering::Acquire);
        let interval = self.interval.load(Ordering::Acquire) as u64;
        
        // Check if enough time has passed
        if current_time < last + interval {
            return false;
        }
        
        // Check domain imbalance
        if let Some(domain) = self.topology.get_highest_domain(cpu) {
            let load = domain.calculate_load();
            let nr_cpus = domain.span.weight();
            
            if nr_cpus == 0 {
                return false;
            }
            
            let avg_load = load / nr_cpus;
            let threshold = (avg_load * domain.imbalance_pct) / 100;
            
            // Check if any group exceeds threshold
            let busiest = domain.find_busiest_group();
            if !busiest.is_null() {
                // SAFETY: unsafe block required for low-level memory or hardware access
                unsafe {
                    let busiest_load = (*busiest).load.load(Ordering::Acquire);
                    return busiest_load > threshold;
                }
            }
        }
        
        false
    }
}

/// Global load balancer
static LOAD_BALANCER: crate::sync_oncelock::OnceLock<LoadBalancer> = crate::sync_oncelock::OnceLock::new();

/// Get the load balancer
pub fn load_balancer() -> &'static LoadBalancer {
    LOAD_BALANCER.get_or_init(LoadBalancer::new)
}

/// Initialize scheduling domains
pub fn init_sched_domains(nr_cpus: u32) {
    load_balancer().init(nr_cpus);
}
