/*
 * Nuva OS - Hal - Quantum - Qrng - HealthTest
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
/*
 * QRNG Health Tests - NIST SP 800-90B Compliant
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Implements mandatory health tests for physical entropy sources
 * as required by NIST SP 800-90B Section 4:
 * - Restart Test (power-on health test)
 * - Repetition Count Test (detects stuck-at faults)
 * - Adaptive Proportion Test (detects bias faults)
 * - Continuous runtime monitoring
 */

use core::sync::atomic::{AtomicBool, AtomicU64, AtomicU32, Ordering};
use alloc::vec::Vec;

use super::QrngProvider;

/// Maximum sample size for adaptive proportion test
pub const ADAPTIVE_PROPORTION_WINDOW: usize = 512;

/// Cutoff values for NIST SP 800-90B health tests
/// Repetition Count: C = 1 + ceil(-log2(alpha) / H)
/// For alpha=2^-30, H=1.0: C=31; H=0.5: C=61
pub const REPETITION_COUNT_CUTOFF_H1: u32 = 31;
pub const REPETITION_COUNT_CUTOFF_H05: u32 = 61;

/// Adaptive Proportion: for W=512, alpha=2^-20, H=1.0: C≈33
pub const ADAPTIVE_PROPORTION_CUTOFF_H1: u32 = 33;

/// Number of restart test samples
pub const RESTART_TEST_SAMPLES: usize = 1024;

/// Min-entropy per sample (scaled by 1000 for fixed-point)
/// H=1.0 -> 1000, H=0.5 -> 500
pub type MinEntropyThousandths = u32;

/// Health test result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthTestResult {
    /// Test passed
    Pass = 0,
    /// Test failed - entropy source may be compromised
    Fail = 1,
    /// Test not yet performed
    NotRun = 2,
    /// Insufficient data
    InsufficientData = 3,
}

/// Health test error type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthTestError {
    /// QRNG provider error
    QrngError,
    /// Test configuration invalid
    InvalidConfig,
    /// Continuous test already initialized
    AlreadyInitialized,
}

/// Comprehensive health test results
#[derive(Debug, Clone)]
pub struct HealthTestReport {
    /// Restart test result
    pub restart_test: HealthTestResult,
    /// Repetition count test result
    pub repetition_count_test: HealthTestResult,
    /// Adaptive proportion test result
    pub adaptive_proportion_test: HealthTestResult,
    /// Overall health status
    pub overall: HealthTestResult,
    /// Number of samples tested
    pub samples_tested: usize,
}

impl HealthTestReport {
    /// Create empty report
    pub const fn new() -> Self {
        HealthTestReport {
            restart_test: HealthTestResult::NotRun,
            repetition_count_test: HealthTestResult::NotRun,
            adaptive_proportion_test: HealthTestResult::NotRun,
            overall: HealthTestResult::NotRun,
            samples_tested: 0,
        }
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.restart_test == HealthTestResult::Pass
            && self.repetition_count_test == HealthTestResult::Pass
            && self.adaptive_proportion_test == HealthTestResult::Pass
    }
}

/// Repetition Count Test (NIST SP 800-90B Section 4.4.1)
/// Detects stuck-at faults where the entropy source produces
/// the same value repeatedly.
/// C = 1 + ceil(-log2(alpha) / H)
/// where H is the per-sample min-entropy and alpha is the
/// acceptable false positive rate.
/// For alpha = 2^-30 and H = 1 bit: C = 31
/// For alpha = 2^-30 and H = 0.5 bit: C = 61
pub struct RepetitionCountTest {
    /// Cutoff value (maximum allowed repetitions)
    cutoff: u32,
    /// Current repeat count
    current_count: u32,
    /// Last observed value
    last_value: Option<u8>,
    /// Test result
    result: AtomicBool,
}

