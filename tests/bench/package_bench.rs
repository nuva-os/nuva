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

//! Package manager performance benchmarks

use super::{BenchmarkContext, BenchmarkRunner, create_bench_benchmark_project, measure_command};
use std::time::Duration;
use alloc::vec;

/// Benchmark package installation
#[test]
fn benchmark_package_installation() {
    let ctx = BenchmarkContext::new("package_install_bench");
    let project = create_bench_benchmark_project(&ctx, "install_test", 100);
    
    let runner = BenchmarkRunner::new(1, 3);
    let result = runner.run("package_install", || {
        measure_command("nuva", &["pkg", "add", "nuva-std"], &project)
    });
    
    println!("{}", result.format());
}

/// Benchmark dependency resolution
#[test]
fn benchmark_dependency_resolution() {
    let ctx = BenchmarkContext::new("dep_resolution_bench");
    let project = create_bench_benchmark_project(&ctx, "dep_resolution_test", 100);
    
    // Add multiple dependencies
    let deps = vec!["nuva-std", "nuva-net", "nuva-crypto"];
    
    for dep in deps {
        let _ = measure_command("nuva", &["pkg", "add", dep], &project);
    }
    
    let runner = BenchmarkRunner::new(1, 3);
    let result = runner.run("dep_resolution", || {
        measure_command("nuva", &["pkg", "lock"], &project)
    });
    
    println!("{}", result.format());
}

/// Benchmark package search
#[test]
fn benchmark_package_search() {
    let ctx = BenchmarkContext::new("package_search_bench");
    let project = create_bench_benchmark_project(&ctx, "search_test", 100);
    
    let runner = BenchmarkRunner::new(1, 5);
    let result = runner.run("package_search", || {
        measure_command("nuva", &["pkg", "search", "std"], &project)
    });
    
    println!("{}", result.format());
}

/// Benchmark package update
#[test]
fn benchmark_package_update() {
    let ctx = BenchmarkContext::new("package_update_bench");
    let project = create_bench_benchmark_project(&ctx, "update_test", 100);
    
    // Add dependency first
    let _ = measure_command("nuva", &["pkg", "add", "nuva-std"], &project);
    
    let runner = BenchmarkRunner::new(1, 3);
    let result = runner.run("package_update", || {
        measure_command("nuva", &["pkg", "update"], &project)
    });
    
    println!("{}", result.format());
}

/// Benchmark package list
#[test]
fn benchmark_package_list() {
    let ctx = BenchmarkContext::new("package_list_bench");
    let project = create_bench_benchmark_project(&ctx, "list_test", 100);
    
    // Add multiple dependencies
    let deps = vec!["nuva-std", "nuva-net", "nuva-crypto"];
    for dep in deps {
        let _ = measure_command("nuva", &["pkg", "add", dep], &project);
    }
    
    let runner = BenchmarkRunner::new(1, 10);
    let result = runner.run("package_list", || {
        measure_command("nuva", &["pkg", "list"], &project)
    });
    
    println!("{}", result.format());
}

/// Benchmark package removal
#[test]
fn benchmark_package_removal() {
    let ctx = BenchmarkContext::new("package_removal_bench");
    let project = create_bench_benchmark_project(&ctx, "removal_test", 100);
    
    // Add dependencies first
    let deps = vec!["nuva-std", "nuva-net"];
    for dep in deps {
        let _ = measure_command("nuva", &["pkg", "add", dep], &project);
    }
    
    let runner = BenchmarkRunner::new(1, 3);
    let result = runner.run("package_removal", || {
        measure_command("nuva", &["pkg", "remove", "nuva-std"], &project)
    });
    
    println!("{}", result.format());
}

/// Benchmark package publish (dry run)
#[test]
fn benchmark_package_publish() {
    let ctx = BenchmarkContext::new("package_publish_bench");
    let project = create_bench_benchmark_project(&ctx, "publish_test", 100);
    
    let runner = BenchmarkRunner::new(1, 3);
    let result = runner.run("package_publish", || {
        measure_command("nuva", &["pkg", "publish", "--dry-run"], &project)
    });
    
    println!("{}", result.format());
}

