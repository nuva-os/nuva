/*
 * Nuva OS - Kernel - Core - Mod
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
pub mod cache;
pub mod cpu;
pub mod defense;
pub mod kernel_thread;
pub mod mempool;
pub mod perf_tune;
pub mod posix;
pub mod random;
pub mod signal;
pub mod time;
pub mod wait;
pub mod workqueue;

// Three-level privilege architecture (EL2/EL1/EL0)
pub mod privilege;
pub mod supervisor_call;
pub mod cross_level;

pub use privilege::{NvArchPrivilegeMapping, NvTrapContext, NvSupervisorContext};
pub use supervisor_call::NvSupervisorCall;
pub use cross_level::NvCrossLevelAccessEnforcement;
