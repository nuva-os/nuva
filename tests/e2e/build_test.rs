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

//! End-to-end tests for the build system

use super::{TestContext, create_test_project, run_command};
use std::path::PathBuf;

/// Test complete build flow
#[test]
fn test_complete_build_flow() {
    let ctx = TestContext::new("build_flow_test");
    let project = create_test_project(&ctx, "build_test_project");
    
    // Initialize the project
    let output = run_command("nuva", &["init"], &project)
        .expect("Failed to run init command");
    assert!(output.contains("Initialized"));
    
    // Build the project
    let output = run_command("nuva", &["build"], &project)
        .expect("Failed to run build command");
    assert!(output.contains("Compiling") || output.contains("Building"));
    
    // Check if binary was created
    let binary = project.join("target/debug/build_test_project");
    assert!(binary.exists(), "Binary should exist after build");
    
    // Clean the build
    let output = run_command("nuva", &["clean"], &project)
        .expect("Failed to run clean command");
    assert!(output.contains("Cleaned"));
}

/// Test build with different optimization levels
#[test]
fn test_build_with_optimization_levels() {
    let ctx = TestContext::new("opt_levels_test");
    let project = create_test_project(&ctx, "opt_test_project");
    
    // Build with -O0
    let output = run_command("nuva", &["build", "--opt-level", "0"], &project)
        .expect("Failed to run build command");
    assert!(output.contains("Compiling"));
    
    // Build with -O2
    let output = run_command("nuva", &["build", "--opt-level", "2"], &project)
        .expect("Failed to run build command");
    assert!(output.contains("Compiling"));
    
    // Build with -O3
    let output = run_command("nuva", &["build", "--opt-level", "3"], &project)
        .expect("Failed to run build command");
    assert!(output.contains("Compiling"));
}

/// Test build with debug info
#[test]
fn test_build_with_debug_info() {
    let ctx = TestContext::new("debug_build_test");
    let project = create_test_project(&ctx, "debug_test_project");
    
    // Build with debug info
    let output = run_command("nuva", &["build", "--debug"], &project)
        .expect("Failed to run build command");
    assert!(output.contains("Compiling"));
    
    // Check if debug symbols are included
    let binary = project.join("target/debug/debug_test_project");
    if binary.exists() {
        // In a real implementation, we would check for debug symbols
        // using tools like objdump or readelf
    }
}

/// Test build with custom target
#[test]
fn test_build_with_custom_target() {
    let ctx = TestContext::new("custom_target_test");
    let project = create_test_project(&ctx, "target_test_project");
    
    // Build for a specific target
    let output = run_command("nuva", &["build", "--target", "aarch64-unknown-linux-gnu"], &project)
        .expect("Failed to run build command");
    assert!(output.contains("Compiling") || output.contains("Building"));
}

/// Test incremental build
#[test]
fn test_incremental_build() {
    let ctx = TestContext::new("incremental_build_test");
    let project = create_test_project(&ctx, "incremental_test_project");
    
    // First build
    let output1 = run_command("nuva", &["build"], &project)
        .expect("Failed to run first build");
    assert!(output1.contains("Compiling"));
    
    // Modify source file
    let main_file = project.join("src/main.nuva");
    let new_content = r#"
fn main() {
    println!("Modified: Hello from Nuva!");
}
"#;
    fs::write(&main_file, new_content).expect("Failed to modify main file");
    
    // Second build (should be incremental)
    let output2 = run_command("nuva", &["build"], &project)
        .expect("Failed to run second build");
    // In a real implementation, we would verify that only the modified file was recompiled
    assert!(output2.contains("Compiling"));
}

/// Test build with dependencies
#[test]
fn test_build_with_dependencies() {
    let ctx = TestContext::new("dependencies_build_test");
    let project = create_test_project(&ctx, "deps_test_project");
    
    // Create a library file
    let lib_file = project.join("src/lib.nuva");
    fs::write(&lib_file, r#"
pub fn helper() {
    println!("Helper function");
}
"#).expect("Failed to write lib file");
    
    // Update main to use the library
    let main_file = project.join("src/main.nuva");
    let new_content = r#"
use lib::helper;
use alloc::format;

fn main() {
    helper();
    println!("Hello from Nuva!");
}
"#;
    fs::write(&main_file, new_content).expect("Failed to write main file");
    
    // Build with dependencies
    let output = run_command("nuva", &["build"], &project)
        .expect("Failed to run build command");
    assert!(output.contains("Compiling"));
}

/// Test build failure handling
#[test]
fn test_build_failure_handling() {
    let ctx = TestContext::new("build_failure_test");
    let project = create_test_project(&ctx, "failure_test_project");
    
    // Create a file with syntax error
    let main_file = project.join("src/main.nuva");
    let error_content = r#"
fn main() {
    println!("Hello from Nuva!"
}
"#; // Missing closing parenthesis
    fs::write(&main_file, error_content).expect("Failed to write main file");
    
    // Build should fail
    let result = run_command("nuva", &["build"], &project);
    assert!(result.is_err(), "Build should fail with syntax error");
}

/// Test parallel build
#[test]
fn test_parallel_build() {
    let ctx = TestContext::new("parallel_build_test");
    let project = create_test_project(&ctx, "parallel_test_project");
    
    // Create multiple source files
    for i in 1..=5 {
        let file = project.join(format!("src/module{}.nuva", i));
        fs::write(&file, format!(r#"
pub fn module_{}() {{
    println!("Module {}");
}}
"#, i, i)).expect("Failed to write module file");
    }
    
    // Build with parallel jobs
    let output = run_command("nuva", &["build", "--jobs", "4"], &project)
        .expect("Failed to run parallel build");
    assert!(output.contains("Compiling"));
}

/// Test build cache
#[test]
fn test_build_cache() {
    let ctx = TestContext::new("build_cache_test");
    let project = create_test_project(&ctx, "cache_test_project");
    
    // First build
    let output1 = run_command("nuva", &["build"], &project)
        .expect("Failed to run first build");
    
    // Second build without changes (should use cache)
    let output2 = run_command("nuva", &["build"], &project)
        .expect("Failed to run second build");
    // In a real implementation, we would verify cache usage
}

/// Test cross-compilation
#[test]
fn test_cross_compilation() {
    let ctx = TestContext::new("cross_compile_test");
    let project = create_test_project(&ctx, "cross_test_project");
    
    // Cross-compile for different architecture
    let output = run_command("nuva", &["build", "--target", "x86_64-unknown-linux-gnu"], &project)
        .expect("Failed to run cross-compilation");
    assert!(output.contains("Compiling") || output.contains("Building"));
}
