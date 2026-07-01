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

// ! buildexecutedevice

use std::path::PathBuf;
use std::process::Command;
use crate::error::SdkError;
use super::config::BuildConfig;
use super::target::TargetKind;
use super::scheduler::BuildNode;
use super::BuildResult;
use alloc::vec;
use alloc::format;
use alloc::vec::Vec;

/// buildexecutedevice
pub struct BuildExecutor {
    /// buildconfigure
    config: BuildConfig,
}

impl BuildExecutor {
    pub fn new(config: &BuildConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// executebuildNode
    pub fn execute(&mut self, node: &BuildNode) -> Result<BuildResult, SdkError> {
        match node.kind {
            TargetKind::Lib => self.compile_library(node),
            TargetKind::Bin => self.compile_binary(node),
            TargetKind::Test => self.compile_test(node),
            TargetKind::Bench => self.compile_benchmark(node),
            TargetKind::Example => self.compile_example(node),
        }
    }

    /// Compile library
    fn compile_library(&mut self, node: &BuildNode) -> Result<BuildResult, SdkError> {
        let start = std::time::Instant::now();
        let output = self.config.out_dir.join(format!("lib{}.a", node.name));
        
        log_info!("Compiling library: {}", node.name);
        
        // Compile source files
        let object_files = self.compile_sources(&node.path)?;
        
        // Create static library
        if !object_files.is_empty() {
            self.create_static_library(&object_files, &output)?;
        }
        
        let elapsed = start.elapsed().as_millis() as u64;
        
        Ok(BuildResult::Success {
            outputs: vec![output],
            compile_time_ms: elapsed,
        })
    }

    /// Compile binary
    fn compile_binary(&mut self, node: &BuildNode) -> Result<BuildResult, SdkError> {
        let start = std::time::Instant::now();
        let output = self.config.out_dir.join(&node.name);
        
        log_info!("Compiling binary: {}", node.name);
        
        // Compile source files
        let object_files = self.compile_sources(&node.path)?;
        
        // Link
        if !object_files.is_empty() {
            self.link(&object_files, &output)?;
        }
        
        let elapsed = start.elapsed().as_millis() as u64;
        
        Ok(BuildResult::Success {
            outputs: vec![output],
            compile_time_ms: elapsed,
        })
    }

    /// Compile test
    fn compile_test(&mut self, node: &BuildNode) -> Result<BuildResult, SdkError> {
        let start = std::time::Instant::now();
        let output = self.config.out_dir.join(format!("{}-test", node.name));
        
        log_info!("Compiling test: {}", node.name);
        
        let object_files = self.compile_sources(&node.path)?;
        
        if !object_files.is_empty() {
            self.link(&object_files, &output)?;
        }
        
        let elapsed = start.elapsed().as_millis() as u64;
        
        Ok(BuildResult::Success {
            outputs: vec![output],
            compile_time_ms: elapsed,
        })
    }

    /// Compile benchmark
    fn compile_benchmark(&mut self, node: &BuildNode) -> Result<BuildResult, SdkError> {
        let start = std::time::Instant::now();
        let output = self.config.out_dir.join(format!("{}-bench", node.name));
        
        log_info!("Compiling benchmark: {}", node.name);
        
        let object_files = self.compile_sources(&node.path)?;
        
        if !object_files.is_empty() {
            self.link(&object_files, &output)?;
        }
        
        let elapsed = start.elapsed().as_millis() as u64;
        
        Ok(BuildResult::Success {
            outputs: vec![output],
            compile_time_ms: elapsed,
        })
    }

    /// Compile example
    fn compile_example(&mut self, node: &BuildNode) -> Result<BuildResult, SdkError> {
        let start = std::time::Instant::now();
        let output = self.config.out_dir.join(format!("{}-example", node.name));
        
        log_info!("Compiling example: {}", node.name);
        
        let object_files = self.compile_sources(&node.path)?;
        
        if !object_files.is_empty() {
            self.link(&object_files, &output)?;
        }
        
        let elapsed = start.elapsed().as_millis() as u64;
        
        Ok(BuildResult::Success {
            outputs: vec![output],
            compile_time_ms: elapsed,
        })
    }

    /// compilesourcefile
    fn compile_sources(&self, path: &PathBuf) -> Result<Vec<PathBuf>, SdkError> {
        let mut object_files = vec![];
        
        if path.is_file() {
            let output = self.compile_file(path)?;
            object_files.push(output);
        } else if path.is_dir() {
            for entry in walkdir::WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    if let Some(ext) = file_path.extension() {
                        if ext == "nuva" || ext == "rs" {
                            let output = self.compile_file(file_path)?;
                            object_files.push(output);
                        }
                    }
                }
            }
        }
        
