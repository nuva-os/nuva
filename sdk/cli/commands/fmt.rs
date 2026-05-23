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

// ! Format source code using rustfmt

use crate::NuvaSdk;
use crate::error::SdkError;
use crate::cli::args::FmtCommand;
use crate::cli::output;

/// Execute format command
pub fn execute(sdk: &mut NuvaSdk, cmd: FmtCommand) -> Result<(), SdkError> {
    if cmd.check {
        output::info("Checking formatting...");
    } else {
        output::info("Formatting source files...");
    }

    let target_files = if cmd.files.is_empty() {
        collect_rust_sources(sdk)?
    } else {
        cmd.files.clone()
    };

    if target_files.is_empty() {
        output::warning("No source files found to format");
        return Ok(());
    }

    output::info(&format!("Found {} source files", target_files.len()));

    let mut rustfmt_args = vec!["rustfmt".to_string()];

    if cmd.check {
        rustfmt_args.push("--check".to_string());
    }

    for file in &target_files {
        rustfmt_args.push(file.clone());
    }

    let result = std::process::Command::new("rustfmt")
        .args(&rustfmt_args[1..])
        .output()
        .map_err(|e| SdkError::ExecutionError(format!("Failed to run rustfmt: {}", e)))?;

    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);

    if cmd.check {
        if result.status.success() {
            output::success("All files are properly formatted");
        } else {
            output::error("Some files need formatting:");
            if !stdout.is_empty() {
                println!("{}", stdout);
            }
            if !stderr.is_empty() {
                eprintln!("{}", stderr);
            }
            return Err(SdkError::CommandError("Formatting check failed".to_string()));
        }
    } else {
        if result.status.success() {
            output::success("Formatting completed");
        } else {
            output::error("Formatting failed:");
            if !stderr.is_empty() {
                eprintln!("{}", stderr);
            }
            return Err(SdkError::CommandError("Formatting failed".to_string()));
        }
    }

    Ok(())
}

/// Collect all Rust source files in the project
fn collect_rust_sources(sdk: &NuvaSdk) -> Result<Vec<String>, SdkError> {
    let workspace = sdk.workspace()
        .ok_or_else(|| SdkError::WorkspaceNotFound("No workspace loaded".to_string()))?;

    let root = workspace.root();
    let mut sources = Vec::new();

    collect_sources_recursive(root, &mut sources)?;

    Ok(sources)
}

/// Recursively collect .rs files
fn collect_sources_recursive(dir: &std::path::Path, sources: &mut Vec<String>) -> Result<(), SdkError> {
    if !dir.exists() {
        return Ok(());
    }

    let entries = std::fs::read_dir(dir)
        .map_err(|e| SdkError::IoError(e.to_string()))?;

    for entry in entries {
        let entry = entry.map_err(|e| SdkError::IoError(e.to_string()))?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if dir_name == "target" || dir_name == ".git" || dir_name == "node_modules" {
                continue;
            }

            collect_sources_recursive(&path, sources)?;
        } else if path.extension().map(|e| e == "rs").unwrap_or(false) {
            if let Some(path_str) = path.to_str() {
                sources.push(path_str.to_string());
            }
        }
    }

    Ok(())
}
