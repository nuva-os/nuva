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

// ! Generate documentation

use crate::NuvaSdk;
use crate::error::SdkError;
use crate::cli::args::DocCommand;
use crate::cli::output;
use alloc::vec;
use alloc::format;

/// Execute documentation generation command
pub fn execute(sdk: &mut NuvaSdk, cmd: DocCommand) -> Result<(), SdkError> {
    output::info("Generating documentation...");

    let mut doc_args = vec!["doc".to_string(), "--no-deps".to_string()];

    if let Some(ref output_dir) = cmd.output {
        doc_args.push("--target-dir".to_string());
        doc_args.push(output_dir.clone());
    }

    let result = std::process::Command::new("cargo")
        .args(&doc_args)
        .output()
        .map_err(|e| SdkError::ExecutionError(format!("Failed to run cargo doc: {}", e)))?;

    let stderr = String::from_utf8_lossy(&result.stderr);

    if !result.status.success() {
        if !stderr.is_empty() {
            eprintln!("{}", stderr);
        }
        return Err(SdkError::CommandError("Documentation generation failed".to_string()));
    }

    if !stderr.is_empty() {
        let warning_count = stderr.lines().filter(|l| l.contains("warning:")).count();
        if warning_count > 0 {
            output::warning(&format!("{} documentation warning(s)", warning_count));
        }
    }

    output::success("Documentation generated successfully");

    if cmd.open {
        output::info("Opening documentation in browser...");
        let doc_path = if let Some(ref output_dir) = cmd.output {
            std::path::PathBuf::from(output_dir).join("doc")
        } else {
            std::path::PathBuf::from("target/doc")
        };

        let index_html = doc_path.join("index.html");
        if index_html.exists() {
            open_in_browser(&index_html)?;
        } else {
            output::warning("Documentation index not found");
        }
    }

    Ok(())
}

/// Open path in default browser
fn open_in_browser(path: &std::path::Path) -> Result<(), SdkError> {
    let url = format!("file:///{}", path.display().replace('\\', "/"));

    #[cfg(target_os = "windows")]
    let result = std::process::Command::new("cmd")
        .args(&["/C", "start", &url])
        .spawn();

    #[cfg(target_os = "macos")]
    let result = std::process::Command::new("open")
        .arg(&url)
        .spawn();

    #[cfg(target_os = "linux")]
    let result = std::process::Command::new("xdg-open")
        .arg(&url)
        .spawn();

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let result: Result<std::process::Child, std::io::Error> = Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "Cannot open browser on this platform",
    ));

    result
        .map_err(|e| SdkError::ExecutionError(format!("Failed to open browser: {}", e)))?;

    Ok(())
}
