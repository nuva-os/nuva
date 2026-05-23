/*
 * Nuva OS - HAL - LoongArch64
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

// LoongArch64 CPU abstraction
pub mod cpu;

// LoongArch64 MMU and page table management
pub mod mmu;

// LoongArch64 interrupt and exception handling
pub mod interrupt;

// LoongArch64 LSX 128-bit SIMD
pub mod lsx;

// LoongArch64 LASX 256-bit SIMD
pub mod lasx;

// LoongArch64 LVZ virtualization
pub mod lvz;

// LoongArch64 LBT binary translation
pub mod lbt;

/// Initialize LoongArch64 HAL
pub fn init_loongarch64_hal() {
    // Initialize MMU (page table base, TLB flush, enable paging)
    mmu::init_mmu();

    // Initialize interrupt controller (EIOINTC)
    interrupt::init_interrupt();

    // Detect and initialize SIMD extensions
    let lsx_ok = lsx::lsx_detect();
    let lasx_ok = lasx::lasx_detect();
    if lasx_ok {
        // LASX implies LSX; both are available
    }

    // Detect LVZ virtualization
    let _lvz_ok = lvz::lvz_detect();

    // Detect LBT binary translation
    let _lbt_ok = lbt::lbt_detect();
}
