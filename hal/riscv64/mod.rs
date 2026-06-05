/*
 * Nuva OS - HAL - RISC-V 64 Module
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

//! RISC-V 64 HAL module: hardware abstraction layer for RISC-V platforms.

pub mod cpu;
pub mod mmu;
pub mod interrupt;

/// Initialize RISC-V 64 HAL.
pub fn init_hal_riscv() {
    log_info!("RISC-V: HAL initialization");
    cpu::init_cpu_hal();
    mmu::init_mmu();
    interrupt::init_interrupt();
    log_info!("RISC-V: HAL initialized");
}
