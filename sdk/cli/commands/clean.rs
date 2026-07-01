/*
 * Nuva OS
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License at
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

// ! Clean build artifacts

use crate::NuvaSdk;
use crate::error::SdkError;
use crate::cli::args::CleanCommand;
use crate::cli::output;
use alloc::format;

/// Execute clean command
pub fn execute(sdk: &mut NuvaSdk, cmd: CleanCommand) -> Result<(), SdkError> {
    if cmd.all {
        output::info("Cleaning all build artifacts...");
    } else {
        output::info("Cleaning build artifacts...");
    }

    let workspace = sdk.workspace()
        .ok_or_else(|| SdkError::WorkspaceNotFound("No workspace loaded".to_string()))?;

    let root = workspace.root();
    let target_dir = root.join("target");

    if target_dir.exists() {
        if cmd.all {
            remove_dir_recursive(&target_dir)?;
            output::info("Removed entire target directory");
        } else {
            if let Some(ref target) = cmd.target {
                let target_specific = target_dir.join(target);
                if target_specific.exists() {
                    remove_dir_recursive(&target_specific)?;
                    output::info(&format!("Removed target/{}", target));
                } else {
                    output::warning(&format!("target/{} does not exist", target));
                }
            } else {
                for subdir in &["debug", "release"] {
                    let dir = target_dir.join(subdir);
                    if dir.exists() {
                        remove_dir_recursive(&dir)?;
                        output::info(&format!("Removed target/{}", subdir));
                    }
                }
            }
        }
    } else {
        output::info("No target directory found, nothing to clean");
    }

    if cmd.all {
        let cache_dir = root.join(".cache");
        if cache_dir.exists() {
            remove_dir_recursive(&cache_dir)?;
            output::info("Removed .cache directory");
        }
    }

    output::success("Clean completed");
    Ok(())
}

/// Remove directory recursively
fn remove_dir_recursive(path: &std::path::Path) -> Result<(), SdkError> {
    std::fs::remove_dir_all(path)
        .map_err(|e| SdkError::IoError(format!("Failed to remove {}: {}", path.display(), e)))
}
