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

// ! POSIX compatibilityTestingModule

pub mod file_ops;
pub mod process_ops;
pub mod pthread_ops;
pub mod signal_ops;

use crate::compat::{TestCategory, TestResult, TestStatus};

/// POSIX compatibilityTestingsuitecase
pub struct PosixCompatSuite;

impl PosixCompatSuite {
    pub fn new() -> Self {
        Self
    }

    pub fn run_all(&self) -> Vec<TestResult> {
        let mut results = Vec::new();
        results.extend(file_ops::run_tests());
        results.extend(process_ops::run_tests());
        results.extend(signal_ops::run_tests());
        results.extend(pthread_ops::run_tests());
        results
    }
}

fn make_result(name: &str, status: TestStatus, duration_us: u64) -> TestResult {
    TestResult {
        name: format!("posix_{}", name),
        category: TestCategory::PosixCompat,
        status,
        duration_us,
        arch: None,
        platform: None,
    }
}

/// POSIX levelcategory
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PosixComplianceLevel {
    /// all
    Full,
    /// partsplit
    Partial,
    /// not
    None,
}

/// POSIX Interfacestate
pub struct PosixInterfaceStatus {
    /// Interfacename
    pub name: String,
    /// levelcategory
    pub compliance: PosixComplianceLevel,
    /// defectlose feature
    pub missing_features: Vec<String>,
    /// note
    pub notes: Option<String>,
}

impl PosixInterfaceStatus {
    pub fn full(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            compliance: PosixComplianceLevel::Full,
            missing_features: vec![],
            notes: None,
        }
    }

    pub fn partial(name: impl Into<String>, missing: Vec<String>) -> Self {
        Self {
            name: name.into(),
            compliance: PosixComplianceLevel::Partial,
            missing_features: missing,
            notes: None,
        }
    }

    pub fn none(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            compliance: PosixComplianceLevel::None,
            missing_features: vec![],
            notes: None,
        }
    }
}
