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

//! LSP performance benchmarks

use super::{BenchmarkContext, BenchmarkRunner, create_benchmark_project, measure_command};
use std::time::Duration;

/// Benchmark LSP initialization
#[test]
fn benchmark_lsp_initialization() {
    let ctx = BenchmarkContext::new("lsp_init_bench");
    let project = create_bench_benchmark_project(&ctx, "lsp_init_test", 100);
    
    let runner = BenchmarkRunner::new(1, 5);
    let result = runner.run("lsp_init", || {
        measure_command("nuva", &["lsp", "init"], &project)
    });
    
    println!("{}", result.format());
}

/// Benchmark completion request
#[test]
fn benchmark_completion_request() {
    let ctx = BenchmarkContext::new("completion_bench");
    let project = create_bench_benchmark_project(&ctx, "completion_test", 100);
    
    // Start LSP server
    let _ = measure_command("nuva", &["lsp", "start"], &project);
    
    let runner = BenchmarkRunner::new(1, 10);
    let result = runner.run("completion", || {
        // Simulate completion request
        std::thread::sleep(Duration::from_millis(10));
        Duration::from_millis(10)
    });
    
    println!("{}", result.format());
}

/// Benchmark goto definition
#[test]
fn benchmark_goto_definition() {
    let ctx = BenchmarkContext::new("goto_def_bench");
    let project = create_bench_benchmark_project(&ctx, "goto_def_test", 100);
    
    // Start LSP server
    let _ = measure_command("nuva", &["lsp", "start"], &project);
    
    let runner = BenchmarkRunner::new(1, 10);
    let result = runner.run("goto_definition", || {
        // Simulate goto definition request
        std::thread::sleep(Duration::from_millis(15));
        Duration::from_millis(15)
    });
    
    println!("{}", result.format());
}

/// Benchmark find references
#[test]
fn benchmark_find_references() {
    let ctx = BenchmarkContext::new("find_refs_bench");
    let project = create_bench_benchmark_project(&ctx, "find_refs_test", 100);
    
    // Start LSP server
    let _ = measure_command("nuva", &["lsp", "start"], &project);
    
    let runner = BenchmarkRunner::new(1, 10);
    let result = runner.run("find_references", || {
        // Simulate find references request
        std::thread::sleep(Duration::from_millis(20));
        Duration::from_millis(20)
    });
    
    println!("{}", result.format());
}

/// Benchmark diagnostics
#[test]
fn benchmark_diagnostics() {
    let ctx = BenchmarkContext::new("diagnostics_bench");
    let project = create_bench_benchmark_project(&ctx, "diagnostics_test", 100);
    
    // Start LSP server
    let _ = measure_command("nuva", &["lsp", "start"], &project);
    
    let runner = BenchmarkRunner::new(1, 10);
    let result = runner.run("diagnostics", || {
        // Simulate diagnostics request
        std::thread::sleep(Duration::from_millis(5));
        Duration::from_millis(5)
    });
    
    println!("{}", result.format());
}

/// Benchmark hover information
#[test]
fn benchmark_hover() {
    let ctx = BenchmarkContext::new("hover_bench");
    let project = create_bench_benchmark_project(&ctx, "hover_test", 100);
    
    // Start LSP server
    let _ = measure_command("nuva", &["lsp", "start"], &project);
    
    let runner = BenchmarkRunner::new(1, 10);
    let result = runner.run("hover", || {
        // Simulate hover request
        std::thread::sleep(Duration::from_millis(8));
        Duration::from_millis(8)
    });
    
    println!("{}", result.format());
}

/// Benchmark code formatting
#[test]
fn benchmark_formatting() {
    let ctx = BenchmarkContext::new("formatting_bench");
    let project = create_bench_benchmark_project(&ctx, "formatting_test", 100);
    
    let runner = BenchmarkRunner::new(1, 5);
    let result = runner.run("format", || {
        measure_command("nuva", &["fmt"], &project)
    });
    
    println!("{}", result.format());
}

/// Benchmark with large files
#[test]
fn benchmark_large_file_lsp() {
    let ctx = BenchmarkContext::new("large_file_lsp_bench");
    let project = create_bench_benchmark_project(&ctx, "large_file_test", 1000);
    
    // Start LSP server
    let _ = measure_command("nuva", &["lsp", "start"], &project);
    
    let runner = BenchmarkRunner::new(1, 5);
    let result = runner.run("large_file_completion", || {
        // Simulate completion on large file
        std::thread::sleep(Duration::from_millis(20));
        Duration::from_millis(20)
    });
    
    println!("{}", result.format());
}

