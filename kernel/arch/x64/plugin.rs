/*
 * Nuva OS - Kernel - x86_64 Architecture Plugin
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

use alloc::boxed::Box;
use alloc::vec::Vec;

use super::super::{ArchOps, ArchPlugin, ArchPluginMeta, ArchType, DeviceInfo, PluginError};
use super::super::super::{PageTableOps, IrqControllerOps, TimerOps, PowerOps, ContextOps};
use super::super::super::{PhysAddr, VirtAddr, ProtFlags, CpuContext};

/// x86_64 plugin metadata
pub const X64_PLUGIN_META: ArchPluginMeta = ArchPluginMeta {
    name: "x86_64",
    version: "1.0.0",
    arch_type: ArchType::X64,
    supported_devices: &[
        "intel_core",
        "amd_ryzen",
        "generic-x86_64",
    ],
    description: "x86_64 architecture plugin",
    priority: 100,
};

/// x86_64 Architecture Plugin
pub struct X64Plugin {
    initialized: bool,
}

impl X64Plugin {
    pub const fn new() -> Self {
        Self {
            initialized: false,
        }
    }
}

impl ArchPlugin for X64Plugin {
    fn meta(&self) -> &ArchPluginMeta {
        &X64_PLUGIN_META
    }

    fn init(&self) -> Result<(), PluginError> {
        super::init_arch();
        Ok(())
    }

    fn detect(&self) -> bool {
        #[cfg(target_arch = "x86_64")]
        {
            let (_, ebx, ecx, edx) = super::cpuid(0, 0);
            let is_genuine_intel = ebx == 0x756E_6547 && ecx == 0x6E_49_65_6E && edx == 0x69_6E_65_49;
            let is_authentic_amd = ebx == 0x6874_7541 && ecx == 0x444D_4163 && edx == 0x6974_6E65;
            is_genuine_intel || is_authentic_amd
        }
        #[cfg(not(target_arch = "x86_64"))]
        false
    }

    fn ops(&self) -> &dyn ArchOps {
        &super::X64_ARCH
    }

    fn supported_devices(&self) -> &[&'static str] {
        X64_PLUGIN_META.supported_devices
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }
}
