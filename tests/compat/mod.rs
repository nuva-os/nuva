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

// ! compatibilityTestingFramework
/*!*/
// ! crossArchitecture、crossPlatform、ABI sum POSIX compatibilityTestingSupport

pub mod abi_compat;
pub mod arch_compat;
pub mod config;
pub mod platform_compat;
pub mod posix_compat;
pub mod reporter;
pub mod runner;

use config::TestConfig;
use reporter::TestReporter;
use runner::TestRunner;
use alloc::vec::Vec;

/// compatibilityTestingFrameworkmainstruct
pub struct CompatTestFramework {
    /// TestingConfiguration
    config: TestConfig,
    /// Testingrundevice
    runner: TestRunner,
    /// TestingReportdevice
    reporter: TestReporter,
}

impl CompatTestFramework {
    /// createnew TestingFrameworkrealexample
    pub fn new(config: TestConfig) -> Self {
        let runner = TestRunner::new(&config);
        let reporter = TestReporter::new(&config);

        Self {
            config,
            runner,
            reporter,
        }
    }

    /// runplacefinitecompatibilityTesting
    pub fn run_all(&mut self) -> Result<TestReport, TestError> {
        let mut report = TestReport::new();

        // runArchitecturecompatibilityTesting
        if self
            .config
            .test_categories
            .contains(&TestCategory::ArchCompat)
        {
            let arch_results = self.runner.run_arch_compat_tests()?;
            report.merge(arch_results);
        }

        // runPlatformcompatibilityTesting
        if self
            .config
            .test_categories
            .contains(&TestCategory::PlatformCompat)
        {
            let platform_results = self.runner.run_platform_compat_tests()?;
            report.merge(platform_results);
        }

        // run ABI compatibilityTesting
        if self
            .config
            .test_categories
            .contains(&TestCategory::AbiCompat)
        {
            let abi_results = self.runner.run_abi_compat_tests()?;
            report.merge(abi_results);
        }

        // run POSIX compatibilityTesting
        if self
            .config
            .test_categories
            .contains(&TestCategory::PosixCompat)
        {
            let posix_results = self.runner.run_posix_compat_tests()?;
            report.merge(posix_results);
        }

        // generateReport
        self.reporter.generate(&report)?;

        Ok(report)
    }

    /// runexpfixedcategorycategory Testing
    pub fn run_category(&mut self, category: TestCategory) -> Result<TestReport, TestError> {
        let report = match category {
            TestCategory::ArchCompat => self.runner.run_arch_compat_tests()?,
            TestCategory::PlatformCompat => self.runner.run_platform_compat_tests()?,
            TestCategory::AbiCompat => self.runner.run_abi_compat_tests()?,
            TestCategory::PosixCompat => self.runner.run_posix_compat_tests()?,
        };

        self.reporter.generate(&report)?;
        Ok(report)
    }

    /// GetConfigurationreference
    pub fn config(&self) -> &TestConfig {
        &self.config
    }
}

/// TestingcategorycategoryEnum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TestCategory {
    /// ArchitecturecompatibilityTesting
    ArchCompat,
    /// PlatformcompatibilityTesting
    PlatformCompat,
    /// ABI compatibilityTesting
    AbiCompat,
    /// POSIX compatibilityTesting
    PosixCompat,
}

/// TestingReport
#[derive(Debug, Default)]
pub struct TestReport {
    /// TestingresultList
    pub results: Vec<TestResult>,
    /// overcount
    pub passed: usize,
    /// failurecount
    pub failed: usize,
    /// jumpovercount
    pub skipped: usize,
}

impl TestReport {
    /// createnew TestingReport
    pub fn new() -> Self {
        Self::default()
    }

    /// combineparallelOtherReport
    pub fn merge(&mut self, other: TestReport) {
        self.results.extend(other.results);
        self.passed += other.passed;
        self.failed += other.failed;
        self.skipped += other.skipped;
    }

    /// addPlusTestingresult
    pub fn add_result(&mut self, result: TestResult) {
        match &result.status {
            TestStatus::Passed => self.passed += 1,
            TestStatus::Failed(_) => self.failed += 1,
            TestStatus::Skipped(_) => self.skipped += 1,
        }
        self.results.push(result);
    }

    /// Getoverrate
    pub fn pass_rate(&self) -> f64 {
        let total = self.passed + self.failed;
        if total == 0 {
            0.0
        } else {
            self.passed as f64 / total as f64
        }
    }
}

/// formitemTestingresult
#[derive(Debug)]
pub struct TestResult {
    /// Testingname
    pub name: String,
    /// Testingcategorycategory
    pub category: TestCategory,
    /// Testingstate
    pub status: TestStatus,
    /// executetimebetween(us)
    pub duration_us: u64,
    /// Architectureinformation
    pub arch: Option<String>,
    /// Platforminformation
    pub platform: Option<String>,
}

/// Testingstate
#[derive(Debug)]
pub enum TestStatus {
    /// through
    Passed,
    /// fail（containserrorinformation）
    Failed(String),
    /// jumpover(packetsourcefactor)
    Skipped(String),
}

/// TestingError
#[derive(Debug)]
pub struct TestError {
    pub message: String,
    pub category: Option<TestCategory>,
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TestError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_creation() {
        let config = TestConfig::default();
        let framework = CompatTestFramework::new(config);
        assert!(framework.config().parallel_jobs > 0);
    }
}
