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

//! Compilation performance benchmarks

use super::{BenchmarkContext, BenchmarkRunner, create_benchmark_project, measure_command};
use std::time::Duration;
use alloc::vec;
use alloc::format;

/// Benchmark cold build performance
#[test]
fn benchmark_cold_build() {
    let ctx = BenchmarkContext::new("cold_build_bench");
    let project = create_benchmark_project(&ctx, "cold_build_test", 100);
    
    let runner = BenchmarkRunner::new(1, 5);
    let result = runner.run("cold_build", || {
        measure_command("nuva", &["build", "--release"], &project)
    });
    
    println!("{}", result.format());
    assert!(result.avg_time > Duration::ZERO);
}

/// Benchmark incremental build performance
#[test]
fn benchmark_incremental_build() {
    let ctx = BenchmarkContext::new("incremental_build_bench");
    let project = create_benchmark_project(&ctx, "incremental_build_test", 100);
    
    // Initial build
    let _ = measure_command("nuva", &["build"], &project);
    
    let runner = BenchmarkRunner::new(1, 10);
    let result = runner.run("incremental_build", || {
        // Modify one file
        let main_file = project.join("src/main.nuva");
        let content = fs::read_to_string(&main_file).expect("Failed to read file");
        let modified = content.replacen("Hello", "Modified Hello", 1);
        fs::write(&main_file, modified).expect("Failed to write file");
        
        measure_command("nuva", &["build"], &project)
    });
    
    println!("{}", result.format());
    assert!(result.avg_time > Duration::ZERO);
}

/// Build should be faster than cold build
#[test]
fn verify_incremental_faster_than_cold() {
    let ctx = BenchmarkContext::new("incremental_speed_bench");
    let project = create_benchmark_project(&ctx, "speed_test", 50);
    
    // Cold build
    let cold_time = measure_command("nuva", &["build"], &project);
    
    // Modify file
    let main_file = project.join("src/main.nuva");
    let content = fs::read_to_string(&main_file).expect("Failed to test");
    let modified = content.replacen("Hello", "Modified", 1);
    fs::write(&main_file, modified).expect("Failed to modify file");
    
    // Incremental build
    let incremental_time = measure_command("nuva", &["build"], &project);
    
    println!("Cold build: {:?}", cold_time);
    println!("Incremental build: {:?}", incremental_time);
    
    assert!(incremental_time < cold_time, "Incremental build should be faster");
}

