/*
 * Nuva OS - HAL - RISC-V 64 MMU
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

//! RISC-V 64 MMU HAL: Memory management unit initialization and configuration.

/// Initialize MMU HAL.
pub fn init_mmu() {
    log_info!("RISC-V: MMU HAL init");
    // MMU configuration is handled by kernel/arch/riscv64/mmu.rs
    // This HAL layer provides a higher-level interface if needed
}