/// Benchmark with many dependencies
#[test]
fn benchmark_many_dependencies() {
    let ctx = BenchmarkContext::new("many_deps_bench");
    let project = create_bench_benchmark_project(&ctx, "many_deps_test", 100);
    
    // Add many dependencies
    let deps = vec![
        "nuva-std", "nuva-net", "nuva-crypto", "nuva-http", "nuva-json",
        "nuva-async", "nuva-time", "nuva-math", "nuva-random", "nuva-logging",
    ];
    
    for dep in deps {
        let _ = measure_command("nuva", &["pkg", "add", dep], &project);
    }
    
    let runner = BenchmarkRunner::new(1, 3);
    let result = runner.run("many_deps_resolution", || {
        measure_command("nuva", &["pkg", "lock"], &project)
    });
    
    println!("{}", result.format());
}

/// Benchmark package cache
#[test]
fn benchmark_package_cache() {
    let ctx = BenchmarkContext::new("package_cache_bench");
    let project = create_bench_benchmark_project(&ctx, "cache_test", 100);
    
    // First installation
    let _ = measure_command("nuva", &["pkg", "add", "nuva-std"], &project);
    
    // Remove
    let _ = measure_command("nuva", &["pkg", "remove", "nuva-std"], &project);
    
    // Re-install (should use cache)
    let runner = BenchmarkRunner::new(1, 3);
    let result = runner.run("cached_install", || {
        measure_command("nuva", &["pkg", "add", "nuva-std"], &project)
    });
    
    println!("{}", result.format());
}

/// Benchmark Git dependencies
#[test]
fn benchmark_git_dependencies() {
    let ctx = BenchmarkContext::new("git_deps_bench");
    let project = create_bench_benchmark_project(&ctx, "git_deps_test", 100);
    
    let runner = BenchmarkRunner::new(1, 3);
    let result = runner.run("git_dep_install", || {
        measure_command(
            "nuva",
            &["pkg", "add", "https://github.com/example/repo.git"],
            &project
        )
    });
    
    println!("{}", result.format());
}

/// Benchmark local path dependencies
#[test]
fn benchmark_local_dependencies() {
    let ctx = BenchmarkContext::new("local_deps_bench");
    let project = create_bench_benchmark_project(&ctx, "local_deps_test", 100);
    
    // Create local package
    let local_pkg = ctx.temp_dir.join("local_package");
    fs::create_dir_all(&local_pkg).expect("Failed to create local package");
    fs::write(local_pkg.join("Nuva.toml"), r#"[package]
name = "local_pkg"
version = "0.1.0"
"#).expect("Failed to write local package manifest");
    
    let runner = BenchmarkRunner::new(1, 3);
    let result = runner.run("local_dep_install", || {
        measure_command(
            "nuva",
            &["pkg", "add", "--path", local_pkg.to_str().unwrap()],
            &project
        )
    });
    
    println!("{}", result.format());
}

/// Benchmark with features
#[test]
fn benchmark_package_features() {
    let ctx = BenchmarkContext::new("features_bench");
    let project = create_bench_benchmark_project(&ctx, "features_test", 100);
    
    let runner = BenchmarkRunner::new(1, 3);
    let result = runner.run("features_install", || {
        measure_command(
            "nuva",
            &["pkg", "add", "nuva-net", "--features", "tls,http,websocket"],
            &project
        )
    });
    
    println!("{}", result.format());
}

/// Benchmark lock file generation
#[test]
fn benchmark_lock_file_generation() {
    let ctx = BenchmarkContext::new("lock_file_bench");
    let project = create_bench_benchmark_project(&ctx, "lock_file_test", 100);
    
    // Add dependencies
    let deps = vec!["nuva-std", "nuva-net", "nuva-crypto"];
    for dep in deps {
        let _ = measure_command("nuva", &["pkg", "add", dep], &project);
    }
    
    let runner = BenchmarkRunner::new(1, 5);
    let result = runner.run("lock_generation", || {
        measure_command("nuva", &["pkg", "lock"], &project)
    });
    
    println!("{}", result.format());
}

/// Benchmark package metadata validation
#[test]
fn benchmark_metadata_validation() {
    let ctx = BenchmarkContext::new("metadata_validation_bench");
    let project = create_bench_benchmark_project(&ctx, "metadata_test", 100);
    
    let runner = BenchmarkRunner::new(1, 5);
    let result = runner.run("metadata_validation", || {
        measure_command("nuva", &["build"], &project)
    });
    
    println!("{}", result.format());
}
