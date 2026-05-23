/*
 * Nuva OS - Kernel - Test Framework
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

pub mod benchmarks;
pub mod optimization_tests;
pub mod integration;
pub mod benchmark;
pub mod stress;
pub mod regression;

/// Run all tests
pub fn run_all_tests() {
    benchmarks::run_benchmarks();
    optimization_tests::run_optimization_tests();
    integration::run_integration_tests();
    benchmark::run_performance_tests();
    stress::run_stress_tests();
    regression::run_regression_tests();
}
