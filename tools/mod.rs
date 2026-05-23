/*
 * Nuva OS - Tools
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


/// Nuva OS openToolcollection
/// packetencodingtranslatedevice、Linker、LSP、Debuggingdevice、PerformanceAnalysisdeviceetcTool

pub mod compiler;
pub mod linker;
pub mod lsp;

// optionalModule
#[cfg(feature = "debugger")]
pub mod debug;

#[cfg(feature = "profiler")]
pub mod profiler;

#[cfg(feature = "package-manager")]
pub mod pm;

#[cfg(feature = "nuvac")]
pub mod nuvac;

#[cfg(feature = "toolchain")]
pub mod toolchain;

/// Toolversion
pub const TOOLS_VERSION: &str = "0.1.0";

/// InitializeToollink
pub fn init_tools() {
 log_info!("Nuva Tools v{}", TOOLS_VERSION);
 
 // Initializeencodingtranslatedevice
 log_info!("Initializing compiler...");
 
 // InitializeLinker
 log_info!("Initializing linker...");
 
 // initialize LSP
 log_info!("Initializing LSP...");
 
 log_info!("Tools initialized");
}

/// Toollinkinformation
pub struct ToolchainInfo {
 /// encodingtranslatedeviceversion
 pub compiler_version: &'static str,
 /// Linkerversion
 pub linker_version: &'static str,
 /// LSP version
 pub lsp_version: &'static str,
 /// targetArchitectureList
 pub target_archs: Vec<&'static str>,
}

impl Default for ToolchainInfo {
 fn default() -> Self {
 Self {
 compiler_version: "0.1.0",
 linker_version: "0.1.0",
 lsp_version: "0.1.0",
 target_archs: vec!["arm64", "x86_64", "loongarch64"],
 }
 }
}

impl ToolchainInfo {
 /// GetToollinkinformation
 pub fn get() -> Self {
 Self::default()
 }

 /// printstampToollinkinformation
 pub fn print(&self) {
 println!("=== Nuva Toolchain ===");
 println!("Compiler: v{}", self.compiler_version);
 println!("Linker: v{}", self.linker_version);
 println!("LSP: v{}", self.lsp_version);
 println!("Targets: {}", self.target_archs.join(", "));
 }
}