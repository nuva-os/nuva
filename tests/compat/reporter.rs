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

// ! TestingReportdeviceModule

use super::{TestCategory, TestReport, TestResult, TestStatus};
use crate::compat::config::TestConfig;
use std::fs::{self, File};
use std::io::Write as IoWrite;
use std::path::Path;

/// TestingReportdevice
pub struct TestReporter {
    output_dir: String,
    json_report: bool,
    text_report: bool,
    verbose: bool,
}

impl TestReporter {
    /// createnew Reportdevice
    pub fn new(config: &TestConfig) -> Self {
        Self {
            output_dir: config.output_dir.clone(),
            json_report: config.json_report,
            text_report: config.text_report,
            verbose: config.verbose,
        }
    }

    /// generateReport
    pub fn generate(&self, report: &TestReport) -> Result<(), ReportError> {
        // certainprotectedoutputDirectoryExists
        fs::create_dir_all(&self.output_dir).map_err(|e| ReportError::IoError(e.to_string()))?;

        if self.json_report {
            self.generate_json_report(report)?;
        }

        if self.text_report {
            self.generate_text_report(report)?;
        }

        if self.verbose {
            self.print_summary(report);
        }

        Ok(())
    }

    /// generate JSON gridstyleReport
    fn generate_json_report(&self, report: &TestReport) -> Result<(), ReportError> {
        let path = Path::new(&self.output_dir).join("compat-test-report.json");
        let mut file = File::create(&path).map_err(|e| ReportError::IoError(e.to_string()))?;

        let json = self.report_to_json(report);
        file.write_all(json.as_bytes())
            .map_err(|e| ReportError::IoError(e.to_string()))?;

        Ok(())
    }

    /// generatetextbookgridstyleReport
    fn generate_text_report(&self, report: &TestReport) -> Result<(), ReportError> {
        let path = Path::new(&self.output_dir).join("compat-test-report.txt");
        let mut file = File::create(&path).map_err(|e| ReportError::IoError(e.to_string()))?;

        let text = self.report_to_text(report);
        file.write_all(text.as_bytes())
            .map_err(|e| ReportError::IoError(e.to_string()))?;

        Ok(())
    }

    /// willReportconvertas JSON
    fn report_to_json(&self, report: &TestReport) -> String {
        let mut json = String::new();
        json.push_str(
            "{
",
        );
        json.push_str(&format!(
            " \"passed\": {},
",
            report.passed
        ));
        json.push_str(&format!(
            " \"failed\": {},
",
            report.failed
        ));
        json.push_str(&format!(
            " \"skipped\": {},
",
            report.skipped
        ));
        json.push_str(&format!(
            " \"pass_rate\": {:.4},
",
            report.pass_rate()
        ));
        json.push_str(
            " \"results\": [
",
        );

        for (i, result) in report.results.iter().enumerate() {
            json.push_str(
                " {
",
            );
            json.push_str(&format!(
                " \"name\": \"{}\",
",
                result.name
            ));
            json.push_str(&format!(
                " \"category\": \"{}\",
",
                self.category_to_string(result.category)
            ));
            json.push_str(&format!(
                " \"status\": \"{}\",
",
                self.status_to_string(&result.status)
            ));
            json.push_str(&format!(
                " \"duration_us\": {},
",
                result.duration_us
            ));

            if let Some(ref arch) = result.arch {
                json.push_str(&format!(
                    " \"arch\": \"{}\",
",
                    arch
                ));
            }

            if let Some(ref platform) = result.platform {
                json.push_str(&format!(
                    " \"platform\": \"{}\",
",
                    platform
                ));
            }

            json.push_str(" \"duration_ms\": ");
            json.push_str(&format!("{:.3}", result.duration_us as f64 / 1000.0));
            json.push_str(
                "
",
            );

            json.push_str(" }");
            if i < report.results.len() - 1 {
                json.push(',');
            }
            json.push('\n');
        }

        json.push_str(
            " ]
",
        );
        json.push_str(
            "}
",
        );
        json
    }

