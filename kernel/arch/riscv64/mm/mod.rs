/*
 * Nuva OS - Kernel - RISC-V 64 Memory Management Helpers
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

//! RISC-V 64 memory management helper module.
//! Re-exports page table types and constants.

pub use super::mmu::{
    PagingMode, Pte, PTE_V, PTE_R, PTE_W, PTE_X, PTE_U, PTE_G, PTE_A, PTE_D,
    RiscV64PageTable,
};
