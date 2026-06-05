/*
 * Nuva OS - Kernel - Net - Ndp - Ra
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
 *
 * Nuva OS - Kernel - NDP Router Advertisement Processing
 *
 * Router Advertisement processing, prefix management, SLAAC address
 * autoconfiguration, and RS retransmission logic per RFC 4861.
 */

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::kernel::net::ipv6::Ipv6Addr;
use crate::kernel::error::{KernelError, KernelResult};

/// Prefix information from a Router Advertisement
#[derive(Debug, Clone)]
pub struct PrefixInfo {
    pub prefix: Ipv6Addr,
    pub prefix_len: u8,
    pub valid_lifetime: u32,
    pub preferred_lifetime: u32,
    pub on_link: bool,
    pub autonomous: bool,
    pub received_at: u64,
    pub deprecated: bool,
}

impl PrefixInfo {
    pub fn is_valid(&self, now: u64) -> bool {
        now.saturating_sub(self.received_at) < self.valid_lifetime as u64
    }
    pub fn is_preferred(&self, now: u64) -> bool {
        now.saturating_sub(self.received_at) < self.preferred_lifetime as u64
    }
    pub fn update_deprecation(&mut self, now: u64) {
        self.deprecated = !self.is_preferred(now);
    }
}

/// Router information tracked from RA messages
#[derive(Debug, Clone)]
pub struct RouterInfo {
    pub addr: Ipv6Addr,
    pub lifetime: u16,
    pub hop_limit: u8,
    pub reachable_time: u32,
    pub retrans_timer: u32,
    pub managed: bool,
    pub other: bool,
    pub last_ra_at: u64,
}

impl RouterInfo {
    pub fn is_valid(&self, now: u64) -> bool {
        if self.lifetime == 0 { return false; }
        now.saturating_sub(self.last_ra_at) < self.lifetime as u64
    }
}

pub struct RaProcessor {
    routers: BTreeMap<Ipv6Addr, RouterInfo>,
    prefixes: BTreeMap<(Ipv6Addr, u8), PrefixInfo>,
    rs_state: RsState,
    authorized_routers: Vec<Ipv6Addr>,
    ra_guard_enabled: bool,
}

#[derive(Debug, Clone)]
struct RsState {
    rs_sent: u32,
    max_rtr_solicitations: u32,
    rtr_solicitation_interval_ms: u64,
    next_rs_time: u64,
}

impl RsState {
    fn new(max_rtr_solicitations: u32, rtr_solicitation_interval_ms: u64) -> Self {
        RsState { rs_sent: 0, max_rtr_solicitations, rtr_solicitation_interval_ms, next_rs_time: 0 }
    }
    fn can_send(&self) -> bool { self.rs_sent < self.max_rtr_solicitations }
    fn record_sent(&mut self, now: u64) -> u64 {
        self.rs_sent += 1;
        let backoff = self.rtr_solicitation_interval_ms << self.rs_sent.min(3);
        self.next_rs_time = now.saturating_add(backoff);
        backoff
    }
    fn is_time_for_rs(&self, now: u64) -> bool { self.can_send() && now >= self.next_rs_time }
    fn reset(&mut self) { self.rs_sent = self.max_rtr_solicitations; }
}

impl RaProcessor {
    pub fn new(max_rtr_solicitations: u32, rtr_solicitation_interval_ms: u64) -> Self {
        RaProcessor {
            routers: BTreeMap::new(),
            prefixes: BTreeMap::new(),
            rs_state: RsState::new(max_rtr_solicitations, rtr_solicitation_interval_ms),
            authorized_routers: Vec::new(),
            ra_guard_enabled: false,
        }
    }

    pub fn enable_ra_guard(&mut self, authorized: Vec<Ipv6Addr>) {
        self.authorized_routers = authorized;
        self.ra_guard_enabled = true;
    }

    pub fn disable_ra_guard(&mut self) {
        self.ra_guard_enabled = false;
        self.authorized_routers.clear();
    }

    pub fn is_router_authorized(&self, router_addr: &Ipv6Addr) -> bool {
        if !self.ra_guard_enabled { return true; }
        self.authorized_routers.iter().any(|a| a == router_addr)
    }

    pub fn process_ra(
        &mut self, router_addr: Ipv6Addr, hop_limit: u8, managed: bool, other: bool,
        router_lifetime: u16, reachable_time: u32, retrans_timer: u32, now: u64,
    ) -> KernelResult<()> {
        if !self.is_router_authorized(&router_addr) { return Err(KernelError::AccessDenied); }
        let router_info = RouterInfo {
            addr: router_addr, lifetime: router_lifetime, hop_limit,
            reachable_time, retrans_timer, managed, other, last_ra_at: now,
        };
        self.routers.insert(router_addr, router_info);
        self.rs_state.reset();
        Ok(())
    }

    pub fn add_prefix(&mut self, prefix: Ipv6Addr, prefix_len: u8, valid_lifetime: u32,
        preferred_lifetime: u32, on_link: bool, autonomous: bool, now: u64) {
        if valid_lifetime < preferred_lifetime { return; }
        let info = PrefixInfo {
            prefix, prefix_len, valid_lifetime, preferred_lifetime,
            on_link, autonomous, received_at: now, deprecated: preferred_lifetime == 0,
        };
        self.prefixes.insert((prefix, prefix_len), info);
    }

    pub fn slaac_generate(&self, prefix: &Ipv6Addr, prefix_len: u8, iid: &[u8; 8], now: u64) -> Option<Ipv6Addr> {
        let info = self.prefixes.get(&(*prefix, prefix_len))?;
        if !info.autonomous || !info.is_valid(now) { return None; }
        let mut addr = [0u8; 16];
        let prefix_bytes = prefix_len / 8;
        for i in 0..prefix_bytes as usize { addr[i] = prefix.bytes[i]; }
        for i in 0..8 { addr[8 + i] = iid[i]; }
        Some(Ipv6Addr::new(addr))
    }

    pub fn should_send_rs(&mut self, now: u64) -> bool { self.rs_state.is_time_for_rs(now) }
    pub fn record_rs_sent(&mut self, now: u64) -> u64 { self.rs_state.record_sent(now) }
    pub fn get_default_routers(&self, now: u64) -> Vec<&RouterInfo> {
        self.routers.values().filter(|r| r.is_valid(now)).collect()
    }
    pub fn get_on_link_prefixes(&self) -> Vec<&PrefixInfo> {
        self.prefixes.values().filter(|p| p.on_link).collect()
    }
    pub fn update_prefix_deprecation(&mut self, now: u64) {
        for info in self.prefixes.values_mut() { info.update_deprecation(now); }
    }
    pub fn cleanup_expired_routers(&mut self, now: u64) {
        let expired: Vec<Ipv6Addr> = self.routers.iter()
            .filter(|(_, r)| !r.is_valid(now)).map(|(a, _)| *a).collect();
        for addr in expired { self.routers.remove(&addr); }
    }
    pub fn cleanup_expired_prefixes(&mut self, now: u64) {
        let expired: Vec<(Ipv6Addr, u8)> = self.prefixes.iter()
            .filter(|(_, p)| !p.is_valid(now)).map(|(k, _)| *k).collect();
        for key in expired { self.prefixes.remove(&key); }
    }
}