impl RepetitionCountTest {
    /// Create new test with given min-entropy per sample
    /// @param min_entropy_thousandths: Min-entropy per sample * 1000
    /// (e.g., 1000 for H=1.0 bit, 500 for H=0.5 bit)
    /// Cutoff C = 1 + ceil(30 / H) for alpha = 2^-30
    pub fn new(min_entropy_thousandths: MinEntropyThousandths) -> Self {
        let cutoff = if min_entropy_thousandths >= 1000 {
            REPETITION_COUNT_CUTOFF_H1
        } else if min_entropy_thousandths >= 500 {
            REPETITION_COUNT_CUTOFF_H05
        } else if min_entropy_thousandths > 0 {
            let h_times_10 = min_entropy_thousandths / 100;
            let c = (300 / h_times_10) + 1;
            c.min(200)
        } else {
            REPETITION_COUNT_CUTOFF_H05
        };

        RepetitionCountTest {
            cutoff,
            current_count: 0,
            last_value: None,
            result: AtomicBool::new(true),
        }
    }

    /// Create with default parameters (H=1.0 bit, alpha=2^-30)
    pub fn with_defaults() -> Self {
        Self::new(1000)
    }

    /// Process a single sample
    /// Returns false if the test detects a failure (stuck-at fault)
    pub fn test_sample(&mut self, sample: u8) -> bool {
        match self.last_value {
            Some(lv) if lv == sample => {
                self.current_count += 1;
                if self.current_count > self.cutoff {
                    self.result.store(false, Ordering::Release);
                    return false;
                }
            }
            _ => {
                self.last_value = Some(sample);
                self.current_count = 1;
            }
        }
        true
    }

    /// Get current test result
    pub fn result(&self) -> HealthTestResult {
        if self.result.load(Ordering::Acquire) {
            HealthTestResult::Pass
        } else {
            HealthTestResult::Fail
        }
    }

    /// Reset test state
    pub fn reset(&mut self) {
        self.current_count = 0;
        self.last_value = None;
        self.result.store(true, Ordering::Release);
    }

    /// Get cutoff value
    pub fn cutoff(&self) -> u32 {
        self.cutoff
    }
}

/// Adaptive Proportion Test (NIST SP 800-90B Section 4.4.2)
/// Detects bias faults where a specific value appears too
/// frequently within a window of samples.
/// Uses a sliding window of W samples and checks if any
/// value appears more than C times.
/// For W=512, alpha=2^-20, H=1: C ≈ 33
pub struct AdaptiveProportionTest {
    /// Window size
    window_size: usize,
    /// Cutoff value (max occurrences of any value in window)
    cutoff: u32,
    /// Frequency counts for each byte value (simplified: track max only)
    max_count: u32,
    /// Samples processed in current window
    window_position: usize,
    /// Test result
    result: AtomicBool,
}

impl AdaptiveProportionTest {
    /// Create new test with given parameters
    pub fn new(window_size: usize, min_entropy_thousandths: MinEntropyThousandths) -> Self {
        let cutoff = Self::compute_cutoff(window_size, min_entropy_thousandths);

        AdaptiveProportionTest {
            window_size,
            cutoff,
            max_count: 0,
            window_position: 0,
            result: AtomicBool::new(true),
        }
    }

    /// Create with default parameters (W=512, H=1.0)
    pub fn with_defaults() -> Self {
        Self::new(ADAPTIVE_PROPORTION_WINDOW, 1000)
    }

    /// Compute cutoff for adaptive proportion test
    /// For W=512, H=1.0, alpha=2^-20: C = 33
    /// For other parameters, use simplified integer computation:
    /// C = W / 2^H + 5 * sqrt(W / 2^H * (1 - 1/2^H))
    /// Using scaled integer arithmetic for no_std.
    fn compute_cutoff(window_size: usize, min_entropy_thousandths: MinEntropyThousandths) -> u32 {
        if min_entropy_thousandths >= 1000 {
            if window_size >= 512 {
                ADAPTIVE_PROPORTION_CUTOFF_H1
            } else {
                let w = window_size as u32;
                let mean = w / 2;
                let c = mean + 5 * isqrt(mean / 2);
                c.min(window_size as u32)
            }
        } else if min_entropy_thousandths >= 500 {
            let w = window_size as u32;
            let mean = w / 3;
            let c = mean + 5 * isqrt(mean * 2 / 3);
            c.min(window_size as u32)
        } else {
            window_size as u32
        }
    }

