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

// ! Create a new project

use std::path::PathBuf;
use crate::NuvaSdk;
use crate::error::SdkError;
use crate::cli::args::NewCommand;
use crate::cli::output;

/// Execute new project command
pub fn execute(sdk: &mut NuvaSdk, cmd: NewCommand) -> Result<(), SdkError> {
    let path = cmd.path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(&cmd.name));

    output::info(&format!("Creating new project '{}' at {}...", cmd.name, path.display()));

    if path.exists() {
        return Err(SdkError::IoError(format!(
            "Directory {} already exists",
            path.display()
        )));
    }

    std::fs::create_dir_all(&path)
        .map_err(|e| SdkError::IoError(e.to_string()))?;

    let template = cmd.template.as_deref().unwrap_or("default");

    let config = generate_project_config(&cmd.name, template);
    std::fs::write(path.join("Nuva.toml"), config)
        .map_err(|e| SdkError::IoError(e.to_string()))?;

    std::fs::create_dir_all(path.join("src"))
        .map_err(|e| SdkError::IoError(e.to_string()))?;

    let main_content = generate_main_file(template);
    std::fs::write(path.join("src/main.nuva"), main_content)
        .map_err(|e| SdkError::IoError(e.to_string()))?;

    let gitignore = generate_gitignore();
    std::fs::write(path.join(".gitignore"), gitignore)
        .map_err(|e| SdkError::IoError(e.to_string()))?;

    if template == "kernel" || template == "driver" {
        std::fs::create_dir_all(path.join("tests"))
            .map_err(|e| SdkError::IoError(e.to_string()))?;

        let test_content = generate_test_file(template);
        std::fs::write(path.join("tests/main_test.rs"), test_content)
            .map_err(|e| SdkError::IoError(e.to_string()))?;

        let build_content = generate_build_config(template);
        std::fs::write(path.join("build.rs"), build_content)
            .map_err(|e| SdkError::IoError(e.to_string()))?;
    }

    if template == "kernel" {
        std::fs::create_dir_all(path.join("docs"))
            .map_err(|e| SdkError::IoError(e.to_string()))?;
    }

    std::fs::create_dir_all(path.join(".vscode"))
        .map_err(|e| SdkError::IoError(e.to_string()))?;

    let vscode_settings = generate_vscode_settings();
    std::fs::write(path.join(".vscode/settings.json"), vscode_settings)
        .map_err(|e| SdkError::IoError(e.to_string()))?;

    output::success(&format!("Project '{}' created successfully", cmd.name));
    output::info(&format!("  cd {}", path.display()));
    output::info("  nuva build    # Build the project"));
    output::info("  nuva run      # Run the project"));
    Ok(())
}

/// Generate project configuration
fn generate_project_config(name: &str, template: &str) -> String {
    let deps = match template {
        "kernel" => r#"
[dependencies]
nuva-kernel = "0.1.0"
nuva-hal = "0.1.0"
"#,
        "driver" => r#"
[dependencies]
nuva-hal = "0.1.0"
"#,
        "app" => r#"
[dependencies]
nuva-app = "0.1.0"
"#,
        _ => "",
    };

    format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
authors = ["YuJie.Zhang <zhangyujie_china@163.com>"]
description = "A Nuva OS project"
{}

[[bin]]
name = "{}"
path = "src/main.nuva"
"#, name, deps, name)
}

/// Generate main entry file
fn generate_main_file(template: &str) -> String {
    match template {
        "kernel" => r#"// Kernel module entry point
#![no_std]
#![no_main]

fn kernel_main() -> ! {
    loop {}
}
"#.to_string(),
        "driver" => r#"// Driver entry point
#![no_std]

pub fn driver_init() -> i32 {
    0
}
"#.to_string(),
        "app" => r#"// Application entry point

fn main() {
    println!("Hello, Nuva OS!");
}
"#.to_string(),
        _ => r#"// Entry point

fn main() {
    println!("Hello, Nuva OS!");
}
"#.to_string(),
    }
}

/// Generate .gitignore
fn generate_gitignore() -> String {
    r#"/target
/.cache
*.o
*.a
*.so
*.d
*.swp
*.swo
*~
.DS_Store
"#.to_string()
}

/// Generate test file
fn generate_test_file(template: &str) -> String {
    match template {
        "kernel" => r#"// Kernel tests

#[test]
fn test_kernel_init() {
    // Test kernel initialization
}
"#.to_string(),
        "driver" => r#"// Driver tests

#[test]
fn test_driver_init() {
    // Test driver initialization
}
"#.to_string(),
        _ => String::new(),
    }
}

/// Generate build configuration
fn generate_build_config(template: &str) -> String {
    match template {
        "kernel" => r#"// Build script for kernel module

fn main() {
    println!("cargo:rustc-link-arg=-nostdlib");
}
"#.to_string(),
        _ => String::new(),
    }
}

/// Generate VS Code settings
fn generate_vscode_settings() -> String {
    r#"{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all"
}
"#.to_string()
}