    /// willReportconvertastextbook
    fn report_to_text(&self, report: &TestReport) -> String {
        let mut text = String::new();

        text.push_str(
            "========================================
",
        );
        text.push_str(
            " Nuva OS Compatibility Test Report
",
        );
        text.push_str(
            "========================================

",
        );

        text.push_str(
            "Summary:
",
        );
        text.push_str(&format!(
            " Passed: {}
",
            report.passed
        ));
        text.push_str(&format!(
            " Failed: {}
",
            report.failed
        ));
        text.push_str(&format!(
            " Skipped: {}
",
            report.skipped
        ));
        text.push_str(&format!(
            " Pass Rate: {:.2}%

",
            report.pass_rate() * 100.0
        ));

        text.push_str(
            "Details:
",
        );
        text.push_str(
            "----------------------------------------
",
        );

        for result in &report.results {
            let status_icon = match &result.status {
                TestStatus::Passed => "[PASS]",
                TestStatus::Failed(_) => "[FAIL]",
                TestStatus::Skipped(_) => "[SKIP]",
            };

            text.push_str(&format!(
                "{} {} ({})
",
                status_icon,
                result.name,
                self.category_to_string(result.category)
            ));

            if let Some(ref arch) = result.arch {
                text.push_str(&format!(
                    " Arch: {}
",
                    arch
                ));
            }

            if let Some(ref platform) = result.platform {
                text.push_str(&format!(
                    " Platform: {}
",
                    platform
                ));
            }

            text.push_str(&format!(
                " Duration: {:.3} ms
",
                result.duration_us as f64 / 1000.0
            ));

            if let TestStatus::Failed(ref msg) = result.status {
                text.push_str(&format!(
                    " Error: {}
",
                    msg
                ));
            }

            text.push('\n');
        }

        text.push_str(
            "========================================
",
        );
        text
    }

    /// printstampsummarywanttocontrolcontrol
    fn print_summary(&self, report: &TestReport) {
        println!(
            "
========================================"
        );
        println!(" Compatibility Test Summary");
        println!("========================================");
        println!("Passed: {}", report.passed);
        println!("Failed: {}", report.failed);
        println!("Skipped: {}", report.skipped);
        println!("Pass Rate: {:.2}%", report.pass_rate() * 100.0);
        println!(
            "========================================
"
        );
    }

    /// categorycategorybranchString
    fn category_to_string(&self, category: TestCategory) -> &'static str {
        match category {
            TestCategory::ArchCompat => "arch_compat",
            TestCategory::PlatformCompat => "platform_compat",
            TestCategory::AbiCompat => "abi_compat",
            TestCategory::PosixCompat => "posix_compat",
        }
    }

    /// statebranchString
    fn status_to_string(&self, status: &TestStatus) -> String {
        match status {
            TestStatus::Passed => "passed".to_string(),
            TestStatus::Failed(msg) => format!("failed: {}", msg),
            TestStatus::Skipped(reason) => format!("skipped: {}", reason),
        }
    }
}

/// ReportError
#[derive(Debug)]
pub enum ReportError {
    /// IO error
    IoError(String),
    /// formatError
    FormatError(String),
}

impl std::fmt::Display for ReportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReportError::IoError(msg) => write!(f, "IO error: {}", msg),
            ReportError::FormatError(msg) => write!(f, "Format error: {}", msg),
        }
    }
}

impl std::error::Error for ReportError {}

/// ArchitectureerrordifferentReport
#[derive(Debug, Default)]
pub struct ArchDiffReport {
    /// errordifferentprojectList
    pub diffs: Vec<ArchDiff>,
}

impl ArchDiffReport {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_diff(&mut self, diff: ArchDiff) {
        self.diffs.push(diff);
    }
}

/// formitemArchitectureerrordifferent
#[derive(Debug)]
pub struct ArchDiff {
    /// errordifferentname
    pub name: String,
    /// ARM64 rowas
    pub arm64_behavior: String,
    /// x64 rowas
    pub x64_behavior: String,
    /// iswhetherasclosekeyerrordifferent
    pub is_critical: bool,
}

/// PlatformcompatibilityReport
#[derive(Debug, Default)]
pub struct PlatformCompatReport {
    /// Platformname
    pub platform: String,
    /// Support feature
    pub supported_features: Vec<String>,
    /// notSupport feature
    pub unsupported_features: Vec<String>,
    /// warninginformation
    pub warnings: Vec<String>,
}

impl PlatformCompatReport {
    pub fn new(platform: impl Into<String>) -> Self {
        Self {
            platform: platform.into(),
            ..Self::default()
        }
    }
}

/// POSIX compatibilityReport
#[derive(Debug, Default)]
pub struct PosixCompatReport {
    /// all Interface
    pub fully_compatible: Vec<String>,
    /// partsplit Interface
    pub partially_compatible: Vec<PartialCompat>,
    /// not Interface
    pub incompatible: Vec<String>,
}

impl PosixCompatReport {
    pub fn new() -> Self {
        Self::default()
    }
}

/// partsplitinformation
#[derive(Debug)]
pub struct PartialCompat {
    /// Interfacename
    pub interface: String,
    /// defectlose feature
    pub missing_features: Vec<String>,
    /// errordifferentbright
    pub notes: String,
}
