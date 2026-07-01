/*
 * Nuva OS - Kernel - RISC-V 64 Architecture Plugin
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

//! RISC-V 64 architecture plugin implementing ArchPlugin trait.

use alloc::vec::Vec;
use alloc::string::ToString;

use crate::kernel::arch::plugins::*;
use crate::kernel::arch::*;
use super::arch_impl::RISCV64_ARCH;
use alloc::vec;

/// RISC-V 64 plugin metadata.
pub static RISCV64_PLUGIN_META: ArchPluginMeta = ArchPluginMeta {
    name: "riscv64",
    version: "1.0.0",
    arch_type: ArchType::RiscV64,
    supported_devices: &["qemu-virt", "generic-riscv"],
    description: "RISC-V 64-bit (RV64G) architecture plugin for Nuva OS",
    priority: 100,
};

/// RISC-V 64 architecture plugin.
pub struct RiscV64Plugin;

impl RiscV64Plugin {
    /// Create a new RISC-V 64 plugin instance.
    pub const fn new() -> Self {
        RiscV64Plugin
    }
}

impl ArchPlugin for RiscV64Plugin {
    fn meta(&self) -> &ArchPluginMeta {
        &RISCV64_PLUGIN_META
    }

    fn init(&self) -> Result<(), PluginError> {
        crate::kernel::arch::init_arch();
        Ok(())
    }

    fn shutdown(&self) -> Result<(), PluginError> {
        Ok(())
    }

    fn ops(&self) -> &dyn ArchOps {
        &RISCV64_ARCH
    }

    fn is_compatible(&self, device: &DeviceInfo) -> bool {
        for supported in RISCV64_PLUGIN_META.supported_devices {
            if device.name.contains(supported) || device.cpu_model.contains(supported) {
                return true;
            }
        }
        false
    }

    fn get_features(&self) -> Vec<&'static str> {
        vec!["rv64g", "sv39", "fpu", "sbi"]
    }
}

/// Global RISC-V 64 plugin instance.
pub static RISCV64_PLUGIN: RiscV64Plugin = RiscV64Plugin::new();
