/*
 * Nuva OS
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

// ! TestingModule

pub mod bench;
pub mod compat;
pub mod driver;
pub mod e2e;
pub mod integration;
pub mod kat;
pub mod layer;
pub mod perf;
pub mod performance;
pub mod plugin;
pub mod quantum;
pub mod system;
pub mod toolchain;
pub mod unit;

/// runplacefiniteTesting
pub fn run_all_tests() {
    compat::run_compat_tests();
    integration::run_integration_tests();
    performance::run_performance_tests();
}
