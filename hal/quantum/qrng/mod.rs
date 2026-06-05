/*
 * Nuva OS - Hal - Quantum - Qrng - Mod
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
 * Quantum Random Number Generator (QRNG) Provider
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module provides hardware abstraction for quantum random
 * number generators, enabling true randomness from quantum sources.
 */

use core::fmt;
use alloc::vec::Vec;
use alloc::string::String;

pub mod hardware;
pub mod health_test;

/// QRNG Provider trait - Hardware abstraction for quantum RNG
/// Implementations can use:
/// - Hardware QRNG devices (quantum entropy sources)
/// - Quantum entropy pools
/// - Hybrid approaches (quantum + classical fallback)
pub trait QrngProvider: Send + Sync {
    /// Generate random bytes
    /// @param len: Number of bytes to generate
    /// @return: Random bytes
    fn generate(&self, len: usize) -> Result<Vec<u8>, QrngError>;

    /// Generate random u32
    /// @return: Random u32
    fn generate_u32(&self) -> Result<u32, QrngError>;

    /// Generate random u64
    /// @return: Random u64
    fn generate_u64(&self) -> Result<u64, QrngError>;

    /// Generate random in range [0, max)
    /// @param max: Upper bound (exclusive)
    /// @return: Random value in range
    fn generate_range(&self, max: u64) -> Result<u64, QrngError>;

    /// Verify randomness quality
    /// @param data: Data to verify
    /// @return: Quality metrics
    fn verify_randomness(&self, data: &[u8]) -> Result<RandomnessQuality, QrngError>;

    /// Get entropy level
    /// @return: Entropy level (0-100%)
    fn entropy_level(&self) -> u8;

    /// Get provider name
    fn name(&self) -> &str;

    /// Check if quantum source is available
    fn is_quantum_source_available(&self) -> bool;
}

/// Randomness quality metrics
#[derive(Debug, Clone)]
pub struct RandomnessQuality {
    /// Monobit test result (NIST SP 800-22)
    pub monobit_test: f64,

    /// Frequency block test result
    pub frequency_block_test: f64,

    /// Runs test result
    pub runs_test: f64,

    /// Longest run test result
    pub longest_run_test: f64,

    /// Serial test result
    pub serial_test: f64,

    /// Approximate entropy test result
    pub approximate_entropy_test: f64,

    /// Cumulative sum test result
    pub cumulative_sum_test: f64,

    /// Overall quality score (0-100)
    pub overall_score: u8,

    /// Is the data sufficiently random?
    pub is_random: bool,
}

impl RandomnessQuality {
    /// Check if all tests pass
    pub fn all_tests_pass(&self) -> bool {
        // NIST tests pass if p-value > 0.01
        const THRESHOLD: f64 = 0.01;
        self.monobit_test > THRESHOLD
            && self.frequency_block_test > THRESHOLD
            && self.runs_test > THRESHOLD
            && self.longest_run_test > THRESHOLD
            && self.serial_test > THRESHOLD
            && self.approximate_entropy_test > THRESHOLD
            && self.cumulative_sum_test > THRESHOLD
    }
}

/// QRNG error type
#[derive(Debug, Clone)]
pub enum QrngError {
    /// Quantum source not available
    QuantumSourceNotAvailable,

    /// Entropy pool exhausted
    EntropyExhausted,

    /// Hardware error
    HardwareError(String),

    /// Out of memory
    OutOfMemory,

    /// Invalid request
    InvalidRequest,

    /// Quality check failed
    QualityCheckFailed,

    /// Timeout
    Timeout,
}

impl fmt::Display for QrngError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QuantumSourceNotAvailable => write!(f, "Quantum source not available"),
            Self::EntropyExhausted => write!(f, "Entropy pool exhausted"),
            Self::HardwareError(msg) => write!(f, "Hardware error: {}", msg),
            Self::OutOfMemory => write!(f, "Out of memory"),
            Self::InvalidRequest => write!(f, "Invalid request"),
            Self::QualityCheckFailed => write!(f, "Quality check failed"),
            Self::Timeout => write!(f, "Timeout"),
        }
    }
}

/// QRNG statistics
#[derive(Debug, Clone)]
pub struct QrngStats {
    /// Total bytes generated
    pub total_bytes: u64,

    /// Number of requests
    pub request_count: u64,

    /// Average generation time (ns)
    pub avg_generation_time_ns: u64,

    /// Current entropy level (0-100%)
    pub entropy_level: u8,

    /// Number of quality checks performed
    pub quality_checks: u64,

    /// Number of quality check failures
    pub quality_failures: u64,
}
