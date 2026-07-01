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

// ! crosscompileSupport

use std::path::PathBuf;
use crate::error::SdkError;
use super::config::{BuildConfig, TargetConfig};
use alloc::format;
use alloc::vec::Vec;

/// crosscompiledevice
pub struct CrossCompiler {
 /// targetconfigure
 target: TargetConfig,
 /// ToollinkPath
 toolchain_path: PathBuf,
}

impl CrossCompiler {
 pub fn new(target: TargetConfig, toolchain_path: PathBuf) -> Self {
 Self {
 target,
 toolchain_path,
 }
 }

 /// getcompiledevicePath
 pub fn compiler(&self) -> PathBuf {
 self.toolchain_path
 .join("bin")
 .join(format!("{}-clang", self.target.triple))
 }

 /// getlinkdevicePath
 pub fn linker(&self) -> PathBuf {
 self.toolchain_path
 .join("bin")
 .join(format!("{}-ld", self.target.triple))
 }

 /// getcompileflag
 pub fn cflags(&self, config: &BuildConfig) -> Vec<String> {
 let mut flags = self.target.cflags.clone();
 
 // addoptimizationetclevel
 flags.push(config.opt_level.to_flag().to_string());
 
 // adddebuginformation
 if config.debug_info {
 flags.push("-g".to_string());
 }
 
 // addtarget
 flags.push("-target".to_string());
 flags.push(self.target.triple.clone());
 
 flags
 }

 /// getlinkflag
 pub fn ldflags(&self, config: &BuildConfig) -> Vec<String> {
 let mut flags = self.target.ldflags.clone();
 
 // add LTO
 if config.lto {
 flags.push("-flto".to_string());
 }
 
 flags
 }

 /// checkToollinkiswhethercanuse
 pub fn check_toolchain(&self) -> Result<(), SdkError> {
 let compiler = self.compiler();
 if !compiler.exists() {
 return Err(SdkError::BuildError(format!(
 "Compiler not found: {}",
 compiler.display()
 )));
 }
 
 let linker = self.linker();
 if !linker.exists() {
 return Err(SdkError::BuildError(format!(
 "Linker not found: {}",
 linker.display()
 )));
 }
 
 Ok(())
 }
}

/// targetTuple
pub struct TargetTriple {
 pub arch: String,
 pub vendor: String,
 pub os: String,
 pub abi: Option<String>,
}

impl TargetTriple {
 pub fn parse(triple: &str) -> Option<Self> {
 let parts: Vec<&str> = triple.split('-').collect();
 
 if parts.len() < 3 {
 return None;
 }
 
 Some(Self {
 arch: parts[0].to_string(),
 vendor: parts[1].to_string(),
 os: parts[2].to_string(),
 abi: parts.get(3).map(|s| s.to_string()),
 })
 }

 pub fn to_string(&self) -> String {
 if let Some(ref abi) = self.abi {
 format!("{}-{}-{}-{}", self.arch, self.vendor, self.os, abi)
 } else {
 format!("{}-{}-{}", self.arch, self.vendor, self.os)
 }
 }
}

/// Nuva native and Linux compatibility target triples
pub mod targets {
    /// Nuva native target — ARM64 (primary)
    pub const ARM64_NUVA: &str = "aarch64-nuva";
    /// Nuva native target — x86-64 (primary)
    pub const X64_NUVA: &str = "x86_64-nuva";
    /// Compatibility target for Linux userspace testing — ARM64.
    /// Used only for hosted development/testing on Linux systems.
    pub const ARM64_LINUX: &str = "aarch64-unknown-linux-gnu";
    /// Compatibility target for Linux userspace testing — x86-64.
    /// Used only for hosted development/testing on Linux systems.
    pub const X64_LINUX: &str = "x86_64-unknown-linux-gnu";
}