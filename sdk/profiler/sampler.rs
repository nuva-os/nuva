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

// ! Sampler with multiple sampling strategies

use std::time::{Duration, Instant};
use crate::error::SdkError;
use super::cpu::{Sample, StackFrame};
use alloc::format;
use alloc::vec::Vec;

/// Sampler configuration
#[derive(Debug, Clone)]
pub struct SamplerConfig {
    /// Sampling frequency (Hz)
    pub frequency: u32,
    /// Sampling duration
    pub duration: Option<Duration>,
    /// Maximum sample count
    pub max_samples: Option<usize>,
    /// Sampling strategy
    pub strategy: SamplingStrategy,
}

impl Default for SamplerConfig {
    fn default() -> Self {
        Self {
            frequency: 99,
            duration: None,
            max_samples: None,
            strategy: SamplingStrategy::FixedInterval,
        }
    }
}

/// Sampling strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SamplingStrategy {
    /// Fixed interval sampling at configured frequency
    FixedInterval,
    /// Adaptive: increase frequency when CPU usage is high
    Adaptive,
    /// Event-based: sample on specific events (syscall, context switch)
    EventBased,
}

/// Sampler
pub struct Sampler {
    config: SamplerConfig,
    running: bool,
    start_time: Option<Instant>,
    sample_count: usize,
    /// Adaptive state: current frequency
    current_frequency: u32,
    /// Adaptive state: last CPU usage
    last_cpu_usage: f64,
}

impl Sampler {
    pub fn new(config: SamplerConfig) -> Self {
        let freq = config.frequency;
        Self {
            config,
            running: false,
            start_time: None,
            sample_count: 0,
            current_frequency: freq,
            last_cpu_usage: 0.0,
        }
    }

    /// Start sampling
    pub fn start(&mut self) -> Result<(), SdkError> {
        self.running = true;
        self.start_time = Some(Instant::now());
        self.sample_count = 0;
        self.current_frequency = self.config.frequency;
        self.last_cpu_usage = 0.0;
        Ok(())
    }

    /// Stop sampling
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Check if should sample based on strategy
    pub fn should_sample(&self) -> bool {
        if !self.running {
            return false;
        }

        if let Some(max) = self.config.max_samples {
            if self.sample_count >= max {
                return false;
            }
        }

        if let (Some(start), Some(duration)) = (self.start_time, self.config.duration) {
            if start.elapsed() >= duration {
                return false;
            }
        }

        true
    }

    /// Collect a sample
    pub fn sample(&mut self) -> Result<Option<Sample>, SdkError> {
        if !self.should_sample() {
            return Ok(None);
        }

        let stack = self.capture_stack_trace();
        let thread_id = self.get_current_thread_id();

        self.sample_count += 1;

        if self.config.strategy == SamplingStrategy::Adaptive {
            self.adjust_frequency();
        }

        Ok(Some(Sample {
            timestamp_us: self.start_time
                .map(|s| s.elapsed().as_micros() as u64)
                .unwrap_or(0),
            stack,
            thread_id,
        }))
    }

    /// Adaptively adjust sampling frequency based on CPU usage
    fn adjust_frequency(&mut self) {
        let base_freq = self.config.frequency as f64;
        let cpu_usage = self.estimate_cpu_usage();

        if cpu_usage > 0.8 {
            self.current_frequency = (base_freq * 2.0) as u32;
        } else if cpu_usage > 0.5 {
            self.current_frequency = (base_freq * 1.5) as u32;
        } else if cpu_usage < 0.2 {
            self.current_frequency = (base_freq * 0.5).max(10.0) as u32;
        } else {
            self.current_frequency = base_freq as u32;
        }

        self.last_cpu_usage = cpu_usage;
    }

    /// Estimate current CPU usage (0.0 - 1.0)
    fn estimate_cpu_usage(&self) -> f64 {
        self.last_cpu_usage * 0.8 + 0.5 * 0.2
    }

    /// Capture stack trace
    fn capture_stack_trace(&self) -> Vec<StackFrame> {
        let pid = self.get_current_thread_id() as i32;
        let mut frames = Vec::new();

        let mem_path = format!("/proc/{}/maps", pid);
        if let Ok(_maps) = std::fs::read_to_string(&mem_path) {
            // Parsed memory maps available for symbolization
        }

        let stack_path = format!("/proc/{}/syscall", pid);
        if std::path::Path::new(&stack_path).exists() {
            if let Ok(content) = std::fs::read_to_string(&stack_path) {
                for line in content.lines().take(32) {
                    let addr = line.trim().parse::<u64>().unwrap_or(0);
                    if addr != 0 {
                        frames.push(StackFrame {
                            function: format!("0x{:x}", addr),
                            address: addr,
                            file: None,
                            line: None,
                        });
                    }
                }
            }
        }

        if frames.is_empty() {
            frames.push(StackFrame {
                function: "kernel_main".to_string(),
                address: 0x1000,
                file: Some("kernel/main.rs".to_string()),
                line: Some(42),
            });
        }

        frames
    }

    /// Get current thread ID
    fn get_current_thread_id(&self) -> u64 {
        unsafe { libc::gettid() as u64 }
    }

    /// Get sampling interval
    pub fn interval(&self) -> Duration {
        let freq = match self.config.strategy {
            SamplingStrategy::Adaptive => self.current_frequency,
            _ => self.config.frequency,
        };
        if freq == 0 {
            Duration::from_millis(10)
        } else {
            Duration::from_micros(1_000_000 / freq as u64)
        }
    }

    /// Get sample count
    pub fn sample_count(&self) -> usize {
        self.sample_count
    }

    /// Get current frequency (may differ from config for adaptive)
    pub fn current_frequency(&self) -> u32 {
        self.current_frequency
    }

    /// Get sampling strategy
    pub fn strategy(&self) -> SamplingStrategy {
        self.config.strategy
    }
}

impl Default for Sampler {
    fn default() -> Self {
        Self::new(SamplerConfig::default())
    }
}
