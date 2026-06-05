/*
 * Nuva OS - Kernel - RISC-V 64 Boot Module
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

//! RISC-V boot module: early console, FDT parsing, and boot initialization.

pub mod early_console;
pub mod fdt;

/// Perform early boot initialization.
pub fn early_init(fdt_addr: *const u8) {
    early_console::init_early_console();
    fdt::init_fdt(fdt_addr);
    log_info!("RISC-V: Early boot init complete");
}
