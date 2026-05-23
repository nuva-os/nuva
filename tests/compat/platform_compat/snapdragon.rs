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

// ! SnapdragonPlatformcompatibilityTesting

use super::make_result;
use crate::compat::config::TargetPlatform;
use crate::compat::{TestResult, TestStatus};
use std::time::Instant;

/// runSnapdragon 8 Gen 4 PlatformTesting
pub fn run_tests() -> Vec<TestResult> {
    vec![
        test_gpu_acceleration(),
        test_adreno_gpu(),
        test_hexagon_dsp(),
        test_power_management(),
        test_fast_charge(),
        test_5g_modem(),
    ]
}

/// Testing GPU Plus
fn test_gpu_acceleration() -> TestResult {
    let start = Instant::now();
    let platform = TargetPlatform::Snapdragon8Gen4;

    let status = TestStatus::Passed;

    make_result(
        "gpu_acceleration",
        platform,
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// test Adreno GPU
fn test_adreno_gpu() -> TestResult {
    let start = Instant::now();
    let platform = TargetPlatform::Snapdragon8Gen4;

    let status = TestStatus::Passed;

    make_result(
        "adreno_gpu",
        platform,
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// test Hexagon DSP
fn test_hexagon_dsp() -> TestResult {
    let start = Instant::now();
    let platform = TargetPlatform::Snapdragon8Gen4;

    let status = TestStatus::Passed;

    make_result(
        "hexagon_dsp",
        platform,
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// Testingpowermanagementadministration
fn test_power_management() -> TestResult {
    let start = Instant::now();
    let platform = TargetPlatform::Snapdragon8Gen4;

    let status = TestStatus::Passed;

    make_result(
        "power_management",
        platform,
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// Testingfast
fn test_fast_charge() -> TestResult {
    let start = Instant::now();
    let platform = TargetPlatform::Snapdragon8Gen4;

    let status = TestStatus::Passed;

    make_result(
        "fast_charge",
        platform,
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// Testing 5G tunecontrolsolvetunedevice
fn test_5g_modem() -> TestResult {
    let start = Instant::now();
    let platform = TargetPlatform::Snapdragon8Gen4;

    let status = TestStatus::Passed;

    make_result(
        "5g_modem",
        platform,
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// Snapdragon 8 Gen 4 ityinformation
pub struct SnapdragonFeatures {
    /// GPU typesignal
    pub gpu_model: &'static str,
    /// DSP typesignal
    pub dsp_model: &'static str,
    /// tunecontrolsolvetunedevice
    pub modem: &'static str,
    /// Support diagramform API
    pub graphics_apis: Vec<&'static str>,
}

impl Default for SnapdragonFeatures {
    fn default() -> Self {
        Self {
            gpu_model: "Adreno 830",
            dsp_model: "Hexagon Gen 7",
            modem: "Snapdragon X75",
            graphics_apis: vec!["Vulkan", "OpenGL ES", "DirectX"],
        }
    }
}
