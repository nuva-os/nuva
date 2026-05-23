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

// ! PlatformcompatibilityTestingModule

pub mod amd;
pub mod intel;
pub mod kirin;
pub mod loongson;
pub mod snapdragon;

use crate::compat::config::TargetPlatform;
use crate::compat::{TestCategory, TestResult, TestStatus};

/// PlatformcompatibilityTestingsuitecase
pub struct PlatformCompatSuite {
    target_platform: TargetPlatform,
}

impl PlatformCompatSuite {
    pub fn new(target_platform: TargetPlatform) -> Self {
        Self { target_platform }
    }

    pub fn run_all(&self) -> Vec<TestResult> {
        match self.target_platform {
            TargetPlatform::Kirin | TargetPlatform::Kirin9000 | TargetPlatform::Kirin9010 => {
                kirin::run_tests()
            }
            TargetPlatform::Snapdragon8Gen4 => snapdragon::run_tests(),
            TargetPlatform::IntelCore => intel::run_tests(),
            TargetPlatform::AmdRyzen => amd::run_tests(),
            TargetPlatform::Loongson3A6000 | TargetPlatform::Loongson3C6000 => {
                loongson::run_tests()
            }
            _ => vec![],
        }
    }
}

/// PlatformcompatibilityTesting trait
pub trait PlatformCompatTest {
    fn name(&self) -> &'static str;
    fn platform(&self) -> TargetPlatform;
    fn features(&self) -> Vec<&'static str>;
    fn run(&self) -> TestStatus;
}

fn make_result(
    name: &str,
    platform: TargetPlatform,
    status: TestStatus,
    duration_us: u64,
) -> TestResult {
    TestResult {
        name: format!("{}_{}", name, platform.as_str()),
        category: TestCategory::PlatformCompat,
        status,
        duration_us,
        arch: Some(platform.arch().as_str().to_string()),
        platform: Some(platform.as_str().to_string()),
    }
}
