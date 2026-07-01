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

//! Benchmark tests for Nuva SDK
/*!*/
//! This module contains performance benchmarks for various components of the Nuva SDK,
//! including compilation speed, LSP performance, and package manager performance.

pub mod compile_bench;
pub mod lsp_bench;
pub mod package_bench;
pub mod memory_bench;

use std::time::{Duration, Instant};
use std::path::PathBuf;
use std::fs;

/// Benchmark result
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Benchmark name
    pub name: String,
    /// Number of iterations
    pub iterations: u64,
    /// Total duration
    pub duration: Duration,
    /// Average time per iteration
    pub avg_time: Duration,
    /// Min time per iteration
    pub min_time: Duration,
    /// Max time per iteration
    pub max_time: Duration,
}

impl BenchmarkResult {
    /// Create a new benchmark result
    pub fn new(name: String, iterations: u64, durations: Vec<Duration>) -> Self {
        let total_duration: Duration = durations.iter().sum();
        let avg_time = total_duration / iterations as u32;
        let min_time = *durations.iter().min().unwrap_or(&Duration::ZERO);
        let max_time = *durations.iter().max().unwrap_or(&Duration::ZERO);
        
        Self {
            name,
            iterations,
            duration: total_duration,
            avg_time,
            min_time,
            max_time,
        }
    }
    
    /// Format the result for display
    pub fn format(&self) -> String {
        format!(
            "{}: {} iterations, total time: {:?}, avg: {:?}, min: {:?}, max: {:?}",
            self.name,
            self.iterations,
            self.duration,
            self.avg_time,
            self.min_time,
            self.max_time
        )
    }
}

/// Benchmark runner
pub struct BenchmarkRunner {
    /// Number of warmup iterations
    warmup_iterations: u64,
    /// Number of measured iterations
    measured_iterations: u64,
}

impl BenchmarkRunner {
    /// Create a new benchmark runner
    pub fn new(warmup_iterations: u64, measured_iterations: u64) -> Self {
        Self {
            warmup_iterations,
            measured_iterations,
        }
    }
    
    /// Run a benchmark
    pub fn run<F>(&self, name: &str, mut f: F) -> BenchmarkResult
    where
        F: FnMut() -> Duration,
    {
        println!("Running benchmark: {}", name);
        
        // Warmup phase
        for _ in 0..self.warmup_iterations {
            let _ = f();
        }
        
        // Measurement phase
        let mut durations = Vec::with_capacity(self.measured_iterations as usize);
        for _ in 0..self.measured_iterations {
            let duration = f();
            durations.push(duration);
        }
        
        BenchmarkResult::new(name.to_string(), self.measured_iterations, durations)
    }
}

impl Default for BenchmarkRunner {
    fn default() -> Self {
        Self::new(3, 10)
    }
}

/// Benchmark context
pub struct BenchmarkContext {
    /// Workspace directory
    pub workspace: PathBuf,
    /// Temporary directory for benchmark artifacts
    pub temp_dir: PathBuf,
}

impl BenchmarkContext {
    /// Create a new benchmark context
    pub fn new(benchmark_name: &str) -> Self {
        let bench_dir = std::env::temp_dir().join(format!("nuva_bench_{}", benchmark_name));
        fs::create_dir_all(&bench_dir).expect("Failed to create benchmark directory");
        
        Self {
            workspace: bench_dir.join("workspace"),
            temp_dir: bench_dir,
        }
    }
    
    /// Clean up benchmark context
    pub fn cleanup(&self) {
        if self.temp_dir.exists() {
            let _ = fs::remove_dir_all(&self.temp_dir);
        }
    }
}

impl Drop for BenchmarkContext {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// Create a benchmark project
pub fn create_benchmark_project(ctx: &BenchmarkContext, name: &str, source_size: usize) -> PathBuf {
    let project_dir = ctx.workspace.join(name);
    fs::create_dir_all(&project_dir).expect("Failed to create project directory");
    
    // Create src directory
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir).expect("Failed to create src directory");
    
    // Create a main file with specified size
    let mut source = String::from("fn main() {
");
    for i in 0..source_size {
        source.push_str(&format!("    let x{} = {};
", i, i));
    }
    source.push_str("    println!(\"Hello from benchmark!\");
");
    source.push_str("}
");
    
    let main_file = src_dir.join("main.nuva");
    fs::write(&main_file, source).expect("Failed to write main file");
    
    // Create Nuva.toml
    let manifest = project_dir.join("Nuva.toml");
    fs::write(&manifest, r#"[package]
name = "benchmark_project"
version = "0.1.0"
edition = "2024"

[dependencies]
"#).expect("Failed to write Nuva.toml");
    
    project_dir
}

/// Run a command and measure execution time
pub fn measure_command(cmd: &str, args: &[&str], cwd: &PathBuf) -> Duration {
    let start = Instant::now();
    
    let output = std::process::Command::new(cmd)
        .args(args)
        .current_dir(cwd)
        .output();
    
    let _ = output;
    
    start.elapsed()
}

#[cfg(test)]
mod tests {
    use super::*;
use alloc::format;
use alloc::vec::Vec;
    
    #[test]
    fn test_benchmark_runner() {
        let runner = BenchmarkRunner::new(2, 5);
        
        let result = runner.run("test_benchmark", || {
            std::thread::sleep(Duration::from_millis(10));
            Duration::from_millis(10)
        });
        
        assert_eq!(result.iterations, 5);
        assert!(result.avg_time >= Duration::from_millis(10));
    }
    
    #[test]
    fn test_benchmark_context() {
        let ctx = BenchmarkContext::new("context_test");
        assert!(ctx.workspace.exists());
        assert!(ctx.temp_dir.exists());
    }
    
    #[test]
    fn test_benchmark_project_creation() {
        let ctx = BenchmarkContext::new("project_test");
        let project = create_benchmark_project(&ctx, "test_project", 10);
        assert!(project.exists());
        assert!(project.join("src/main.nuva").exists());
        assert!(project.join("Nuva.toml").exists());
    }
}
