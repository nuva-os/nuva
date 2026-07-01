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

//! End-to-end tests for Nuva SDK
/*!*/
//! This module contains comprehensive end-to-end tests that verify the complete
//! functionality of the Nuva SDK toolchain, including build system, package manager,
//! debugger, and debugger.

pub mod build_test;
pub mod package_test;
pub mod debug_test;
pub mod integration_test;

use std::path::PathBuf;
use std::fs;

/// Test context for e2e tests
pub struct TestContext {
    /// Test workspace directory
    pub workspace: PathBuf,
    /// Temporary directory for test artifacts
    pub temp_dir: PathBuf,
}

impl TestContext {
    /// Create a new test context
    pub fn new(test_name: &str) -> Self {
        let test_dir = std::env::temp_dir().join(format!("nuva_e2e_{}", test_name));
        fs::create_dir_all(&test_dir).expect("Failed to create test directory");
        
        Self {
            workspace: test_dir.join("workspace"),
            temp_dir: test_dir,
        }
    }
    
    /// Clean up test context
    pub fn cleanup(&self) {
        if self.temp_dir.exists() {
            let _ = fs::remove_dir_all(&self.temp_dir);
        }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// Create a simple test project
pub fn create_test_project(ctx: &TestContext, name: &str) -> PathBuf {
    let project_dir = ctx.workspace.join(name);
    fs::create_dir_all(&project_dir).expect("Failed to create project directory");
    
    // Create src directory
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir).expect("Failed to create src directory");
    
    // Create a simple main file
    let main_file = src_dir.join("main.nuva");
    fs::write(&main_file, r#"
fn main() {
    println!("Hello from Nuva!");
}
"#).expect("Failed to write main file");
    
    // Create Nuva.toml
    let manifest = project_dir.join("Nuva.toml");
    fs::write(&manifest, r#"[package]
name = "test_project"
version = "0.1.0"
edition = "2024"

[dependencies]
"#).expect("Failed to write Nuva.toml");
    
    project_dir
}

/// Run a command and capture output
pub fn run_command(cmd: &str, args: &[&str], cwd: &PathBuf) -> Result<String, std::io::Error> {
    let output = std::process::Command::new(cmd)
        .args(args)
        .current_dir(cwd)
        .output()?;
    
    if !output.status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Command failed: {}", String::from_utf8_lossy(&output.stderr))
        ));
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
use alloc::format;
    
    #[test]
    fn test_context_creation() {
        let ctx = TestContext::new("context_test");
        assert!(ctx.workspace.exists());
        assert!(ctx.temp_dir.exists());
    }
    
    #[test]
    fn test_project_creation() {
        let ctx = TestContext::new("project_test");
        let project = create_test_project(&ctx, "simple_project");
        assert!(project.exists());
        assert!(project.join("src/main.nuva").exists());
        assert!(project.join("Nuva.toml").exists());
    }
}