    /// Process a single sample
    /// Returns false if any value exceeds the cutoff in the window
    pub fn test_sample(&mut self, sample: u8) -> bool {
        if sample == self.target_symbol {
            self.max_count += 1;
        }

        self.window_position += 1;

        if self.window_position >= self.window_size {
            if self.max_count > self.cutoff {
                self.result.store(false, Ordering::Release);
                return false;
            }
            self.window_position = 0;
            self.max_count = 0;
        }

        true
    }

    /// Process a batch of samples
    /// More efficient than single-sample processing for
    /// the full frequency count approach.
    pub fn test_batch(&mut self, samples: &[u8]) -> bool {
        for chunk in samples.chunks(self.window_size) {
            let mut counts = [0u32; 256];
            for &byte in chunk.iter() {
                counts[byte as usize] += 1;
            }

            let max = *counts.iter().max().unwrap_or(&0);
            if max > self.cutoff {
                self.result.store(false, Ordering::Release);
                return false;
            }
        }
        true
    }

    /// Get current test result
    pub fn result(&self) -> HealthTestResult {
        if self.result.load(Ordering::Acquire) {
            HealthTestResult::Pass
        } else {
            HealthTestResult::Fail
        }
    }

    /// Reset test state
    pub fn reset(&mut self) {
        self.window_position = 0;
        self.max_count = 0;
        self.result.store(true, Ordering::Release);
    }

    /// Get cutoff value
    pub fn cutoff(&self) -> u32 {
        self.cutoff
    }
}

/// Restart Test (NIST SP 800-90B Section 4.4.4)
/// On power-on or restart, verify the entropy source produces
/// different outputs across independent restarts.
/// Collects two sets of samples and checks they are not
/// identical, indicating the source is not deterministic.
pub struct RestartTest {
    /// Number of samples per restart
    num_samples: usize,
    /// First restart samples
    first_samples: Option<Vec<u8>>,
    /// Second restart samples
    second_samples: Option<Vec<u8>>,
    /// Test result
    result: AtomicBool,
}

impl RestartTest {
    /// Create new restart test
    pub fn new(num_samples: usize) -> Self {
        RestartTest {
            num_samples,
            first_samples: None,
            second_samples: None,
            result: AtomicBool::new(true),
        }
    }

    /// Create with default sample count
    pub fn with_defaults() -> Self {
        Self::new(RESTART_TEST_SAMPLES)
    }

    /// Collect first restart sample set
    pub fn collect_first_restart(&mut self, qrng: &dyn QrngProvider) -> Result<(), HealthTestError> {
        match qrng.generate(self.num_samples) {
            Ok(data) => {
                self.first_samples = Some(data);
                Ok(())
            }
            Err(_) => Err(HealthTestError::QrngError),
        }
    }

    /// Collect second restart sample set and compare
    /// The restart test passes if the two sets are not identical.
    /// If they are identical, the source is deterministic and fails.
    pub fn collect_second_and_compare(
        &mut self,
        qrng: &dyn QrngProvider,
    ) -> HealthTestResult {
        match qrng.generate(self.num_samples) {
            Ok(data) => {
                self.second_samples = Some(data);
                self.compare_sets()
            }
            Err(_) => {
                self.result.store(false, Ordering::Release);
                HealthTestResult::Fail
            }
        }
    }

    /// Compare the two sample sets
    fn compare_sets(&mut self) -> HealthTestResult {
        match (&self.first_samples, &self.second_samples) {
            (Some(first), Some(second)) => {
                if first.len() != second.len() {
                    self.result.store(true, Ordering::Release);
                    return HealthTestResult::Pass;
                }

                let identical = first.iter().zip(second.iter()).all(|(a, b)| a == b);

                if identical {
                    self.result.store(false, Ordering::Release);
                    HealthTestResult::Fail
                } else {
                    self.result.store(true, Ordering::Release);
                    HealthTestResult::Pass
                }
            }
            _ => HealthTestResult::InsufficientData,
        }
    }

