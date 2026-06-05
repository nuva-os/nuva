/*
 * Nuva OS - Kernel - Diag - NuvaDiagnostic
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
 * Nuva OS - Kernel - Nuva Diagnostic Interface
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Nuva native diagnostic interface replacing /proc and /sys filesystems.
 * Migration note: Previously used Linux-style /proc and /sys procfs for
 * system diagnostics. Now uses NuvaDiagnostic trait for native queries.
 */

use crate::types::{NuvaError, NuvaDiagTopic};

/// Diagnostic information container
#[derive(Debug, Clone)]
pub struct NuvaDiagInfo {
    pub topic: NuvaDiagTopic,
    pub data: [u64; 8],
    pub text_len: usize,
    pub text_buf: [u8; 256],
}

impl NuvaDiagInfo {
    pub fn new(topic: NuvaDiagTopic) -> Self {
        NuvaDiagInfo {
            topic,
            data: [0; 8],
            text_len: 0,
            text_buf: [0; 256],
        }
    }
}

/// Diagnostic statistics
#[derive(Debug, Clone, Default)]
pub struct NuvaDiagStats {
    pub queries_total: u64,
    pub queries_failed: u64,
    pub last_query_ns: u64,
}

/// Diagnostic parameter type
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum NuvaDiagParam {
    LogLevel        = 0,
    SamplingRate    = 1,
    RetentionPeriod = 2,
    MaxEntries      = 3,
}

/// Diagnostic value (parameter value union)
#[derive(Debug, Clone, Copy)]
pub union NuvaDiagValue {
    pub u32_val: u32,
    pub u64_val: u64,
    pub bool_val: bool,
}

/// Nuva native diagnostic trait.
/// Replaces /proc filesystem reads and /sys parameter configuration.
pub trait NuvaDiagnostic: Send + Sync {
    /// Query diagnostic information by topic.
    /// Migrated from: reading /proc/[topic] files
    fn query(&self, topic: NuvaDiagTopic) -> Result<NuvaDiagInfo, NuvaError>;

    /// Get diagnostic statistics.
    fn stats(&self) -> NuvaDiagStats;

    /// Configure a diagnostic parameter.
    /// Migrated from: writing to /sys/[param] files
    fn configure(&self, param: NuvaDiagParam, value: NuvaDiagValue) -> Result<(), NuvaError>;
}

/// Default NuvaDiagnostic implementation
pub struct DefaultDiagnostic {
    stats: core::sync::atomic::AtomicU64,
}

impl DefaultDiagnostic {
    pub const fn new() -> Self {
        DefaultDiagnostic {
            stats: core::sync::atomic::AtomicU64::new(0),
        }
    }
}

impl NuvaDiagnostic for DefaultDiagnostic {
    fn query(&self, topic: NuvaDiagTopic) -> Result<NuvaDiagInfo, NuvaError> {
        self.stats.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        Ok(NuvaDiagInfo::new(topic))
    }

    fn stats(&self) -> NuvaDiagStats {
        NuvaDiagStats::default()
    }

    fn configure(&self, _param: NuvaDiagParam, _value: NuvaDiagValue) -> Result<(), NuvaError> {
        Ok(())
    }
}

/// Global diagnostic instance
static DIAG: DefaultDiagnostic = DefaultDiagnostic::new();

/// Query diagnostic information (convenience function)
pub fn diag_query(topic: NuvaDiagTopic) -> Result<NuvaDiagInfo, NuvaError> {
    DIAG.query(topic)
}

/// Get diagnostic statistics (convenience function)
pub fn diag_stats() -> NuvaDiagStats {
    DIAG.stats()
}
