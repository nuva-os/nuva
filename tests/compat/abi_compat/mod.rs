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

// ! ABI compatibilityTestingModule

pub mod calling_conv;
pub mod struct_layout;
pub mod syscall_number;

use crate::compat::config::TargetArch;
use crate::compat::{TestCategory, TestResult, TestStatus};
use alloc::format;
use alloc::vec::Vec;

/// ABI compatibilityTestingsuitecase
pub struct AbiCompatSuite {
    target_arch: TargetArch,
}

impl AbiCompatSuite {
    pub fn new(target_arch: TargetArch) -> Self {
        Self { target_arch }
    }

    pub fn run_all(&self) -> Vec<TestResult> {
        let mut results = Vec::new();
        results.extend(struct_layout::run_tests(self.target_arch));
        results.extend(calling_conv::run_tests(self.target_arch));
        results.extend(syscall_number::run_tests(self.target_arch));
        results
    }
}

fn make_result(name: &str, arch: TargetArch, status: TestStatus, duration_us: u64) -> TestResult {
    TestResult {
        name: format!("abi_{}", name),
        category: TestCategory::AbiCompat,
        status,
        duration_us,
        arch: Some(arch.as_str().to_string()),
        platform: None,
    }
}
