/*
 * Nuva OS - Kernel - Net - Ndp - Config
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
 * Nuva OS - Kernel - NDP Configuration
 *
 * Capability-gated configuration management for NDP.
 */

use crate::kernel::capability::nv_capability::{NvCapability, NvRightsSet, NvResourceType};
use crate::kernel::error::{KernelError, KernelResult};

/// Default maximum neighbor cache entries
const DEFAULT_MAX_ENTRIES: usize = 1024;

/// Default reachable time in milliseconds (RFC 4861: 30s)
const DEFAULT_REACHABLE_TIME_MS: u64 = 30_000;

/// Default retransmission timer in milliseconds (RFC 4861: 1s)
const DEFAULT_RETRANS_TIMER_MS: u64 = 1_000;

/// Default maximum unicast NS probes
const DEFAULT_MAX_PROBES: u32 = 3;

/// Default DAD transmit count (RFC 4861: 1)
const DEFAULT_DAD_TRANSMITS: u32 = 1;

/// Default max router solicitations (RFC 4861: 3)
const DEFAULT_MAX_RTR_SOLICITATIONS: u32 = 3;

/// Default router solicitation interval in milliseconds (RFC 4861: 4s)
const DEFAULT_RTR_SOLICITATION_INTERVAL_MS: u64 = 4_000;

/// Default hop limit for NDP messages
const DEFAULT_CUR_HOP_LIMIT: u8 = 64;

/// NDP configuration parameters
pub struct NdpConfig {
    /// Maximum neighbor cache entries
    pub max_entries: usize,
    /// Reachable time in milliseconds
    pub reachable_time_ms: u64,
    /// Retransmission timer in milliseconds
    pub retrans_timer_ms: u64,
    /// Maximum unicast NS probes before declaring failure
    pub max_probes: u32,
    /// Number of DAD NS messages to send
    pub dad_transmits: u32,
    /// Maximum router solicitations to send
    pub max_rtr_solicitations: u32,
    /// Router solicitation interval in milliseconds
    pub rtr_solicitation_interval_ms: u64,
    /// Current hop limit for outgoing NDP messages
    pub cur_hop_limit: u8,
}

impl NdpConfig {
    /// Create configuration with RFC 4861 default values
    pub const fn new() -> Self {
        NdpConfig {
            max_entries: DEFAULT_MAX_ENTRIES,
            reachable_time_ms: DEFAULT_REACHABLE_TIME_MS,
            retrans_timer_ms: DEFAULT_RETRANS_TIMER_MS,
            max_probes: DEFAULT_MAX_PROBES,
            dad_transmits: DEFAULT_DAD_TRANSMITS,
            max_rtr_solicitations: DEFAULT_MAX_RTR_SOLICITATIONS,
            rtr_solicitation_interval_ms: DEFAULT_RTR_SOLICITATION_INTERVAL_MS,
            cur_hop_limit: DEFAULT_CUR_HOP_LIMIT,
        }
    }

    /// Set max_entries (capability-gated: requires ADMIN right on Network resource)
    pub fn set_max_entries(&mut self, cap: &NvCapability, value: usize) -> KernelResult<()> {
        check_net_admin(cap)?;
        if value == 0 {
            return Err(KernelError::InvalidArgument);
        }
        self.max_entries = value;
        Ok(())
    }

    /// Set reachable_time_ms (capability-gated)
    pub fn set_reachable_time_ms(&mut self, cap: &NvCapability, value: u64) -> KernelResult<()> {
        check_net_admin(cap)?;
        if value == 0 {
            return Err(KernelError::InvalidArgument);
        }
        self.reachable_time_ms = value;
        Ok(())
    }

    /// Set retrans_timer_ms (capability-gated)
    pub fn set_retrans_timer_ms(&mut self, cap: &NvCapability, value: u64) -> KernelResult<()> {
        check_net_admin(cap)?;
        if value == 0 {
            return Err(KernelError::InvalidArgument);
        }
        self.retrans_timer_ms = value;
        Ok(())
    }

    /// Set max_probes (capability-gated)
    pub fn set_max_probes(&mut self, cap: &NvCapability, value: u32) -> KernelResult<()> {
        check_net_admin(cap)?;
        if value == 0 {
            return Err(KernelError::InvalidArgument);
        }
        self.max_probes = value;
        Ok(())
    }

    /// Set dad_transmits (capability-gated)
    pub fn set_dad_transmits(&mut self, cap: &NvCapability, value: u32) -> KernelResult<()> {
        check_net_admin(cap)?;
        self.dad_transmits = value;
        Ok(())
    }

    /// Set max_rtr_solicitations (capability-gated)
    pub fn set_max_rtr_solicitations(&mut self, cap: &NvCapability, value: u32) -> KernelResult<()> {
        check_net_admin(cap)?;
        self.max_rtr_solicitations = value;
        Ok(())
    }

    /// Set rtr_solicitation_interval_ms (capability-gated)
    pub fn set_rtr_solicitation_interval_ms(&mut self, cap: &NvCapability, value: u64) -> KernelResult<()> {
        check_net_admin(cap)?;
        self.rtr_solicitation_interval_ms = value;
        Ok(())
    }

    /// Set cur_hop_limit (capability-gated)
    pub fn set_cur_hop_limit(&mut self, cap: &NvCapability, value: u8) -> KernelResult<()> {
        check_net_admin(cap)?;
        self.cur_hop_limit = value;
        Ok(())
    }
}

/// Check that the capability grants ADMIN rights on a Network resource.
///
/// This is the Nuva capability-based equivalent of Linux CAP_NET_ADMIN.
fn check_net_admin(cap: &NvCapability) -> KernelResult<()> {
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