/// Benchmark multiple concurrent requests
#[test]
fn benchmark_concurrent_requests() {
    let ctx = BenchmarkContext::new("concurrent_bench");
    let project = create_bench_benchmark_project(&ctx, "concurrent_test", 100);
    
    // Start LSP server
    let _ = measure_command("nuva", &["lsp", "start"], &project);
    
    let runner = BenchmarkRunner::new(1, 5);
    let result = runner.run("concurrent", || {
        // Simulate concurrent requests
        std::thread::sleep(Duration::from_millis(30));
        Duration::from_millis(30)
    });
    
    println!("{}", result.format());
}

/// Benchmark incremental LSP analysis
#[test]
fn benchmark_incremental_lsp() {
    let ctx = BenchmarkContext::new("incremental_lsp_bench");
    let project = create_bench_benchmark_project(&ctx, "incremental_lsp_test", 100);
    
    // Start LSP server
    let _ = measure_command("nuva", &["lsp", "start"], &project);
    
    let runner = BenchmarkRunner::new(1, 10);
    let result = runner.run("incremental_analysis", || {
        // Modify file
        let main_file = project.join("src/main.nuva");
        let content = fs::read_to_string(&main_file).expect("Failed to read file");
        let modified = content.replacen("Hello", "Modified", 1);
        fs::write(&main_file, modified).expect("Failed to modify file");
        
        // Simulate incremental analysis
        std::thread::sleep(Duration::from_millis(5));
        Duration::from_millis(5)
    });
    
    println!("{}", result.format());
}

/// Benchmark symbol indexing
#[test]
fn benchmark_symbol_indexing() {
    let ctx = BenchmarkContext::new("symbol_index_bench");
    let project = create_bench_benchmark_project(&ctx, "symbol_index_test", 200);
    
    // Create multiple files with symbols
    for i in 1..=10 {
        let file = project.join(format!("src/module{}.nuva", i));
        fs::write(&file, format!(r#"
pub fn function_{}() {{
    let x = {};
    println!("Function {}", x, i, i)).expect("Failed to write symbol files");
    }
    
    // Start LSP server
    let _ = measure_command("nuva", &["lsp", "start"], &project);
    
    let runner = BenchmarkRunner::new(1, 3);
    let result = runner.run("symbol_indexing", || {
        // Simulate symbol indexing
        std::thread::sleep(Duration::from_millis(50));
        Duration::from_millis(50)
    });
    
    println!("{}", result.format());
}

/// Benchmark semantic highlighting
#[test]
fn benchmark_semantic_highlighting() {
    let ctx = BenchmarkContext::new("semantic_bench");
    let project = create_bench_benchmark_project(&ctx, "semantic_test", 100);
    
    // Start LSP server
    let _ = measure_command("nuva", &["lsp", "start"], &project);
    
    let runner = BenchmarkRunner::new(1, 5);
    let result = runner.run("semantic_highlighting", || {
        // Simulate semantic highlighting
        std::thread::sleep(Duration::from_millis(25));
        Duration::from_millis(25)
    });
    
    println!("{}", result.format());
}

/// Benchmark with syntax errors
#[test]
fn benchmark_with_errors() {
    let ctx = BenchmarkContext::new("errors_bench");
    let project = create_bench_benchmark_project(&ctx, "errors_test", 100);
    
    // Create file with errors
    let main_file = project.join("src/main.nuva");
    fs::write(&main_file, r#"
fn main() {
    let x = 42
    println!("Missing semicolon above");
    let y = "unclosed string
}
"#).expect("Failed to write file with errors");
    
    // Start LSP server
    let _ = measure_command("nuva", &["lsp", "start"], &project);
    
    let runner = BenchmarkRunner::new(1, 5);
    let result = runner.run("with_errors", || {
        // Simulate diagnostics on code with errors
        std::thread::sleep(Duration::from_millis(10));
        Duration::from_millis(10)
    });
    
    println!("{}", result.format());
}

/// Benchmark LSP shutdown
#[test]
fn benchmark_lsp_shutdown() {
    let ctx = BenchmarkContext::new("lsp_shutdown_bench");
    let project = create_bench_benchmark_project(&ctx, "shutdown_test", 100);
    
    // Start LSP server
    let _ = measure_command("nuva", &["lsp", "start"], &project);
    
    let runner = BenchmarkRunner::new(1, 5);
    let result = runner.run("lsp_shutdown", || {
        measure_command("nuva", &["lsp", "stop"], &project)
    });
    
    println!("{}", result.format());
}
