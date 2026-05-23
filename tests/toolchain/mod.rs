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

// ! ToollinkcollectionsuccessTesting
/*!*/
// ! endtoendTestingsumPerformancebasecriterionTesting

pub mod benchmark;
pub mod e2e;
pub mod integration;

use std::path::PathBuf;
use std::time::Duration;

/// TestingFramework
pub struct TestFramework {
    /// TestingConfiguration
    config: TestConfig,
    /// Testingresult
    results: Vec<TestResult>,
}

impl TestFramework {
    pub fn new(config: TestConfig) -> Self {
        Self {
            config,
            results: vec![],
        }
    }

    /// runplacefiniteTesting
    pub fn run_all(&mut self) -> TestSummary {
        // runendtoendTesting
        self.run_e2e_tests();

        // runcollectionsuccessTesting
        self.run_integration_tests();

        // runbasecriterionTesting
        if self.config.run_benchmarks {
            self.run_benchmarks();
        }

        self.generate_summary()
    }

    /// runendtoendTesting
    fn run_e2e_tests(&mut self) {
        let e2e_tests = e2e::collect_e2e_tests(&self.config);

        for test in e2e_tests {
            let result = self.run_test(&test);
            self.results.push(result);
        }
    }

    /// runcollectionsuccessTesting
    fn run_integration_tests(&mut self) {
        let integration_tests = integration::collect_integration_tests(&self.config);

        for test in integration_tests {
            let result = self.run_test(&test);
            self.results.push(result);
        }
    }

    /// runbasecriterionTesting
    fn run_benchmarks(&mut self) {
        let benchmarks = benchmark::collect_benchmarks(&self.config);

        for bench in benchmarks {
            let result = self.run_benchmark(&bench);
            self.results.push(result);
        }
    }

    /// runformitemTesting
    fn run_test(&self, test: &Test) -> TestResult {
        let start = std::time::Instant::now();

        let outcome = match test.kind {
            TestKind::E2E => e2e::run_e2e_test(test),
            TestKind::Integration => integration::run_integration_test(test),
            TestKind::Benchmark => benchmark::run_benchmark_test(test),
        };

        let duration = start.elapsed();

        TestResult {
            name: test.name.clone(),
            kind: test.kind,
            outcome,
            duration,
            metadata: test.metadata.clone(),
        }
    }

    /// runbasecriterionTesting
    fn run_benchmark(&self, bench: &Test) -> TestResult {
        let start = std::time::Instant::now();

        // runmanytimetakeflatvalue
        let iterations = self.config.benchmark_iterations;
        let mut total_duration = Duration::ZERO;

        for _ in 0..iterations {
            let iter_start = std::time::Instant::now();
            let _ = benchmark::run_benchmark_test(bench);
            total_duration += iter_start.elapsed();
        }

        let avg_duration = total_duration / iterations as u32;

        TestResult {
            name: bench.name.clone(),
            kind: TestKind::Benchmark,
            outcome: TestOutcome::Passed,
            duration: avg_duration,
            metadata: bench.metadata.clone(),
        }
    }

    /// generateTestingsummarywant
    fn generate_summary(&self) -> TestSummary {
        let total = self.results.len();
        let passed = self
            .results
            .iter()
            .filter(|r| r.outcome == TestOutcome::Passed)
            .count();
        let failed = self
            .results
            .iter()
            .filter(|r| r.outcome == TestOutcome::Failed)
            .count();
        let skipped = self
            .results
            .iter()
            .filter(|r| r.outcome == TestOutcome::Skipped)
            .count();

        let total_duration: Duration = self.results.iter().map(|r| r.duration).sum();

        TestSummary {
            total,
            passed,
            failed,
            skipped,
            total_duration,
            results: self.results.clone(),
        }
    }
}

/// TestingConfiguration
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// TestingrootDirectory
    pub test_root: PathBuf,
    /// iswhetherrunbasecriterionTesting
    pub run_benchmarks: bool,
    /// basecriterionTestingeratimenumber
    pub benchmark_iterations: usize,
    /// ParallelTestingnumber
    pub parallel_jobs: usize,
    /// timeouttimebetween
    pub timeout: Duration,
    /// targetArchitecture
    pub target_arch: String,
    /// targetPlatform
    pub target_platform: String,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            test_root: PathBuf::from("tests"),
            run_benchmarks: true,
            benchmark_iterations: 10,
            parallel_jobs: 4,
            timeout: Duration::from_secs(300),
            target_arch: "aarch64".to_string(),
            target_platform: "kirin9020".to_string(),
        }
    }
}

/// test
#[derive(Debug, Clone)]
pub struct Test {
    /// Testingname
    pub name: String,
    /// TestingType
    pub kind: TestKind,
    /// TestingPath
    pub path: PathBuf,
    /// data
    pub metadata: TestMetadata,
}

/// TestingType
#[derive(Debug, Clone, Copy)]
pub enum TestKind {
    E2E,
    Integration,
    Benchmark,
}

/// Testingdata
#[derive(Debug, Clone, Default)]
pub struct TestMetadata {
    /// description
    pub description: Option<String>,
    /// Label
    pub tags: Vec<String>,
    /// dependency
    pub dependencies: Vec<String>,
}

/// Testingresult
#[derive(Debug, Clone)]
pub struct TestResult {
    /// Testingname
    pub name: String,
    /// TestingType
    pub kind: TestKind,
    /// Testingresult
    pub outcome: TestOutcome,
    /// consumetime
    pub duration: Duration,
    /// data
    pub metadata: TestMetadata,
}

/// Testingresult
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestOutcome {
    Passed,
    Failed,
    Skipped,
    Timeout,
}

/// Testingsummarywant
#[derive(Debug, Clone)]
pub struct TestSummary {
    /// total
    pub total: usize,
    /// overnumber
    pub passed: usize,
    /// failurenumber
    pub failed: usize,
    /// jumpovernumber
    pub skipped: usize,
    /// totalconsumetime
    pub total_duration: Duration,
    /// resultlist
    pub results: Vec<TestResult>,
}

impl TestSummary {
    /// printstampsummarywant
    pub fn print(&self) {
        println!(
            "
=== Test Summary ==="
        );
        println!("Total: {}", self.total);
        println!(
            "Passed: {} ({:.1}%)",
            self.passed,
            self.passed as f64 / self.total as f64 * 100.0
        );
        println!(
            "Failed: {} ({:.1}%)",
            self.failed,
            self.failed as f64 / self.total as f64 * 100.0
        );
        println!(
            "Skipped: {} ({:.1}%)",
            self.skipped,
            self.skipped as f64 / self.total as f64 * 100.0
        );
        println!("Duration: {:?}", self.total_duration);

        if self.failed > 0 {
            println!(
                "
=== Failed Tests ==="
            );
            for result in &self.results {
                if result.outcome == TestOutcome::Failed {
                    println!(" - {} ({:?})", result.name, result.duration);
                }
            }
        }
    }
}
