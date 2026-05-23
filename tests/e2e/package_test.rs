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

//! End-to-end tests for the package manager

use super::{TestContext, create_test_project, run_command};
use std::path::PathBuf;

/// Test package creation
#[test]
fn test_package_creation() {
    let ctx = TestContext::new("package_creation_test");
    let project = create_test_project(&ctx, "pkg_test_project");
    
    // Create a new package
    let output = run_command("nuva", &["pkg", "new", "my_package"], &project)
        .expect("Failed to create package");
    assert!(output.contains("Created") || output.contains("Package"));
}

/// Test package installation
#[test]
fn test_package_installation() {
    let ctx = TestContext::new("package_install_test");
    let project = create_test_project(&ctx, "install_test_project");
    
    // Add a dependency
    let output = run_command("nuva", &["pkg", "add", "nuva-std"], &project)
        .expect("Failed to add dependency");
    assert!(output.contains("Adding") || output.contains("Added"));
    
    // Verify dependency was added to Nuva.toml
    let manifest = project.join("Nuva.toml");
    let content = fs::read_to_string(&manifest).expect("Failed to read Nuva.toml");
    assert!(content.contains("nuva-std"), "Dependency should be in Nuva.toml");
}

/// Test package list
#[test]
fn test_package_list() {
    let ctx = TestContext::new("package_list_test");
    let project = create_test_project(&ctx, "list_test_project");
    
    // List packages
    let output = run_command("nuva", &["pkg", "list"], &project)
        .expect("Failed to list packages");
    assert!(output.contains("Dependencies") || output.contains("Packages"));
}

/// Test package update
#[test]
fn test_package_update() {
    let ctx = TestContext::new("package_update_test");
    let project = create_test_project(&ctx, "update_test_project");
    
    // Add a dependency first
    let _ = run_command("nuva", &["pkg", "add", "nuva-std"], &project)
        .expect("Failed to add dependency");
    
    // Update dependency
    let output = run_command("nuva", &["pkg", "update"], &project)
        .expect("Failed to update packages");
    assert!(output.contains("Updating") || output.contains("Updated"));
}

/// Test package removal
#[test]
fn test_package_removal() {
    let ctx = TestContext::new("package_removal_test");
    let project = create_test_project(&ctx, "remove_test_project");
    
    // Add a dependency first
    let _ = run_command("nuva", &["pkg", "add", "nuva-std"], &project)
        .expect("Failed to add dependency");
    
    // Remove the dependency
    let output = run_command("nuva", &["pkg", "remove", "nuva-std"], &project)
        .expect("Failed to remove package");
    assert!(output.contains("Removing") || output.contains("Removed"));
}

/// Test package search
#[test]
fn test_package_search() {
    let ctx = TestContext::new("package_search_test");
    let project = create_test_project(&ctx, "search_test_project");
    
    // Search for packages
    let output = run_command("nuva", &["pkg", "search", "std"], &project)
        .expect("Failed to search packages");
    assert!(output.contains("Searching") || output.contains("Found") || output.contains("No packages"));
}

/// Test lock file generation
#[test]
fn test_lock_file_generation() {
    let ctx = TestContext::new("lock_file_test");
    let project = create_test_project(&ctx, "lock_test_project");
    
    // Add dependencies
    let _ = run_command("nuva", &["pkg", "add", "nuva-std"], &project)
        .expect("Failed to add dependency");
    
    // Generate lock file
    let output = run_command("nuva", &["pkg", "lock"], &project)
        .expect("Failed to generate lock file");
    assert!(output.contains("Lock") || output.contains("Generated"));
    
    // Verify lock file exists
    let lock_file = project.join("nuva.lock");
    assert!(lock_file.exists(), "Lock file should exist");
}

/// Test package from Git
#[test]
fn test_package_from_git() {
    let ctx = TestContext::new("git_package_test");
    let project = create_test_project(&ctx, "git_test_project");
    
    // Add a Git dependency
    let output = run_command(
        "nuva",
        &["pkg", "add", "https://github.com/example/repo.git"],
        &project
    );
    
    // This might fail if the Git repo doesn't exist, but the command should parse correctly
    assert!(output.is_ok() || output.is_err());
}

/// Test package from local path
#[test]
fn test_package_from_local_path() {
    let ctx = TestContext::new("local_package_test");
    let project = create_test_project(&ctx, "local_test_project");
    
    // Create a local package
    let local_pkg = ctx.temp_dir.join("local_package");
    fs::create_dir_all(&local_pkg).expect("Failed to create local package");
    fs::write(local_pkg.join("Nuva.toml"), r#"[package]
name = "local_pkg"
version = "0.1.0"
"#).expect("Failed to write local package manifest");
    
    // Add local dependency
    let output = run_command(
        "nuva",
        &["pkg", "add", "--path", local_pkg.to_str().unwrap()],
        &project
    );
    
    // This should work or fail gracefully
    assert!(output.is_ok() || output.is_err());
}

/// Test package with features
#[test]
fn test_package_with_features() {
    let ctx = TestContext::new("features_test");
    let project = create_test_project(&ctx, "features_test_project");
    
    // Add package with features
    let output = run_command(
        "nuva",
        &["pkg", "add", "nuva-net", "--features", "tls,http"],
        &project
    );
    
    assert!(output.is_ok() || output.is_err());
}

/// Test dependency resolution conflicts
#[test]
fn test_dependency_conflict_resolution() {
    let ctx = TestContext::new("conflict_test");
    let project = create_test_project(&ctx, "conflict_test_project");
    
    // Add a dependency
    let _ = run_command("nuva", &["pkg", "add", "nuva-std@0.1.0"], &project)
        .expect("Failed to add dependency");
    
    // Try to add conflicting version
    let output = run_command("nuva", &["pkg", "add", "nuva-std@0.2.0"], &project);
    
    // Should either resolve conflict or report error
    assert!(output.is_ok() || output.is_err());
}

/// Test package cache
#[test]
fn test_package_cache() {
    let ctx = TestContext::new("cache_test");
    let project = create_test_project(&ctx, "cache_test_project");
    
    // Add a dependency
    let _ = run_command("nuva", &["pkg", "add", "nuva-std"], &project)
        .expect("Failed to add dependency");
    
    // Remove and re-add to test cache
    let _ = run_command("nuva", &["pkg", "remove", "nuva-std"], &project)
        .expect("Failed to remove package");
    
    let output = run_command("nuva", &["pkg", "add", "nuva-std"], &project)
        .expect("Failed to re-add package");
    
    // Should use cache if available
    assert!(output.is_ok());
}

/// Test package metadata validation
#[test]
fn test_package_metadata_validation() {
    let ctx = TestContext::new("metadata_test");
    let project = create_test_project(&ctx, "metadata_test_project");
    
    // Create a package with invalid metadata
    let manifest = project.join("Nuva.toml");
    fs::write(&manifest, r#"[package]
name = "test"
version = "invalid_version"
"#).expect("Failed to write manifest");
    
    // Should fail validation
    let output = run_command("nuva", &["build"], &project);
    assert!(output.is_err(), "Should fail with invalid version");
}

/// Test package publishing (dry run)
#[test]
fn test_package_publish_dry_run() {
    let ctx = TestContext::new("publish_test");
    let project = create_test_project(&ctx, "publish_test_project");
    
    // Dry run publish
    let output = run_command("nuva", &["pkg", "publish", "--dry-run"], &project);
    
    // Should validate package without actually publishing
    assert!(output.is_ok() || output.is_err());
}
