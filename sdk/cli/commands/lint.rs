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

// ! Lint source code using clippy and custom rules

use crate::NuvaSdk;
use crate::error::SdkError;
use crate::cli::args::LintCommand;
use crate::cli::output;
use alloc::vec;
use alloc::format;

/// Execute lint command
pub fn execute(sdk: &mut NuvaSdk, cmd: LintCommand) -> Result<(), SdkError> {
    if cmd.fix {
        output::info("Running linter and fixing issues...");
    } else {
        output::info("Running linter...");
    }

    let target_config = sdk.config();
    let target_triple = &target_config.target.triple;

    let mut clippy_args = vec![
        "clippy".to_string(),
        "--target".to_string(),
        target_triple.clone(),
    ];

    if cmd.fix {
        clippy_args.push("--fix".to_string());
        clippy_args.push("--allow-dirty".to_string());
    }

    if !cmd.files.is_empty() {
        for file in &cmd.files {
            clippy_args.push(file.clone());
        }
    }

    clippy_args.push("--".to_string());
    clippy_args.push("-D".to_string());
    clippy_args.push("warnings".to_string());

    let result = std::process::Command::new("cargo")
        .args(&clippy_args)
        .output()
        .map_err(|e| SdkError::ExecutionError(format!("Failed to run clippy: {}", e)))?;

    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);

    let mut warning_count = 0usize;
    let mut error_count = 0usize;

    for line in stderr.lines() {
        if line.contains("warning:") {
            warning_count += 1;
        } else if line.contains("error:") {
            error_count += 1;
        }
    }

    if !stdout.is_empty() {
        println!("{}", stdout);
    }

    if !stderr.is_empty() {
        eprintln!("{}", stderr);
    }

    if error_count > 0 {
        output::error(&format!("Found {} error(s) and {} warning(s)", error_count, warning_count));
        return Err(SdkError::CommandError(format!(
            "Clippy found {} error(s)",
            error_count
        )));
    } else if warning_count > 0 {
        output::warning(&format!("Found {} warning(s)", warning_count));
    }

    output::success("Linting completed");

    Ok(())
}