    /// Get current test result
    pub fn result(&self) -> HealthTestResult {
        if self.result.load(Ordering::Acquire) {
            HealthTestResult::Pass
        } else {
            HealthTestResult::Fail
        }
    }
}

/// Continuous health test state for runtime monitoring
/// Maintains state between calls for the mandatory continuous
/// health tests that must be applied to every sample from
/// the entropy source.
pub struct ContinuousHealthTest {
    /// Repetition count test
    rep_test: RepetitionCountTest,
    /// Adaptive proportion test
    adapt_test: AdaptiveProportionTest,
    /// Total samples tested
    total_samples: AtomicU64,
    /// Total failures detected
    total_failures: AtomicU64,
    /// Is initialized
    initialized: AtomicBool,
}

impl ContinuousHealthTest {
    /// Create new continuous health test
    pub fn new() -> Self {
        ContinuousHealthTest {
            rep_test: RepetitionCountTest::with_defaults(),
            adapt_test: AdaptiveProportionTest::with_defaults(),
            total_samples: AtomicU64::new(0),
            total_failures: AtomicU64::new(0),
            initialized: AtomicBool::new(false),
        }
    }

    /// Initialize continuous health test
    pub fn init(&mut self) {
        self.rep_test.reset();
        self.adapt_test.reset();
        self.total_samples.store(0, Ordering::Release);
        self.total_failures.store(0, Ordering::Release);
        self.initialized.store(true, Ordering::Release);
    }

    /// Test a single sample from the entropy source
    /// Must be called for every sample from the QRNG.
    /// Returns false if any health test fails.
    pub fn test_sample(&mut self, sample: u8) -> bool {
        if !self.initialized.load(Ordering::Acquire) {
            return true;
        }

        self.total_samples.fetch_add(1, Ordering::Relaxed);

        let rep_ok = self.rep_test.test_sample(sample);
        let adapt_ok = self.adapt_test.test_sample(sample);

        if !rep_ok || !adapt_ok {
            self.total_failures.fetch_add(1, Ordering::Relaxed);
            return false;
        }

        true
    }

    /// Test a batch of samples
    pub fn test_batch(&mut self, samples: &[u8]) -> bool {
        for &sample in samples.iter() {
            if !self.test_sample(sample) {
                return false;
            }
        }
        true
    }

    /// Get statistics
    pub fn stats(&self) -> (u64, u64) {
        (
            self.total_samples.load(Ordering::Acquire),
            self.total_failures.load(Ordering::Acquire),
        )
    }

    /// Get individual test results
    pub fn test_results(&self) -> (HealthTestResult, HealthTestResult) {
        (self.rep_test.result(), self.adapt_test.result())
    }
}

