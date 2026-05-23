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

//! Build command

use crate::NuvaSdk;
use crate::error::SdkError;
use crate::cli::args::BuildCommand;
use crate::cli::output;

/// Execute build command
pub fn execute(sdk: &mut NuvaSdk, cmd: BuildCommand) -> Result<(), SdkError> {
    let mode = if cmd.release { "release" } else { "debug" };
    output::info(&format!("Building in {} mode...", mode));
    
    if let Some(target) = &cmd.target {
        output::info(&format!("Target: {}", target));
    }
    
    if !cmd.features.is_empty() {
        output::info(&format!("Features: {}", cmd.features.join(", ")));
    }
    
    // 1. Parse build configuration
    let config = sdk.load_build_config()?;
    output::debug(&format!("Build configuration loaded: {:?}", config));
    
    // 2. Determine optimization level
    let opt_level = if cmd.release {
        cmd.opt_level.unwrap_or(2)
    } else {
        cmd.opt_level.unwrap_or(0)
    };
    output::debug(&format!("Optimization level: {}", opt_level));
    
    // 3. Determine parallel jobs
    let jobs = cmd.jobs.unwrap_or_else(|| num_cpus::get());
    output::debug(&format!("Parallel jobs: {}", jobs));
    
    // 4. Load project manifest
    let manifest = sdk.load_manifest()?;
    output::info(&format!("Building {} v{}", manifest.name, manifest.version));
    
    // 5. Collect source files
    let sources = sdk.collect_sources()?;
    output::info(&format!("Found {} source files", sources.len()));
    
    // 6. Check dependencies
    let deps = sdk.resolve_dependencies()?;
    if !deps.is_empty() {
        output::info(&format!("Resolved {} dependencies", deps.len()));
    }
    
    // 7. Compile source files
    output::info("Compiling source files...");
    let compilation_start = std::time::Instant::now();
    
    let mut compiled_units = Vec::new();
    for (i, source) in sources.iter().enumerate() {
        output::progress(i + 1, sources.len(), &format!("Compiling {}", source.display()));
        
        let compiled = sdk.compile_source(source, opt_level)?;
        compiled_units.push(compiled);
    }
    output::clear_line();
    
    let compilation_time = compilation_start.elapsed();
    output::debug(&format!("Compilation completed in {:?}", compilation_time));
    
    // 8. Link compiled units
    output::info("Linking...");
    let linking_start = std::time::Instant::now();
    
    let output_path = sdk.link_compiled_units(&compiled_units, cmd.release, cmd.target.as_deref())?;
    
    let linking_time = linking_start.elapsed();
    output::debug(&format!("Linking completed in {:?}", linking_time));
    
    // 9. Generate debug info if needed
    if cmd.debug_info {
        output::info("Generating debug information...");
        sdk.generate_debug_info(&output_path)?;
    }
    
    // 10. Generate output
    let output_size = std::fs::metadata(&output_path)?.len();
    let output_size_mb = output_size as f64 / (1024.0 * 1024.0);
    
    output::success(&format!(
        "Build completed successfully: {} ({:.2} MB)",
        output_path.display(),
        output_size_mb
    ));
    
    // Print timing information
    let total_time = compilation_time + linking_time;
    output::debug(&format!("Total build time: {:?}", total_time));
    
    Ok(())
}