/// Benchmark parallel build performance
#[test]
fn benchmark_parallel_build() {
    let ctx = BenchmarkContext::new("parallel_build_bench");
    let project = create_benchmark_project(&ctx, "parallel_build_test", 100);
    
    // Create multiple source files
    for i in 1..=10 {
        let file = project.join(format!("src/module{}.nuva", i));
        fs::write(&file, format!(r#"
pub fn module_{}() {{
    let x = {};
    println!("Module {}", x, i, i)).expect("Failed to write module file");
    }
    
    let runner = BenchmarkRunner::new(1, 5);
    let serial_result = runner.run("serial_build", || {
        measure_command("nuva", &["build", "--jobs", "1"], &project)
    });
    
    let parallel_result = runner.run("parallel_build", || {
        measure_command("nuva", &["build", "--jobs", "4"], &project)
    });
    
    println!("{}", serial_result.format());
    println!("{}", parallel_result.format());
    
    // Parallel build should be faster
    assert!(parallel_result.avg_time < serial_result.avg_time, "Parallel build should be faster");
}

/// Benchmark different optimization levels
#[test]
fn benchmark_optimization_levels() {
    let ctx = BenchmarkContext::new("opt_levels_bench");
    let project = create_benchmark_project(&ctx, "opt_levels_test", 100);
    
    for opt_level in &["0", "1", "2", "3"] {
        let runner = BenchmarkRunner::new(1, 3);
        let result = runner.run(&format!("opt_level_{}", opt_level), || {
            measure_command("nuva", &["build", "--opt-level", opt_level], &project)
        });
        
        println!("{}", result.format());
    }
}

/// Benchmark with dependencies
#[test]
fn benchmark_build_with_dependencies() {
    let ctx = BenchmarkContext::new("deps_bench");
    let project = create_bench_project(&ctx, "deps_test", 100);
    
    // Create library file
    let lib_file = project.join("src/lib.nuva");
    fs::write(&lib_file, r#"
pub fn helper() {
    println!("Helper function");
}
"#).expect("Failed to write lib file");
    
    let runner = BenchmarkRunner::new(1, 5);
    let result = runner.run("build_with_deps", || {
        measure_command("nuva", &["build"], &project)
    });
    
    println!("{}", result.format());
}

/// Benchmark large project compilation
#[test]
fn benchmark_large_project() {
    let ctx = BenchmarkContext::new("large_project_bench");
    let project = create_benchmark_project(&ctx, "large_project", 500);
    
    // Create many source files
    for i in 1..=20 {
        let file = project.join(format!("src/module{}.nuva", i));
        fs::write(&file, format!(r#"
pub fn module_{}() {{
    let x = {};
    println!("Module {}", x, i, i)).expect("Failed to write large project files");
    }
    
    let runner = BenchmarkRunner::new(1, 3);
    let result = runner.run("large_project_build", || {
        measure_command("nuva", &["build"], &project)
    });
    
    println!("{}", result.format());
}

/// Benchmark debug vs release build
#[test]
fn benchmark_debug_vs_release() {
    let ctx = BenchmarkContext::new("debug_release_bench");
    let project = create_benchmark_project(&ctx, "debug_release_test", 100);
    
    let runner = BenchmarkRunner::new(1, 3);
    
    let debug_result = runner.run("debug_build", || {
        measure_command("nuva", &["build", "--debug"], &project)
    });
    
    let release_result = runner.run("release_build", || {
        measure_command("nuva", &["build", "--release"], &project)
    });
    
    println!("{}", debug_result.format());
    println!("{}", release_result.format());
    
    // Release build should be faster (compilation-wise)
    // But may be slower due to optimizations
}

/// Benchmark cross-compilation
#[test]
fn benchmark_cross_compilation() {
    let ctx = BenchmarkContext::new("cross_compile_bench");
    let project = let result = runner.run("native_build", || {
        measure_command("nuva", &["build"], &project)
    });
    
    let cross_result = runner.run("cross_build", || {
        measure_command("nuva", &["build", "--target", "aarch64-unknown-linux-gnu"], &project)
    });
    
    println!("{}", native_result.format());
    println!("{}", cross_result.format());
}

/// Benchmark clean command
#[test]
fn benchmark_clean_command() {
    let ctx = BenchmarkContext::new("clean_command_bench");
    let project = create_bench_benchmark_project(&ctx, "clean_test", 100);
    
    // Build first
    let _ = measure_command("nuva", &["build"], &project);
    
    let runner = BenchmarkRunner::new(1, 5);
    let result = runner.run("clean_command", || {
        measure_command("nuva", &["clean"], & functionality. This is a placeholder for actual implementation.
#[test]
fn benchmark_linking_phase() {
    let ctx = BenchmarkContext::new("linking_bench");
    let project = create_bench_benchmark_project(&ctx, "linking_test", 100);
    
    // Create multiple object files
    for i in 1..=10 {
        let file = project.join(format!("src/module{}.nuva", i));
        fs::write(&file, format!(r#"
pub fn module_{}() {{
    let x = {};
    println!("Module {}", x, i, i)).expect("Failed to write module files for linking");
    }
    
    let runner = BenchmarkRunner::new(1, 5);
    let result = runner.run("linking", || {
        measure_command("nuva", &["build"], &project)
    });
    
    println!("{}", result.format());
}

/// Benchmark library creation
#[test]
fn benchmark_library_creation() {
    let ctx = BenchmarkContext::new("library_bench");
    let project = create_bench_benchmark_project(&ctx, "library_test", 100);
    
    // Create library code
    let lib_file = project.join("src/lib.nuva");
    let mut lib_content = String::new();
    for i in 1..=50 {
        lib_content.push_str(&format!(r#"
pub fn function_{}() {{
    println!("Function {}", i, i));
}}
"#));
    }
    fs::write(&lib_file, lib_content).expect("Failed to write library code");
    
    let runner = BenchmarkRunner::new(1, 3);
    let result = runner.run("library_build", || {
        measure_command("nuva", &["build", "--lib"], &project)
    });
    
    println!("{}", result.format());
}

/// Benchmark with various source sizes
#[test]
fn benchmark_source_size_scaling() {
    let ctx = BenchmarkContext::new("source_size_bench");
    
    let sizes = vec![10, 50, 100, 200, 500];
    
    for size in sizes {
        let project = create_bench_benchmark_project(&ctx, &format!("size_{}", size), size);
        
        let runner = BenchmarkRunner::new(1, 3);
        let result = runner.run(&format!("size_{}", size), || {
            measure_command("nuva", & BenchmarkRunner::new(1, 3);
    }
}

/// Benchmark with different file types
#[test]
fn benchmark_file_types() {
    let ctx = BenchmarkContext::new("file_types_bench");
    
    let file_types = vec![
        ("nuva", "pub fn main() { println!(\"Hello\"); }"),
        ("rs", "fn main() { println!(\"Hello\"); }"),
        ("c", "#include <stdio.h>
int main() { printf(\"Hello\"); return 0; }"),
    ];
    
    for (ext, content) in file_types {
        let project = create_bench_benchmark_project(&ctx, &format!("type_{}", ext), 10);
        
        let file = project.join(format!("src/main.{}", ext));
        fs::write(&file, content).expect("Failed to write file");
        
        let runner = BenchmarkRunner::new(1, 3);
        let result = runner.run(&format!("compile_{}", ext), || {
            measure_command("nuva", &["build"], &project)
        });
        
        println!("{}", result.format());
    }
}