/// Integer square root (Newton's method)
/// No floating point required for no_std.
fn isqrt(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

/// Execute all startup health tests (NIST SP 800-90B)
/// Must be called during boot or after QRNG restart.
/// All tests must pass before the QRNG output can be used.
pub fn qrng_health_test(qrng: &dyn QrngProvider) -> HealthTestReport {
    let mut report = HealthTestReport::new();

    let sample_count = 4096;
    let samples = match qrng.generate(sample_count) {
        Ok(data) => data,
        Err(_) => {
            report.overall = HealthTestResult::Fail;
            return report;
        }
    };

    report.samples_tested = samples.len();

    let mut rep_test = RepetitionCountTest::with_defaults();
    let mut rep_pass = true;
    for &sample in samples.iter() {
        if !rep_test.test_sample(sample) {
            rep_pass = false;
            break;
        }
    }
    report.repetition_count_test = if rep_pass {
        HealthTestResult::Pass
    } else {
        HealthTestResult::Fail
    };

    let mut adapt_test = AdaptiveProportionTest::with_defaults();
    let adapt_pass = adapt_test.test_batch(&samples);
    report.adaptive_proportion_test = if adapt_pass {
        HealthTestResult::Pass
    } else {
        HealthTestResult::Fail
    };

    let mut restart_test = RestartTest::with_defaults();
    if restart_test.collect_first_restart(qrng).is_ok() {
        report.restart_test = restart_test.collect_second_and_compare(qrng);
    } else {
        report.restart_test = HealthTestResult::Fail;
    }

    if report.all_passed() {
        report.overall = HealthTestResult::Pass;
    } else {
        report.overall = HealthTestResult::Fail;
    }

    report
}

/// Create and initialize continuous health test for runtime monitoring
/// This test must be applied to every sample from the QRNG
/// during normal operation.
pub fn qrng_continuous_test() -> ContinuousHealthTest {
    let mut test = ContinuousHealthTest::new();
    test.init();
    test
}

/// Global continuous health test state
static mut CONTINUOUS_HEALTH_TEST: Option<ContinuousHealthTest> = None;

/// Initialize the global continuous health test
pub fn init_continuous_health_test() {
    // SAFETY: single-threaded init during boot
    unsafe {
        let mut test = ContinuousHealthTest::new();
        test.init();
        CONTINUOUS_HEALTH_TEST = Some(test);
    }
}

/// Test a sample against the global continuous health test
pub fn continuous_test_sample(sample: u8) -> bool {
    // SAFETY: read-only access after init; write is atomic
    unsafe {
        if let Some(ref mut test) = CONTINUOUS_HEALTH_TEST {
            test.test_sample(sample)
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repetition_count_test_pass() {
        let mut test = RepetitionCountTest::with_defaults();
        let varied: [u8; 100] = core::array::from_fn(|i| (i % 256) as u8);
        for &byte in varied.iter() {
            assert!(test.test_sample(byte));
        }
        assert_eq!(test.result(), HealthTestResult::Pass);
    }

    #[test]
    fn test_repetition_count_test_fail() {
        let mut test = RepetitionCountTest::with_defaults();
        for _ in 0..35 {
            assert!(test.test_sample(0x42));
        }
        assert_eq!(test.result(), HealthTestResult::Pass);
        for _ in 0..5 {
            test.test_sample(0x42);
        }
        assert_eq!(test.result(), HealthTestResult::Fail);
    }

    #[test]
    fn test_repetition_count_reset() {
        let mut test = RepetitionCountTest::with_defaults();
        for _ in 0..40 {
            test.test_sample(0x42);
        }
        test.reset();
        assert_eq!(test.result(), HealthTestResult::Pass);
    }

    #[test]
    fn test_adaptive_proportion_pass() {
        let mut test = AdaptiveProportionTest::with_defaults();
        let varied: Vec<u8> = (0..512).map(|i| (i % 256) as u8).collect();
        assert!(test.test_batch(&varied));
        assert_eq!(test.result(), HealthTestResult::Pass);
    }

    #[test]
    fn test_adaptive_proportion_cutoff() {
        let test = AdaptiveProportionTest::with_defaults();
        assert!(test.cutoff() > 0);
    }

    #[test]
    fn test_health_test_report_new() {
        let report = HealthTestReport::new();
        assert_eq!(report.restart_test, HealthTestResult::NotRun);
        assert_eq!(report.overall, HealthTestResult::NotRun);
    }

    #[test]
    fn test_continuous_health_test() {
        let mut test = ContinuousHealthTest::new();
        test.init();
        for i in 0u8..100 {
            assert!(test.test_sample(i));
        }
        let (samples, failures) = test.stats();
        assert_eq!(samples, 100);
        assert_eq!(failures, 0);
    }

    #[test]
    fn test_restart_test() {
        let mut test = RestartTest::with_defaults();
        assert_eq!(test.num_samples, RESTART_TEST_SAMPLES);
    }
}