        Ok(object_files)
    }

    /// Compile a single file
    fn compile_file(&self, path: &PathBuf) -> Result<PathBuf, SdkError> {
        let output = self.config.out_dir
            .join("deps")
            .join(path.file_name().unwrap())
            .with_extension("o");
        
        std::fs::create_dir_all(output.parent().unwrap())
            .map_err(|e| SdkError::IoError(e.to_string()))?;
        
        // Determine file type and invoke appropriate compiler
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        
        match ext {
            "nuva" => self.compile_nuva_file(path, &output)?,
            "rs" => self.compile_rust_file(path, &output)?,
            "c" => self.compile_c_file(path, &output)?,
            "cpp" | "cc" | "cxx" => self.compile_cpp_file(path, &output)?,
            _ => {
                return Err(SdkError::BuildError(format!(
                    "Unsupported file type: {}", ext
                )));
            }
        }
        
        log_info!("Compiled: {} -> {}", path.display(), output.display());
        
        Ok(output)
    }

    /// Compile Nuva source file
    fn compile_nuva_file(&self, input: &PathBuf, output: &PathBuf) -> Result<(), SdkError> {
        log_debug!("Compiling Nuva file: {}", input.display());
        
        // Build compiler command
        let mut cmd = Command::new("nuvac");
        
        // Add optimization level
        let opt_level = match self.config.opt_level {
            0 => "-O0",
            1 => "-O1",
            2 => "-O2",
            3 => "-O3",
            _ => "-O2",
        };
        cmd.arg(opt_level);
        
        // Add target architecture
        cmd.arg("--target").arg(&self.config.target.triple);
        
        // Add debug info if enabled
        if self.config.debug {
            cmd.arg("-g");
        }
        
        // Add include directories
        for include_dir in &self.config.include_dirs {
            cmd.arg("-I").arg(include_dir);
        }
        
        // Add defines
        for (key, value) in &self.config.defines {
            if let Some(val) = value {
                cmd.arg(format!("-D{}={}", key, val));
            } else {
                cmd.arg(format!("-D{}", key));
            }
        }
        
        // Add input and output
        cmd.arg("-o").arg(output);
        cmd.arg(input);
        
        // Execute compiler
        let result = cmd.output()
            .map_err(|e| SdkError::BuildError(format!("Failed to execute compiler: {}", e)))?;
        
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SdkError::CompilationError(format!(
                "Compilation failed for {}: {}",
                input.display(),
                stderr
            )));
        }
        
        Ok(())
    }

    /// Compile Rust source file
    fn compile_rust_file(&self, input: &PathBuf, output: &PathBuf) -> Result<(), SdkError> {
        log_debug!("Compiling Rust file: {}", input.display());
        
        // Use rustc for Rust files
        let mut cmd = Command::new("rustc");
        
        // Add optimization level
        let opt_level = match self.config.opt_level {
            0 => "0",
            1 => "1",
            2 => "2",
            3 => "3",
            _ => "2",
        };
        cmd.arg(format!("-Copt-level={}", opt_level));
        
        // Add target architecture
        cmd.arg("--target").arg(&self.config.target.triple);
        
        // Add debug info if enabled
        if self.config.debug {
            cmd.arg("-g");
        }
        
        // Emit object file
        cmd.arg("--emit=obj");
        cmd.arg("-o").arg(output);
        cmd.arg(input);
        
        // Execute compiler
        let result = cmd.output()
            .map_err(|e| SdkError::BuildError(format!("Failed to execute rustc: {}", e)))?;
        
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SdkError::CompilationError(format!(
                "Compilation failed for {}: {}",
                input.display(),
                stderr
            )));
        }
        
        Ok(())
    }

    /// Compile C source file
    fn compile_c_file(&self, input: &PathBuf, output: &PathBuf) -> Result<(), SdkError> {
        log_debug!("Compiling C file: {}", input.display());
        
        let mut cmd = Command::new("gcc");
        
        // Add optimization level
        let opt_level = match self.config.opt_level {
            0 => "-O0",
            1 => "-O1",
            2 => "-O2",
            3 => "-O3",
            _ => "-O2",
        };
        cmd.arg(opt_level);
        
        // Add target architecture
        cmd.arg("-march=native");
        
        // Add debug info if enabled
        if self.config.debug {
            cmd.arg("-g");
        }
        
        // Add include directories
        for include_dir in &self.config.include_dirs {
            cmd.arg("-I").arg(include_dir);
        }
        
        // Add defines
        for (key, value) in &self.config.defines {
            if let Some(val) = value {
                cmd.arg(format!("-D{}={}", key, val));
            } else {
                cmd.arg(format!("-D{}", key));
            }
        }
        
        // Emit object file
        cmd.arg("-c").arg("-o").arg(output);
        cmd.arg(input);
        
        // Execute compiler
        let result = cmd.output()
            .map_err(|e| SdkError::BuildError(format!("Failed to execute gcc: {}", e)))?;
        
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SdkError::CompilationError(format!(
                "Compilation failed for {}: {}",
                input.display(),
                stderr
            )));
        }
        
        Ok(())
    }

    /// Compile C++ source file
    fn compile_cpp_file(&self, input: &PathBuf, output: &PathBuf) -> Result<(), SdkError> {
        log_debug!("Compiling C++ file: {}", input.display());
        
        let mut cmd = Command::new("g++");
        
        // Add optimization level
        let opt_level = match self.config.opt_level {
            0 => "-O0",
            1 => "-O1",
            2 => "-O2",
            3 => "-O3",
            _ => "-O2",
        };
        cmd.arg(opt_level);
        
        // Add target architecture
        cmd.arg("-march=native");
        
        // Add debug info if enabled
        if self.config.debug {
            cmd.arg("-g");
        }
        
        // Add include directories
        for include_dir in &self.config.include_dirs {
            cmd.arg("-I").arg(include_dir);
        }
        
        // Add defines
        for (key, value) in &self.config.defines {
            if let Some(val) = value {
                cmd.arg(format!("-D{}={}", key, val));
            } else {
                cmd.arg(format!("-D{}", key));
            }
        }
        
        // Emit object file
        cmd.arg("-c").arg("-o").arg(output);
        cmd.arg(input);
        
        // Execute compiler
        let result = cmd.output()
            .map_err(|e| SdkError::BuildError(format!("Failed to execute g++: {}", e)))?;
        
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SdkError::CompilationError(format!(
                "Compilation failed for {}: {}",
                input.display(),
                stderr
            )));
        }
        
        Ok(())
    }

    /// Create static library
    fn create_static_library(&self, objects: &[PathBuf], output: &PathBuf) -> Result<(), SdkError> {
        log_debug!("Creating static library: {}", output.display());
        
        std::fs::create_dir_all(output.parent().unwrap())
            .map_err(|e| SdkError::IoError(e.to_string()))?;
        
        let mut cmd = Command::new("ar");
        cmd.arg("rcs");
        cmd.arg(output);
        
        for obj in objects {
            cmd.arg(obj);
        }
        
        let result = cmd.output()
            .map_err(|e| SdkError::BuildError(format!("Failed to execute ar: {}", e)))?;
        
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SdkError::LinkError(format!(
                "Failed to create static library: {}",
                stderr
            )));
        }
        
        // Run ranlib for better indexing
        let mut ranlib_cmd = Command::new("ranlib");
        ranlib_cmd.arg(output);
        
        let _ = ranlib_cmd.output();
        
        log_info!("Created static library: {}", output.display());
        
        Ok(())
    }

    /// Link object files
    fn link(&self, objects: &[PathBuf], output: &PathBuf) -> Result<(), SdkError> {
        log_debug!("Linking: {}", output.display());
        
        std::fs::create_dir_all(output.parent().unwrap())
            .map_err(|e| SdkError::IoError(e.to_string()))?;
        
        let mut cmd = Command::new("gcc");
        
        // Add optimization level
        let opt_level = match self.config.opt_level {
            0 => "-O0",
            1 => "-O1",
            2 => "-O2",
            3 => "-O3",
            _ => "-O2",
        };
        cmd.arg(opt_level);
        
        // Add target architecture
        cmd.arg("-march=native");
        
        // Add debug info if enabled
        if self.config.debug {
            cmd.arg("-g");
        }
        
        // Add library search paths
        for lib_dir in &self.config.lib_dirs {
            cmd.arg("-L").arg(lib_dir);
        }
        
        // Add libraries
        for lib in &self.config.libs {
            cmd.arg(format!("-l{}", lib));
        }
        
        // Add object files
        for obj in objects {
            cmd.arg(obj);
        }
        
        // Set output
        cmd.arg("-o").arg(output);
        
        // Execute linker
        let result = cmd.output()
            .map_err(|e| SdkError::BuildError(format!("Failed to execute linker: {}", e)))?;
        
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            return Err(SdkError::LinkError(format!(
                "Linking failed: {}",
                stderr
            )));
        }
        
        log_info!("Linked: {}", output.display());
        
        Ok(())
    }
}