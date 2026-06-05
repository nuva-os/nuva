/*
 * Nuva OS - Kernel - Equipment - Mod
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
 * Nuva OS - Kernel - Equipment Mode Fault Domain
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * NvEquipmentFaultDomain: independent fault domain for EL1 services.
 * Each equipment service runs in its own fault domain with:
 * - Independent address space (memory isolation)
 * - Independent capability boundary
 * - Supervisor call capability set (EL1→EL2 authorized operations)
 * - Heartbeat monitoring for liveness detection
 *
 * INVARIANT: Equipment services are isolated by independent fault domains.
 * INVARIANT: One service fault does not affect other services.
 * INVARIANT: ∀s ∈ EquipmentServices: crash(s) → healthy(KernelMode)
 */

pub mod fault_domain;
pub mod monitor;
pub mod recovery;

pub use fault_domain::{NvEquipmentFaultDomain, NvEquipmentServiceState, NvRestartPolicy};
pub use monitor::NvEquipmentMonitor;
pub use recovery::NvEquipmentRecovery;

/// Initialize equipment mode subsystem
pub fn init_equipment() {
    log_info!("NvEquipment fault domain subsystem initialized");
}
