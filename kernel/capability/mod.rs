/*
 * Nuva OS - Kernel - Capability - Mod
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
 * Nuva OS - Kernel - Capability-Based Security Module
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Nuva native capability model replacing POSIX uid/gid and Linux LSM.
 * INVARIANT: token_id is kernel-issued and unforgeable.
 * INVARIANT: child.rights ⊆ parent.rights (permission monotonicity).
 */

pub mod nv_capability;
pub mod manager;

pub use nv_capability::{NvCapability, NvResourceType, NvRightsSet};
pub use manager::NvCapabilityManager;
