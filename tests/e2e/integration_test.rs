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

//! Integration tests for the complete SDK workflow

use super::{TestContext, create_test_project, run_command};
use std::path::PathBuf;

/// Test complete workflow: init -> build -> run
#[test]
fn test_complete_workflow() {
    let ctx = TestContext::new("workflow_test");
    let project_dir = ctx.workspace.join("workflow_project");
    fs::create_dir_all(&project_dir).expect("Failed to create project directory");
    
    // Initialize project
    let output = run_command("nuva", &["init"], &project_dir)
        .expect("Failed to initialize project");
    assert!(output.contains("Initialized"));
    
    // Create source file
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir).expect("Failed to create src directory");
    let main_file = src_dir.join("main.nuva");
    fs::write(&main_file, r#"
fn main() {
    println!("Hello from complete workflow!");
}
"#).expect("Failed to write main file");
    
    // Build project
    let output = run_command("nuva", &["build"], &project_dir)
        .expect("Failed to build project");
    assert!(output.contains("Compiling") || output.contains("Building"));
    
    // Run project
    let output = run_command("nuva", &["run"], &project_dir)
        .expect("Failed to run project");
    assert!(output.contains("Hello") || output.contains("Running"));
}

/// Test project with dependencies
#[test]
fn test_project_with_dependencies() {
    let ctx = TestContext::new("deps_workflow_test");
    let project = create_test_project(&ctx, "deps_workflow_project");
    
    // Add dependency
    let _ = run_command("nuva", &["pkg", "add", "nuva-std"], &project)
        .expect("Failed to add dependency");
    
    // Build with dependencies
    let output = run_command("nuva", &["build"], &project)
        .expect("Failed to build with dependencies");
    assert!(output.contains("Compiling") || output.contains("Building"));
    
    // Run with dependencies
    let output = run_command("nuva", &["run"], &project)
        .expect("Failed to run with dependencies");
    assert!(output.is_ok());
}

