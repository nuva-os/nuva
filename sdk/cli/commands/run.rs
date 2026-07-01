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

//! Run command

use crate::NuvaSdk;
use crate::error::SdkError;
use crate::cli::args::RunCommand;
use crate::cli::output;
use alloc::format;
use alloc::vec::Vec;

/// Execute run command
pub fn execute(sdk: &mut NuvaSdk, cmd: RunCommand) -> Result<(), SdkError> {
    let mode = if cmd.release { "release" } else { "debug" };
    output::info(&format!("Running in {} mode...", mode));
    
    // 1. Check if build is needed
    let output_path = if cmd.build {
        // Build first
        output::info("Building project before running...");
        let build_cmd = crate::cli::args::BuildCommand {
            release: cmd.release,
            target: cmd.target.clone(),
            features: cmd.features.clone(),
            jobs: None,
            opt_level: None,
            debug_info: false,
        };
        crate::cli::commands::build::execute(sdk, build_cmd)?;
        
        // Get the output path
        let manifest = sdk.load_manifest()?;
        let target_dir = if cmd.release {
            "target/release"
        } else {
            "target/debug"
        };
        let binary_name = format!("{}{}", manifest.name, std::env::consts::EXE_SUFFIX);
        Some(std::path::PathBuf::from(target_dir).join(binary_name))
    } else {
        // Use existing binary
        None
    };
    
    // 2. Determine the executable path
    let executable = if let Some(path) = output_path {
        path
    } else if let Some(ref bin) = cmd.binary {
        std::path::PathBuf::from(bin)
    } else {
        // Find the default binary
        let manifest = sdk.load_manifest()?;
        let target_dir = if cmd.release {
            "target/release"
        } else {
            "target/debug"
        };
        let default_path = std::path::PathBuf::from(target_dir)
            .join(format!("{}{}", manifest.name, std::env::consts::EXE_SUFFIX));
        
        if !default_path.exists() {
            output::warning("Binary not found, building...");
            let build_cmd = crate::cli::args::BuildCommand {
                release: cmd.release,
                target: cmd.target.clone(),
                features: cmd.features.clone(),
                jobs: None,
                opt_level: None,
                debug_info: false,
            };
            crate::cli::commands::build::execute(sdk, build_cmd)?;
        }
        
        default_path
    };
    
    // 3. Check if executable exists
    if !executable.exists() {
        return Err(SdkError::FileNotFound(executable.display().to_string()));
    }
    
    output::info(&format!("Running {}...", executable.display()));
    
    // 4. Prepare environment variables
    let mut env = std::env::vars().collect::<Vec<_>>();
    
    // Add project-specific environment variables
    if let Some(manifest) = sdk.load_manifest().ok() {
        env.push(("NUVA_PROJECT_NAME".to_string(), manifest.name.clone()));
        env.push(("NUVA_PROJECT_VERSION".to_string(), manifest.version.clone()));
    }
    
    // Add mode-specific environment variables
    env.push(("NUVA_BUILD_MODE".to_string(), mode.to_string()));
    
    // Add custom environment variables from command
    for (key, value) in &cmd.env {
        env.push((key.clone(), value.clone()));
    }
    
    // 5. Run the executable
    let run_start = std::time::Instant::now();
    
    let mut child = std::process::Command::new(&executable)
        .args(&cmd.args)
        .envs(env)
        .current_dir(sdk.workspace_path())
        .spawn()
        .map_err(|e| SdkError::ExecutionError(format!("Failed to spawn process: {}", e)))?;
    
    // 6. Wait for completion
    let status = child.wait()
        .map_err(|e| SdkError::ExecutionError(format!("Failed to wait for process: {}", e)))?;
    
    let run_time = run_start.elapsed();
    
    // 7. Report results
    if status.success() {
        output::success(&format!("Program completed successfully in {:?}", run_time));
        
        if let Some(code) = status.code() {
            output::debug(&format!("Exit code: {}", code));
        }
    } else {
        output::error(&format!("Program failed with exit code: {:?}", status.code()));
        
        if !cmd.no_fail {
            return Err(SdkError::ExecutionError(format!(
                "Program exited with non-zero status: {:?}",
                status.code()
            )));
        }
    }
    
    // 8. Print timing information
    output::debug(&format!("Execution time: {:?}", run_time));
    
    Ok(())
}
