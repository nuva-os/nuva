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

// ! TestingConfigurationModule

use super::TestCategory;

/// TestingConfiguration
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// targetArchitecture
    pub target_arch: TargetArch,
    /// targetPlatform
    pub target_platform: TargetPlatform,
    /// Testingcategorycategory
    pub test_categories: Vec<TestCategory>,
    /// Paralleltaskservicenumber
    pub parallel_jobs: usize,
    /// timeouttimebetween(ms)
    pub timeout_ms: u64,
    /// outputDirectory
    pub output_dir: String,
    /// fineoutput
    pub verbose: bool,
    /// generate JSON Report
    pub json_report: bool,
    /// generatetextbookReport
    pub text_report: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            target_arch: TargetArch::All,
            target_platform: TargetPlatform::All,
            test_categories: vec![
                TestCategory::ArchCompat,
                TestCategory::PlatformCompat,
                TestCategory::AbiCompat,
                TestCategory::PosixCompat,
            ],
            parallel_jobs: 4,
            timeout_ms: 30000,
            output_dir: "target/compat-test".to_string(),
            verbose: false,
            json_report: true,
            text_report: true,
        }
    }
}

impl TestConfig {
    /// createnew ConfigurationBuilddevice
    pub fn builder() -> TestConfigBuilder {
        TestConfigBuilder::default()
    }

    /// onlyTesting ARM64 Architecture
    pub fn arm64_only() -> Self {
        Self {
            target_arch: TargetArch::Arm64,
            ..Self::default()
        }
    }

    /// onlyTesting x86-64 Architecture
    pub fn x64_only() -> Self {
        Self {
            target_arch: TargetArch::X64,
            ..Self::default()
        }
    }

    /// VerificationConfigurationvalidity
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.parallel_jobs == 0 {
            return Err(ConfigError::InvalidParallelJobs);
        }
        if self.timeout_ms == 0 {
            return Err(ConfigError::InvalidTimeout);
        }
        if self.test_categories.is_empty() {
            return Err(ConfigError::NoTestCategories);
        }
        Ok(())
    }
}

/// ConfigurationBuilddevice
#[derive(Debug, Default)]
pub struct TestConfigBuilder {
    config: TestConfig,
}

impl TestConfigBuilder {
    pub fn target_arch(mut self, arch: TargetArch) -> Self {
        self.config.target_arch = arch;
        self
    }

    pub fn target_platform(mut self, platform: TargetPlatform) -> Self {
        self.config.target_platform = platform;
        self
    }

    pub fn test_categories(mut self, categories: Vec<TestCategory>) -> Self {
        self.config.test_categories = categories;
        self
    }

    pub fn parallel_jobs(mut self, jobs: usize) -> Self {
        self.config.parallel_jobs = jobs;
        self
    }

    pub fn timeout_ms(mut self, timeout: u64) -> Self {
        self.config.timeout_ms = timeout;
        self
    }

    pub fn output_dir(mut self, dir: impl Into<String>) -> Self {
        self.config.output_dir = dir.into();
        self
    }

    pub fn verbose(mut self, verbose: bool) -> Self {
        self.config.verbose = verbose;
        self
    }

    pub fn build(self) -> Result<TestConfig, ConfigError> {
        let config = self.config;
        config.validate()?;
        Ok(config)
    }
}

/// targetArchitecture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetArch {
    /// placefiniteArchitecture
    All,
    /// ARM64 Architecture
    Arm64,
    /// x86-64 Architecture
    X64,
    /// LongcoreArchitecture (LoongArch64)
    LoongArch64,
}

impl TargetArch {
    /// GetArchitecturename
    pub fn as_str(&self) -> &'static str {
        match self {
            TargetArch::All => "all",
            TargetArch::Arm64 => "arm64",
            TargetArch::X64 => "x64",
            TargetArch::LoongArch64 => "loongarch64",
        }
    }

    /// GetplacefiniteArchitectureList
    pub fn all_archs() -> Vec<TargetArch> {
        vec![TargetArch::Arm64, TargetArch::X64, TargetArch::LoongArch64]
    }
}

/// targetPlatform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPlatform {
    /// placefinitePlatform
    All,
    /// seathoughtKirinsystemcolumn (use)
    Kirin,
    /// Kirin 9000 systemcolumn
    Kirin9000,
    /// Kirin 9010 systemcolumn
    Kirin9010,
    /// Snapdragon 8 Gen 4
    Snapdragon8Gen4,
    /// use x64
    GenericX64,
    /// Intel Core
    IntelCore,
    /// AMD Ryzen
    AmdRyzen,
    /// Longcore 3A6000 systemcolumn
    Loongson3A6000,
    /// Longcore 3C6000 systemcolumn (serviceservicedevice)
    Loongson3C6000,
}

