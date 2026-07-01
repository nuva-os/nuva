/*
 * Nuva OS - Kernel - Net - Ndp - Mod
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
 * Nuva OS - Kernel - NDP Module Entry
 *
 * Neighbor Discovery Protocol (NDP) module for IPv6.
 * Implements RFC 4861: Neighbor Discovery for IPv6.
 */

pub mod cache;
pub mod nud;
pub mod ra;
pub mod dad;
pub mod security;
pub mod core;
pub mod config;
pub mod stats;

// Re-export key types for convenient access
pub use cache::{NeighborCache, CacheKey, NeighborEntry};
pub use nud::{NudState, NudEvent, NudAction, NudMachine, NudTimer};
pub use ra::RaProcessor;
pub use dad::{DadEngine, DadState, DadEntry};
pub use security::{NdpSecurity, SendVerifier, NoopSendVerifier};
pub use core::{NdpCore, NdpMessage};
pub use config::NdpConfig;
pub use stats::NdpStats;

use crate::kernel::net::ipv6::Ipv6Addr;
use crate::kernel::capability::nv_capability::{NvCapability, NvRightsSet, NvResourceType};
use crate::kernel::error::{KernelError, KernelResult};

/// NDP plugin: manages the global NdpCore instance
pub struct NdpPlugin {
    /// The core NDP coordinator
    core: Option<NdpCore>,
}

impl NdpPlugin {
    /// Create a new uninitialized NDP plugin
    pub const fn new() -> Self {
        NdpPlugin { core: None }
    }

    /// Initialize the NDP plugin with default configuration
    pub fn init(&mut self) -> KernelResult<()> {
        if self.core.is_some() {
            return Err(KernelError::InvalidArgument);
        }
        let config = NdpConfig::new();
        let mut ndp_core = NdpCore::new(config);
        ndp_core.init();
        self.core = Some(ndp_core);
        log_info!("NDP plugin initialized");
        Ok(())
    }

    /// Initialize the NDP plugin with custom configuration (capability-gated)
    pub fn init_with_config(&mut self, cap: &NvCapability, config: NdpConfig) -> KernelResult<()> {
        NdpCore::check_net_control(cap)?;
        if self.core.is_some() {
            return Err(KernelError::InvalidArgument);
        }
        let mut ndp_core = NdpCore::new(config);
        ndp_core.init();
        self.core = Some(ndp_core);
        log_info!("NDP plugin initialized with custom config");
        Ok(())
    }

    /// Shut down the NDP plugin
    pub fn shutdown(&mut self) {
        self.core = None;
        log_info!("NDP plugin shut down");
    }

    /// Get a reference to the NdpCore
    pub fn core(&self) -> Option<&NdpCore> {
        self.core.as_ref()
    }

    /// Get a mutable reference to the NdpCore
    pub fn core_mut(&mut self) -> Option<&mut NdpCore> {
        self.core.as_mut()
    }
}

/// Global NDP plugin instance
static NDP_PLUGIN: crate::sync_oncelock::OnceLock<NdpPlugin> = crate::sync_oncelock::OnceLock::new();

/// Initialize the NDP subsystem with default configuration
pub fn init_ndp() -> KernelResult<&'static NdpPlugin> {
    NDP_PLUGIN.get_or_init(|| {
        let mut plugin = NdpPlugin::new();
        let _ = plugin.init();
        plugin
    });
    Ok(NDP_PLUGIN.get().unwrap())
}

