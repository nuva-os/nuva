/*
 * Nuva OS - Kernel - Net - Ndp - Core
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
 * Nuva OS - Kernel - NDP Core Coordinator
 *
 * Central coordinator for the Neighbor Discovery Protocol.
 * Dispatches ICMPv6 NDP messages, manages the neighbor cache,
 * NUD state machines, DAD engine, RA processor, and security.
 */

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::kernel::net::ipv6::Ipv6Addr;
use crate::kernel::net::icmpv6::icmpv6_type;
use crate::kernel::capability::nv_capability::{NvCapability, NvRightsSet, NvResourceType};
use crate::kernel::error::{KernelError, KernelResult};

use super::cache::{NeighborCache, CacheKey, NeighborEntry};
use super::nud::{NudState, NudEvent, NudAction, NudMachine, NudTimer};
use super::ra::RaProcessor;
use super::dad::{DadEngine, DadState};
use super::security::{NdpSecurity, SendVerifier, NoopSendVerifier};
use super::config::NdpConfig;
use super::stats::NdpStats;

/// NDP message metadata extracted from incoming ICMPv6
#[derive(Debug, Clone)]
pub struct NdpMessage {
    /// ICMPv6 type code
    pub msg_type: u8,
    /// Source IPv6 address
    pub src_addr: Ipv6Addr,
    /// Destination IPv6 address
    pub dst_addr: Ipv6Addr,
    /// Hop limit from the IPv6 header
    pub hop_limit: u8,
    /// Interface index where the message was received
    pub ifindex: u32,
    /// Target address (from NS/NA/Redirect)
    pub target_addr: Ipv6Addr,
    /// Source link-layer address option (if present)
    pub slla: Option<[u8; 6]>,
    /// Target link-layer address option (if present)
    pub tlla: Option<[u8; 6]>,
    /// Router Advertisement fields (only valid for RA)
    pub ra_hop_limit: u8,
    pub ra_managed: bool,
    pub ra_other: bool,
    pub ra_router_lifetime: u16,
    pub ra_reachable_time: u32,
    pub ra_retrans_timer: u32,
    /// Redirect destination (only valid for Redirect)
    pub redirect_dst: Option<Ipv6Addr>,
}

impl NdpMessage {
    /// Create a minimal NDP message with required fields
    pub fn new(msg_type: u8, src_addr: Ipv6Addr, dst_addr: Ipv6Addr,
               hop_limit: u8, ifindex: u32, target_addr: Ipv6Addr) -> Self {
        NdpMessage {
            msg_type, src_addr, dst_addr, hop_limit, ifindex, target_addr,
            slla: None, tlla: None,
            ra_hop_limit: 0, ra_managed: false, ra_other: false,
            ra_router_lifetime: 0, ra_reachable_time: 0, ra_retrans_timer: 0,
            redirect_dst: None,
        }
    }
}

/// NUD machine key: (address, ifindex)
type NudKey = (Ipv6Addr, u32);

/// Core NDP protocol coordinator
pub struct NdpCore {
    /// Neighbor cache
    cache: NeighborCache,
    /// Per-entry NUD state machines
    nud_machines: BTreeMap<NudKey, NudMachine>,
    /// Router Advertisement processor
    ra: RaProcessor,
    /// Duplicate Address Detection engine
    dad: DadEngine,
    /// Security validator
    security: NdpSecurity,
    /// SEND verifier
    send_verifier: NoopSendVerifier,
    /// Configuration
    config: NdpConfig,
    /// Statistics
    stats: NdpStats,
    /// Whether NDP is initialized
    initialized: bool,
}

impl NdpCore {
    /// Create a new NdpCore with the given configuration
    pub fn new(config: NdpConfig) -> Self {
        let max_entries = config.max_entries;
        let max_probes = config.max_probes;
        let dad_transmits = config.dad_transmits;
        let max_rs = config.max_rtr_solicitations;
        let rs_interval = config.rtr_solicitation_interval_ms;
        NdpCore {
            cache: NeighborCache::new(max_entries),
            nud_machines: BTreeMap::new(),
            ra: RaProcessor::new(max_rs, rs_interval),
            dad: DadEngine::new(dad_transmits, config.retrans_timer_ms),
            security: NdpSecurity::new(),
            send_verifier: NoopSendVerifier,
            config,
            stats: NdpStats::new(),
            initialized: false,
        }
    }

