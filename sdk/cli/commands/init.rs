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

// ! Initialize project in current directory

use crate::NuvaSdk;
use crate::error::SdkError;
use crate::cli::args::InitCommand;
use crate::cli::output;

/// Execute init command
pub fn execute(sdk: &mut NuvaSdk, cmd: InitCommand) -> Result<(), SdkError> {
    output::info("Initializing project...");

    let name = cmd.name.as_ref().cloned().unwrap_or_else(|| {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "nuva-project".to_string())
    });

    let template = cmd.template.as_deref().unwrap_or("default");

    let config = generate_project_config(&name, template);
    std::fs::write("Nuva.toml", config)
        .map_err(|e| SdkError::IoError(e.to_string()))?;

    std::fs::create_dir_all("src")
        .map_err(|e| SdkError::IoError(e.to_string()))?;

    let main_content = generate_main_file(template);
    std::fs::write("src/main.nuva", main_content)
        .map_err(|e| SdkError::IoError(e.to_string()))?;

    let build_content = generate_build_config(template);
    std::fs::write("build.rs", build_content)
        .map_err(|e| SdkError::IoError(e.to_string()))?;

    let gitignore = generate_gitignore();
    std::fs::write(".gitignore", gitignore)
        .map_err(|e| SdkError::IoError(e.to_string()))?;

    if template == "kernel" || template == "driver" {
        std::fs::create_dir_all("tests")
            .map_err(|e| SdkError::IoError(e.to_string()))?;

        let test_content = generate_test_file(template);
        std::fs::write("tests/main_test.rs", test_content)
            .map_err(|e| SdkError::IoError(e.to_string()))?;
    }

    if template == "kernel" {
        std::fs::create_dir_all("docs")
            .map_err(|e| SdkError::IoError(e.to_string()))?;

        let readme = generate_readme(&name, template);
        std::fs::write("docs/README.md", readme)
            .map_err(|e| SdkError::IoError(e.to_string()))?;
    }

    output::success(&format!("Project '{}' initialized successfully", name));
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
nuva-syslib = "0.1.0"
"#,
        _ => "",
    };

    format!(r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
authors = ["YuJie.Zhang <kellen9903@gmail.com>"]
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
    // Initialize kernel subsystems
    loop {}
}
"#.to_string(),
        "driver" => r#"// Driver entry point
#![no_std]

pub fn driver_init() -> i32 {
    // Initialize hardware driver
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

/// Generate build configuration
fn generate_build_config(template: &str) -> String {
    match template {
        "kernel" => r#"// Build script for kernel module

fn main() {
    // Kernel-specific build configuration
    println!("cargo:rustc-link-arg=-nostdlib");
}
"#.to_string(),
        _ => String::new(),
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

/// Generate README
fn generate_readme(name: &str, template: &str) -> String {
    format!(r#"# {}

A {} module for Nuva OS.

## Building

```bash
nuva build
```

## Testing

```bash
nuva test
```

## Running

```bash
nuva run
```
"#, name, template)
}
