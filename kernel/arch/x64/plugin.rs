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

use crate::kernel::arch::plugins::{ArchPlugin, ArchPluginMeta, ArchType, DeviceInfo, PluginError};
use crate::kernel::arch::{ArchOps, PageTableOps, IrqControllerOps, TimerOps, PowerOps, ContextOps};
use crate::kernel::arch::{PhysAddr, VirtAddr, ProtFlags, CpuContext};

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

    fn shutdown(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn ops(&self) -> &dyn ArchOps {
        &super::X64_ARCH
    }

    fn is_compatible(&self, device: &DeviceInfo) -> bool {
        X64_PLUGIN_META.supported_devices.contains(&device.name.as_str())
    }

    fn get_features(&self) -> Vec<&'static str> {
        vec!["x86_64", "sse", "sse2", "fxsr"]
    }
}