    /// Initialize the NDP core
    pub fn init(&mut self) {
        if self.initialized { return; }
        self.initialized = true;
        log_info!("NDP core initialized");
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Main entry point: receive and dispatch an ICMPv6 NDP message
    pub fn receive_icmpv6(&mut self, msg: &NdpMessage, now: u64) -> KernelResult<()> {
        if !self.initialized {
            return Err(KernelError::InvalidArgument);
        }

        // Security: validate hop limit = 255 and source address constraints
        let is_redirect = msg.msg_type == icmpv6_type::ND_REDIRECT;
        if let Err(e) = self.security.validate_ndp_message(msg.hop_limit, &msg.src_addr, is_redirect) {
            self.stats.inc_security_failures();
            return Err(e);
        }

        match msg.msg_type {
            icmpv6_type::ND_RS => self.handle_rs(msg, now),
            icmpv6_type::ND_RA => self.handle_ra(msg, now),
            icmpv6_type::ND_NS => self.handle_ns(msg, now),
            icmpv6_type::ND_NA => self.handle_na(msg, now),
            icmpv6_type::ND_REDIRECT => self.handle_redirect(msg, now),
            _ => Err(KernelError::InvalidArgument),
        }
    }

    /// Resolve a neighbor: returns MAC address if reachable, triggers resolution otherwise
    pub fn resolve_neighbor(&mut self, addr: &Ipv6Addr, ifindex: u32, now: u64) -> KernelResult<Option<[u8; 6]>> {
        let key = CacheKey::new(*addr, ifindex);
        if let Some(mac) = self.cache.lookup_mac(&key) {
            self.stats.inc_cache_hits();
            return Ok(Some(mac));
        }

        // Cache miss: create incomplete entry and start NUD
        self.stats.inc_cache_misses();
        self.cache.create_incomplete(key.clone());

        let nud_key = (*addr, ifindex);
        let machine = NudMachine::new(self.config.max_probes);
        self.nud_machines.insert(nud_key, machine);

        // Trigger initial NS probe
        let nud_key_ref = (*addr, ifindex);
        if let Some(machine) = self.nud_machines.get_mut(&nud_key_ref) {
            let actions = machine.transition(NudEvent::ProbeTimeout);
            self.process_nud_actions(&nud_key_ref, &actions, now);
        }

        self.stats.inc_ns_sent();
        Ok(None)
    }

    /// Handle Router Solicitation (RS) - typically on a router interface
    fn handle_rs(&mut self, _msg: &NdpMessage, _now: u64) -> KernelResult<()> {
        // In a host-only implementation, RS is received by routers.
        // Log for diagnostics but take no action on a host.
        log_debug!("NDP: received Router Solicitation");
        Ok(())
    }

    /// Handle Router Advertisement (RA)
    fn handle_ra(&mut self, msg: &NdpMessage, now: u64) -> KernelResult<()> {
        // Validate RA source (RA Guard)
        if let Err(e) = self.security.validate_ra_source(&msg.src_addr) {
            self.stats.inc_security_failures();
            return Err(e);
        }

        // Process RA in the RA processor
        self.ra.process_ra(
            msg.src_addr,
            msg.ra_hop_limit,
            msg.ra_managed,
            msg.ra_other,
            msg.ra_router_lifetime,
            msg.ra_reachable_time,
            msg.ra_retrans_timer,
            now,
        )?;

        // Update neighbor cache with router link-layer address
        if let Some(slla) = msg.slla {
            let key = CacheKey::new(msg.src_addr, msg.ifindex);
            let entry = NeighborEntry::new(slla, NudState::Stale, true);
            self.cache.insert(key, entry);
        }

        self.stats.inc_ra_received();
        log_debug!("NDP: processed Router Advertisement from {:?}", msg.src_addr);
        Ok(())
    }

    /// Handle Neighbor Solicitation (NS)
    fn handle_ns(&mut self, msg: &NdpMessage, now: u64) -> KernelResult<()> {
        // Check for DAD conflict: if the target is our tentative address
        if self.dad.is_duplicate(&msg.target_addr, msg.ifindex) {
            return Ok(());
        }
        if self.dad.get_state(&msg.target_addr, msg.ifindex) == Some(DadState::Tentative) {
            // DAD conflict detected
            self.dad.handle_conflict(&msg.target_addr, msg.ifindex)?;
            self.stats.inc_dad_conflicts();
            log_warn!("NDP: DAD conflict detected for {:?}", msg.target_addr);
            return Ok(());
        }

        // Update neighbor cache with sender SLLA
        if let Some(slla) = msg.slla {
            let key = CacheKey::new(msg.src_addr, msg.ifindex);
            if let Some(existing) = self.cache.lookup(&key) {
                if existing.mac_addr != slla {
                    // MAC address changed: move to STALE per RFC 4861
                    let entry = NeighborEntry::new(slla, NudState::Stale, existing.is_router);
                    self.cache.insert(key, entry);
                }
            } else {
                // New neighbor: create STALE entry
                let entry = NeighborEntry::new(slla, NudState::Stale, false);
                self.cache.insert(key, entry);
            }
        }

        log_debug!("NDP: processed Neighbor Solicitation for {:?}", msg.target_addr);
        let _ = now; // used for future timer scheduling
        Ok(())
    }

    /// Handle Neighbor Advertisement (NA)
    fn handle_na(&mut self, msg: &NdpMessage, now: u64) -> KernelResult<()> {
        let key = CacheKey::new(msg.target_addr, msg.ifindex);
        let nud_key = (msg.target_addr, msg.ifindex);

        // Process NUD transition for ReceiveNA event
        let mut actions = Vec::new();
        if let Some(machine) = self.nud_machines.get_mut(&nud_key) {
            actions = machine.transition(NudEvent::ReceiveNA);
        }

        // Update neighbor cache
        if let Some(tlla) = msg.tlla {
            let is_router = msg.ra_router_lifetime > 0;
            let entry = NeighborEntry::new(tlla, NudState::Reachable, is_router);
            self.cache.insert(key, entry);
        } else if self.cache.lookup(&key).is_some() {
            // No TLLA: just update state to Reachable
            self.cache.update_state(&CacheKey::new(msg.target_addr, msg.ifindex), NudState::Reachable);
        }

        // Process NUD actions
        self.process_nud_actions(&nud_key, &actions, now);

        self.stats.inc_na_sent();
        log_debug!("NDP: processed Neighbor Advertisement for {:?}", msg.target_addr);
        Ok(())
    }

    /// Handle Redirect message
    fn handle_redirect(&mut self, msg: &NdpMessage, now: u64) -> KernelResult<()> {
        // Validate redirect per security rules
        if let Some(ref redirect_dst) = msg.redirect_dst {
            if let Err(e) = self.security.validate_redirect(&msg.src_addr, redirect_dst, msg.hop_limit) {
                self.stats.inc_security_failures();
                return Err(e);
            }
        }

        // Update neighbor cache with target link-layer address if present
        if let (Some(tlla), Some(ref redirect_dst)) = (msg.tlla, msg.redirect_dst) {
            let key = CacheKey::new(*redirect_dst, msg.ifindex);
            let entry = NeighborEntry::new(tlla, NudState::Stale, false);
            self.cache.insert(key, entry);
        }

        self.stats.inc_redirect_received();
        log_debug!("NDP: processed Redirect from {:?}", msg.src_addr);
        let _ = now;
        Ok(())
    }

    /// Process actions returned by NUD state machine transitions
    fn process_nud_actions(&mut self, nud_key: &NudKey, actions: &[NudAction], now: u64) {
        for action in actions {
            match action {
                NudAction::SendNS => {
                    self.stats.inc_ns_sent();
                    log_debug!("NDP: sending NS for {:?}", nud_key.0);
                }
                NudAction::SetTimer => {
                    // Timer scheduling would be done by the platform timer subsystem
                    let _ = now;
                }
                NudAction::ClearEntry => {
                    let cache_key = CacheKey::new(nud_key.0, nud_key.1);
                    self.cache.remove(&cache_key);
                    self.nud_machines.remove(nud_key);
                }
                NudAction::MarkFailed => {
                    let cache_key = CacheKey::new(nud_key.0, nud_key.1);
                    self.cache.update_state(&cache_key, NudState::Failed);
                    self.stats.inc_nud_failures();
                    log_warn!("NDP: NUD failed for {:?}", nud_key.0);
                }
                NudAction::None => {}
            }
        }
    }

    /// Handle a NUD timer expiration
    pub fn handle_nud_timeout(&mut self, addr: &Ipv6Addr, ifindex: u32, timer: NudTimer, now: u64) {
        let nud_key = (*addr, ifindex);
        let event = timer.to_event();

        let mut actions = Vec::new();
        if let Some(machine) = self.nud_machines.get_mut(&nud_key) {
            actions = machine.transition(event);
        }

        self.process_nud_actions(&nud_key, &actions, now);
    }

    /// Check that the capability grants network control rights
    pub fn check_net_control(cap: &NvCapability) -> KernelResult<()> {
        if cap.is_revoked() {
            return Err(KernelError::CapabilityExpired);
        }
        if cap.resource_type != NvResourceType::Network {
            return Err(KernelError::CapabilityDenied);
        }
        if !cap.has_rights(NvRightsSet::ADMIN) {
            return Err(KernelError::CapabilityDenied);
        }
        Ok(())
    }

    /// Start Duplicate Address Detection for an address (capability-gated)
    pub fn start_dad(&mut self, cap: &NvCapability, addr: Ipv6Addr, ifindex: u32, now: u64) -> KernelResult<DadState> {
        Self::check_net_control(cap)?;
        self.dad.start_dad(addr, ifindex, now)
    }

    /// Check if we should send a Router Solicitation
    pub fn maybe_send_rs(&mut self, now: u64) -> bool {
        self.ra.should_send_rs(now)
    }

    /// Record that a Router Solicitation was sent
    pub fn record_rs_sent(&mut self, now: u64) -> u64 {
        self.stats.inc_rs_sent();
        self.ra.record_rs_sent(now)
    }

    /// Periodic maintenance: clean up expired entries, update deprecation
    pub fn periodic_maintenance(&mut self, now: u64) {
        // Clean up expired neighbor cache entries
        let expired = self.cache.collect_expired(now);
        for key in &expired {
            self.cache.remove(key);
            let nud_key = (key.addr, key.ifindex);
            self.nud_machines.remove(&nud_key);
        }

        // Clean up expired routers and prefixes
        self.ra.cleanup_expired_routers(now);
        self.ra.cleanup_expired_prefixes(now);
        self.ra.update_prefix_deprecation(now);

        // Retransmit DAD NS for tentative addresses
        let tentative = self.dad.get_tentative_addresses(now);
        for (addr, ifindex) in tentative {
            if self.dad.should_retransmit(&addr, ifindex, now) {
                if let Ok(()) = self.dad.record_ns_sent(&addr, ifindex, now) {
                    self.stats.inc_ns_sent();
                }
            }
        }
    }

    /// Get a reference to the statistics
    pub fn stats(&self) -> &NdpStats { &self.stats }

    /// Get a reference to the configuration
    pub fn config(&self) -> &NdpConfig { &self.config }

    /// Get a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut NdpConfig { &mut self.config }

    /// Get a reference to the neighbor cache
    pub fn cache(&self) -> &NeighborCache { &self.cache }

    /// Get a reference to the RA processor
    pub fn ra_processor(&self) -> &RaProcessor { &self.ra }

    /// Get a reference to the DAD engine
    pub fn dad_engine(&self) -> &DadEngine { &self.dad }

    /// Get a reference to the security module
    pub fn security(&self) -> &NdpSecurity { &self.security }

    /// Get a mutable reference to the security module
    pub fn security_mut(&mut self) -> &mut NdpSecurity { &mut self.security }

    /// Enable RA Guard with authorized router list (capability-gated)
    pub fn enable_ra_guard(&mut self, cap: &NvCapability, authorized: Vec<Ipv6Addr>) -> KernelResult<()> {
        Self::check_net_control(cap)?;
        self.security.enable_ra_guard(authorized);
        self.ra.enable_ra_guard(alloc::vec::Vec::new());
        Ok(())
    }

    /// Disable RA Guard (capability-gated)
    pub fn disable_ra_guard(&mut self, cap: &NvCapability) -> KernelResult<()> {
        Self::check_net_control(cap)?;
        self.security.disable_ra_guard();
        self.ra.disable_ra_guard();
        Ok(())
    }
}