impl TargetPlatform {
    /// GetPlatformname
    pub fn as_str(&self) -> &'static str {
        match self {
            TargetPlatform::All => "all",
            TargetPlatform::Kirin => "kirin",
            TargetPlatform::Kirin9000 => "kirin9000",
            TargetPlatform::Kirin9010 => "kirin9010",
            TargetPlatform::Snapdragon8Gen4 => "snapdragon8gen4",
            TargetPlatform::GenericX64 => "generic-x64",
            TargetPlatform::IntelCore => "intel-core",
            TargetPlatform::AmdRyzen => "amd-ryzen",
            TargetPlatform::Loongson3A6000 => "loongson3a6000",
            TargetPlatform::Loongson3C6000 => "loongson3c6000",
        }
    }

    /// GetPlatformlogshould Architecture
    pub fn arch(&self) -> TargetArch {
        match self {
            TargetPlatform::Kirin
            | TargetPlatform::Kirin9000
            | TargetPlatform::Kirin9010
            | TargetPlatform::Snapdragon8Gen4 => TargetArch::Arm64,
            TargetPlatform::GenericX64 | TargetPlatform::IntelCore | TargetPlatform::AmdRyzen => {
                TargetArch::X64
            }
            TargetPlatform::Loongson3A6000 | TargetPlatform::Loongson3C6000 => {
                TargetArch::LoongArch64
            }
            TargetPlatform::All => TargetArch::All,
        }
    }

    /// Get ARM64 PlatformList
    pub fn arm64_platforms() -> Vec<TargetPlatform> {
        vec![
            TargetPlatform::Kirin,
            TargetPlatform::Kirin9000,
            TargetPlatform::Kirin9010,
            TargetPlatform::Snapdragon8Gen4,
        ]
    }

    /// Get x64 PlatformList
    pub fn x64_platforms() -> Vec<TargetPlatform> {
        vec![
            TargetPlatform::GenericX64,
            TargetPlatform::IntelCore,
            TargetPlatform::AmdRyzen,
        ]
    }

    /// GetLongcorePlatformList
    pub fn loongarch_platforms() -> Vec<TargetPlatform> {
        vec![
            TargetPlatform::Loongson3A6000,
            TargetPlatform::Loongson3C6000,
        ]
    }

    /// GetseathoughtKirinPlatformList
    pub fn kirin_platforms() -> Vec<TargetPlatform> {
        vec![
            TargetPlatform::Kirin,
            TargetPlatform::Kirin9000,
            TargetPlatform::Kirin9010,
        ]
    }
}

/// ConfigurationError
#[derive(Debug)]
pub enum ConfigError {
    /// invalid Paralleltaskservicenumber
    InvalidParallelJobs,
    /// invalid timeouttimebetween
    InvalidTimeout,
    /// finiteexpfixedTestingcategorycategory
    NoTestCategories,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidParallelJobs => write!(f, "parallel_jobs must be greater than 0"),
            ConfigError::InvalidTimeout => write!(f, "timeout_ms must be greater than 0"),
            ConfigError::NoTestCategories => write!(f, "at least one test category is required"),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;
use alloc::vec;
use alloc::vec::Vec;

    #[test]
    fn test_default_config() {
        let config = TestConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_builder() {
        let config = TestConfig::builder()
            .target_arch(TargetArch::Arm64)
            .parallel_jobs(8)
            .verbose(true)
            .build()
            .unwrap();

        assert_eq!(config.target_arch, TargetArch::Arm64);
        assert_eq!(config.parallel_jobs, 8);
        assert!(config.verbose);
    }

    #[test]
    fn test_platform_arch_mapping() {
        assert_eq!(TargetPlatform::Kirin.arch(), TargetArch::Arm64);
        assert_eq!(TargetPlatform::Kirin9000.arch(), TargetArch::Arm64);
        assert_eq!(TargetPlatform::IntelCore.arch(), TargetArch::X64);
        assert_eq!(
            TargetPlatform::Loongson3A6000.arch(),
            TargetArch::LoongArch64
        );
        assert_eq!(
            TargetPlatform::Loongson3C6000.arch(),
            TargetArch::LoongArch64
        );
    }
}
