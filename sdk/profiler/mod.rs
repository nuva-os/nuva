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

// ! Profiler module with CPU, memory, I/O, and lock profiling

pub mod cpu;
pub mod memory;
pub mod sampler;
pub mod flamegraph;
pub mod io;
pub mod lock;

use crate::error::SdkError;

/// Profiler orchestrator
pub struct Profiler {
    /// CPU profiler
    cpu: cpu::CpuProfiler,
    /// Memory profiler
    memory: memory::MemProfiler,
    /// I/O profiler
    io: io::IoProfiler,
    /// Lock profiler
    lock: lock::LockProfiler,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            cpu: cpu::CpuProfiler::new(),
            memory: memory::MemProfiler::new(),
            io: io::IoProfiler::new(),
            lock: lock::LockProfiler::new(),
        }
    }

    /// Start CPU profiling
    pub fn start_cpu(&mut self, pid: Option<u32>) -> Result<(), SdkError> {
        self.cpu.start(pid)
    }

    /// Stop CPU profiling
    pub fn stop_cpu(&mut self) -> Result<cpu::CpuProfile, SdkError> {
        self.cpu.stop()
    }

    /// Start memory profiling
    pub fn start_memory(&mut self, pid: Option<u32>) -> Result<(), SdkError> {
        self.memory.start(pid)
    }

    /// Stop memory profiling
    pub fn stop_memory(&mut self) -> Result<memory::MemProfile, SdkError> {
        self.memory.stop()
    }

    /// Start I/O profiling
    pub fn start_io(&mut self, pid: Option<u32>) -> Result<(), SdkError> {
        self.io.start(pid)
    }

    /// Stop I/O profiling
    pub fn stop_io(&mut self) -> Result<io::IoProfile, SdkError> {
        self.io.stop()
    }

    /// Start lock profiling
    pub fn start_lock(&mut self, pid: Option<u32>) -> Result<(), SdkError> {
        self.lock.start(pid)
    }

    /// Stop lock profiling
    pub fn stop_lock(&mut self) -> Result<lock::LockProfile, SdkError> {
        self.lock.stop()
    }

    /// Generate flamegraph SVG
    pub fn generate_flamegraph(&self, profile: &cpu::CpuProfile) -> Result<String, SdkError> {
        flamegraph::generate(profile)
    }

    /// Generate flamegraph with custom width
    pub fn generate_flamegraph_with_width(&self, profile: &cpu::CpuProfile, width: usize) -> Result<String, SdkError> {
        flamegraph::generate_with_width(profile, width)
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}