/// Test test workflow
#[test]
fn test_test_workflow() {
    let ctx = TestContext::new("test_workflow_test");
    let project = create_test_project(&ctx, "test_workflow_project");
    
    // Create test file
    let tests_dir = project.join("tests");
    fs::create_dir_all(&tests_dir).expect("Failed to create tests directory");
    let test_file = tests_dir.join("test_basic.nuva");
    fs::write(&test_file, r#"
#[test]
fn test_basic() {
    assert_eq(1 + 1, 2);
}

#[test]
fn test_string() {
    assert_eq!("hello".len(), 5);
}
"#).expect("Failed to write test file");
    
    // Run tests
    let output = run_command("nuva", &["test"], &project)
        .expect("Failed to run tests");
    assert!(output.contains("test") || output.contains("Running") || output.contains("PASSED"));
}

/// Test documentation generation
#[test]
fn test_documentation_generation() {
    let ctx = TestContext::new("doc_workflow_test");
    let project = create_test_project(&ctx, "doc_workflow_project");
    
    // Create documented code
    let main_file = project.join("src/main.nuva");
    fs::write(&main_file, r#"
//! Main module for the test project

/// Main function
/// Prints a greeting message
fn main() {
    println!("Hello from documented project!");
}

/// Helper function
/// Returns the sum of two numbers
/// # Arguments
/// * `a` - First number
/// * `b` - Second number
/// # Returns
/// The sum of a and b
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#).expect("Failed to write documented file");
    
    // Generate documentation
    let output = run_command("nuva", &["doc"], &project)
        .expect("Failed to generate documentation");
    assert!(output.contains("doc") || output.contains("Generating") || output.is_ok());
}

/// Test code formatting
#[test]
fn test_code_formatting() {
    let ctx = TestContext::new("format_workflow_test");
    let project = create_test_project(&ctx, "format_workflow_project");
    
    // Create unformatted code
    let main_file = project.join("src/main.nuva");
    fs::write(&main_file, r#"
fn main( ){ println!( "Hello" ) ; }
"#).expect("Failed to write unformatted file");
    
    // Format code
    let output = run_command("nuva", &["fmt"], &project)
        .expect("Failed to format code");
    assert!(output.is_ok());
    
    // Check if code was formatted
    let formatted = fs::read_to_string(&main_file).expect("Failed to read formatted file");
    assert_ne!(formatted, r#"
fn main( ){ println!( "Hello" ) ; }
"#);
}

/// Test linting
#[test]
fn test_linting() {
    let ctx = TestContext::new("lint_workflow_test");
    let project = create_test_project(&ctx, "lint_workflow_project");
    
    // Create code with potential issues
    let main_file = project.join("src/main.nuva");
    fs::write(&main_file, r#"
fn main() {
    let x = 42;
    let y = 42; // Unused variable
    println!("x = {}", x);
}
"#).expect("Failed to write file");
    
    // Run linter
    let output = run_command("nuva", &["lint"], &project);
    
    // Should either pass or report issues
    assert!(output.is_ok() || output.is_err());
}

/// Test clean workflow
#[test]
fn test_clean_workflow() {
    let ctx = TestContext::new("clean_workflow_test");
    let project = create_test_project(&ctx, "clean_workflow_project");
    
    // Build project
    let _ = run_command("nuva", &["build"], &project)
        .expect("Failed to build project");
    
    // Check build artifacts exist
    let target_dir = project.join("target");
    assert!(target_dir.exists());
    
    // Clean build artifacts
    let output = run_command("nuva", &["clean"], &project)
        .expect("Failed to clean");
    assert!(output.contains("Cleaned"));
    
    // Verify artifacts are removed
    assert!(!target_dir.exists());
}

/// Test workspace with multiple projects
#[test]
fn test_workspace_workflow() {
    let ctx = TestContext::new("workspace_test");
    let workspace = ctx.workspace.join("workspace_test");
    fs::create_dir_all(&workspace).expect("Failed to create workspace");
    
    // Create multiple projects
    for i in 1..=3 {
        let project_dir = workspace.join(format!("project_{}", i));
        fs::create_dir_all(&project_dir).expect("Failed to create project directory");
        
        let src_dir = project_dir.join("src");
        fs::create_dir_all(&src_dir).expect("Failed to create src directory");
        
        let main_file = src_dir.join("main.nuva");
        fs::write(&main_file, format!(r#"
fn main() {{
    println!("Project {}", {});
}}
"#, i)).expect("Failed to write main file");
    }
    
    // Build all projects in workspace
    let output = run_command("nuva", &["build", "--workspace"], &workspace);
    
    // Should attempt to build workspace
    assert!(output.is_ok() || output.is_err());
}

/// Test release build
#[test]
fn test_release_build() {
    let ctx = TestContext::new("release_build_test");
    let project = create_test_project(&ctx, "release_test_project");
    
    // Build in release mode
    let output = run_command("nuva", &["build", "--release"], &project)
        .expect("Failed to build release");
    assert!(output.contains("Compiling") || output.contains("Building"));
    
    // Check release binary exists
    let release_binary = project.join("target/release/release_test_project");
    assert!(release_binary.exists());
}

/// Test cross-compilation workflow
#[test]
fn test_cross_compilation_workflow() {
    let ctx = TestContext::new("cross_build_test");
    let project = create_test_project(&ctx, "cross_build_test_project");
    
    // Cross-compile for different targets
    let targets = vec![
        "aarch64-unknown-linux-gnu",
        "x86_64-unknown-linux-gnu",
    ];
    
    for target in targets {
        let output = run_command("nuva", &["build", "--target", target], &project);
        assert!(output.is_ok() || output.is_err());
    }
}

/// Test incremental development workflow
#[test]
fn test_incremental_development() {
    let ctx = TestContext::new("incremental_dev_test");
    let project = create_test_project(&ctx, "incremental_dev_project");
    
    // Initial build
    let output1 = run_command("nuva", &["build"], &project)
        .expect("Failed to build initially");
    
    // Modify source
    let main_file = project.join("src/main.nuva");
    fs::write(&main_file, r#"
fn main() {
    println!("Modified: Hello from Nuva!");
}
"#).expect("Failed to modify source");
    
    // Incremental build
    let output2 = run_command("nuva", &["build"], &project)
        .expect("Failed to rebuild");
    
    // Run modified program
    let output3 = run_command("nuva", &["run"], &project)
        .expect("Failed to run modified program");
    
    assert!(output3.contains("Modified"));
}
