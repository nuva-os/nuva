/*
 * Nuva OS - Kernel - Net - Ndp - Stats
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
 * Nuva OS - Kernel - NDP Statistics
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Lock-free statistics counters for the Neighbor Discovery Protocol.
 */

use core::sync::atomic::{AtomicU64, Ordering};

/// NDP statistics counters (lock-free, AtomicU64)
pub struct NdpStats {
    /// Router Solicitations sent
    pub rs_sent: AtomicU64,
    /// Router Advertisements received
    pub ra_received: AtomicU64,
    /// Neighbor Solicitations sent
    pub ns_sent: AtomicU64,
    /// Neighbor Advertisements sent
    pub na_sent: AtomicU64,
    /// Neighbor cache hits
    pub cache_hits: AtomicU64,
    /// Neighbor cache misses
    pub cache_misses: AtomicU64,
    /// DAD conflicts detected
    pub dad_conflicts: AtomicU64,
    /// NUD failures
    pub nud_failures: AtomicU64,
    /// Redirect messages received
    pub redirect_received: AtomicU64,
    /// Security validation failures
    pub security_failures: AtomicU64,
}

impl NdpStats {
    /// Create a new zeroed statistics instance
    pub const fn new() -> Self {
        NdpStats {
            rs_sent: AtomicU64::new(0),
            ra_received: AtomicU64::new(0),
            ns_sent: AtomicU64::new(0),
            na_sent: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            dad_conflicts: AtomicU64::new(0),
            nud_failures: AtomicU64::new(0),
            redirect_received: AtomicU64::new(0),
            security_failures: AtomicU64::new(0),
        }
    }

    pub fn inc_rs_sent(&self) { self.rs_sent.fetch_add(1, Ordering::Relaxed); }
    pub fn inc_ra_received(&self) { self.ra_received.fetch_add(1, Ordering::Relaxed); }
    pub fn inc_ns_sent(&self) { self.ns_sent.fetch_add(1, Ordering::Relaxed); }
    pub fn inc_na_sent(&self) { self.na_sent.fetch_add(1, Ordering::Relaxed); }
    pub fn inc_cache_hits(&self) { self.cache_hits.fetch_add(1, Ordering::Relaxed); }
    pub fn inc_cache_misses(&self) { self.cache_misses.fetch_add(1, Ordering::Relaxed); }
    pub fn inc_dad_conflicts(&self) { self.dad_conflicts.fetch_add(1, Ordering::Relaxed); }
    pub fn inc_nud_failures(&self) { self.nud_failures.fetch_add(1, Ordering::Relaxed); }
    pub fn inc_redirect_received(&self) { self.redirect_received.fetch_add(1, Ordering::Relaxed); }
    pub fn inc_security_failures(&self) { self.security_failures.fetch_add(1, Ordering::Relaxed); }

    pub fn get_rs_sent(&self) -> u64 { self.rs_sent.load(Ordering::Relaxed) }
    pub fn get_ra_received(&self) -> u64 { self.ra_received.load(Ordering::Relaxed) }
    pub fn get_ns_sent(&self) -> u64 { self.ns_sent.load(Ordering::Relaxed) }
    pub fn get_na_sent(&self) -> u64 { self.na_sent.load(Ordering::Relaxed) }
    pub fn get_cache_hits(&self) -> u64 { self.cache_hits.load(Ordering::Relaxed) }
    pub fn get_cache_misses(&self) -> u64 { self.cache_misses.load(Ordering::Relaxed) }
    pub fn get_dad_conflicts(&self) -> u64 { self.dad_conflicts.load(Ordering::Relaxed) }
    pub fn get_nud_failures(&self) -> u64 { self.nud_failures.load(Ordering::Relaxed) }
    pub fn get_redirect_received(&self) -> u64 { self.redirect_received.load(Ordering::Relaxed) }
    pub fn get_security_failures(&self) -> u64 { self.security_failures.load(Ordering::Relaxed) }
}
