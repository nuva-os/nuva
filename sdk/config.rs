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

// ! SDK configuremodule

use std::path::PathBuf;

/// SDK configure
#[derive(Debug, Clone)]
pub struct SdkConfig {
 /// SDK version
 pub version: String,
 /// targetArchitecture
 pub target: TargetConfig,
 /// ToollinkPath
 pub toolchain_path: PathBuf,
 /// cachedirectory
 pub cache_dir: PathBuf,
 /// configuredirectory
 pub config_dir: PathBuf,
 /// loglevelcategory
 pub log_level: LogLevel,
 /// Paralleltasknumber
 pub parallel_jobs: usize,
 /// networkconfigure
 pub network: NetworkConfig,
}

impl Default for SdkConfig {
 fn default() -> Self {
 let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
 let nuva_home = home.join(".nuva");
 
 Self {
 version: crate::SDK_VERSION.to_string(),
 target: TargetConfig::default(),
 toolchain_path: nuva_home.join("toolchain"),
 cache_dir: nuva_home.join("cache"),
 config_dir: nuva_home.join("config"),
 log_level: LogLevel::Info,
 parallel_jobs: num_cpus::get(),
 network: NetworkConfig::default(),
 }
 }
}

impl SdkConfig {
 /// createconfigurebuilddevice
 pub fn builder() -> SdkConfigBuilder {
 SdkConfigBuilder::default()
 }

 /// secondaryfileloadconfigure
 pub fn from_file(path: &PathBuf) -> Result<Self, ConfigError> {
 let content = std::fs::read_to_string(path)
 .map_err(|e| ConfigError::IoError(e.to_string()))?;
 
 let config: SdkConfigToml = toml::from_str(&content)
 .map_err(|e| ConfigError::ParseError(e.to_string()))?;
 
 Ok(config.into_sdk_config())
 }

 /// saveconfiguretofile
 pub fn save_to_file(&self, path: &PathBuf) -> Result<(), ConfigError> {
 let config = SdkConfigToml::from_sdk_config(self);
 let content = toml::to_string_pretty(&config)
 .map_err(|e| ConfigError::ParseError(e.to_string()))?;
 
 std::fs::write(path, content)
 .map_err(|e| ConfigError::IoError(e.to_string()))?;
 
 Ok(())
 }

 /// getpacketcachePath
 pub fn package_cache_path(&self) -> PathBuf {
 self.cache_dir.join("packages")
 }

 /// getbuildcachePath
 pub fn build_cache_path(&self) -> PathBuf {
 self.cache_dir.join("build")
 }
}

/// configurebuilddevice
#[derive(Debug, Default)]
pub struct SdkConfigBuilder {
 config: SdkConfig,
}

impl SdkConfigBuilder {
 pub fn target(mut self, target: TargetConfig) -> Self {
 self.config.target = target;
 self
 }

 pub fn toolchain_path(mut self, path: impl Into<PathBuf>) -> Self {
 self.config.toolchain_path = path.into();
 self
 }

 pub fn cache_dir(mut self, dir: impl Into<PathBuf>) -> Self {
 self.config.cache_dir = dir.into();
 self
 }

 pub fn log_level(mut self, level: LogLevel) -> Self {
 self.config.log_level = level;
 self
 }

 pub fn parallel_jobs(mut self, jobs: usize) -> Self {
 self.config.parallel_jobs = jobs;
 self
 }

 pub fn build(self) -> SdkConfig {
 self.config
 }
}

/// targetconfigure
#[derive(Debug, Clone)]
pub struct TargetConfig {
 /// targetArchitecture
 pub arch: TargetArch,
 /// targetPlatform
 pub platform: TargetPlatform,
 /// targetTuple
 pub triple: String,
}

impl Default for TargetConfig {
 fn default() -> Self {
 Self {
 arch: TargetArch::Arm64,
 platform: TargetPlatform::Kirin9020,
 triple: "aarch64-nuva".to_string(),
 }
 }
}

impl TargetConfig {
 pub fn for_arm64() -> Self {
 Self {
 arch: TargetArch::Arm64,
 platform: TargetPlatform::Kirin9020,
 triple: "aarch64-nuva".to_string(),
 }
 }

 pub fn for_x64() -> Self {
 Self {
 arch: TargetArch::X64,
 platform: TargetPlatform::IntelCore,
 triple: "x86_64-nuva".to_string(),
 }
 }
}

/// targetArchitecture
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetArch {
 /// ARM64
 Arm64,
 /// x86-64
 X64,
}

impl TargetArch {
 pub fn as_str(&self) -> &'static str {
 match self {
 TargetArch::Arm64 => "arm64",
 TargetArch::X64 => "x64",
 }
 }
}

/// targetPlatform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetPlatform {
 Kirin9020,
 Snapdragon8Gen4,
 IntelCore,
 AmdRyzen,
}

impl TargetPlatform {
 pub fn as_str(&self) -> &'static str {
 match self {
 TargetPlatform::Kirin9020 => "kirin9020",
 TargetPlatform::Snapdragon8Gen4 => "snapdragon8gen4",
 TargetPlatform::IntelCore => "intel-core",
 TargetPlatform::AmdRyzen => "amd-ryzen",
 }
 }
}

/// loglevelcategory
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
 Trace,
 Debug,
 Info,
 Warn,
 Error,
}

impl LogLevel {
 pub fn as_str(&self) -> &'static str {
 match self {
 LogLevel::Trace => "trace",
 LogLevel::Debug => "debug",
 LogLevel::Info => "info",
 LogLevel::Warn => "warn",
 LogLevel::Error => "error",
 }
 }
}

/// networkconfigure
#[derive(Debug, Clone)]
pub struct NetworkConfig {
 /// packetrepolibrary URL
 pub registry_url: String,
 /// Proxysettings
 pub proxy: Option<String>,
 /// timeouttime(second)
 pub timeout_secs: u64,
 /// retrytimenumber
 pub retries: usize,
}

impl Default for NetworkConfig {
 fn default() -> Self {
 Self {
 registry_url: "https://registry.nuva.io".to_string(),
 proxy: None,
 timeout_secs: 30,
 retries: 3,
 }
 }
}

/// configureerror
#[derive(Debug)]
pub enum ConfigError {
 IoError(String),
 ParseError(String),
}

impl std::fmt::Display for ConfigError {
 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
 match self {
 ConfigError::IoError(msg) => write!(f, "IO error: {}", msg),
 ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
 }
 }
}

impl std::error::Error for ConfigError {}

/// TOML configurestruct
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SdkConfigToml {
 version: String,
 target: TargetConfigToml,
 toolchain_path: String,
 cache_dir: String,
 config_dir: String,
 log_level: String,
 parallel_jobs: usize,
 network: NetworkConfigToml,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TargetConfigToml {
 arch: String,
 platform: String,
 triple: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct NetworkConfigToml {
 registry_url: String,
 proxy: Option<String>,
 timeout_secs: u64,
 retries: usize,
}

impl SdkConfigToml {
 fn into_sdk_config(self) -> SdkConfig {
 SdkConfig {
 version: self.version,
 target: TargetConfig {
 arch: match self.target.arch.as_str() {
 "x64" => TargetArch::X64,
 _ => TargetArch::Arm64,
 },
 platform: match self.target.platform.as_str() {
 "snapdragon8gen4" => TargetPlatform::Snapdragon8Gen4,
 "intel-core" => TargetPlatform::IntelCore,
 "amd-ryzen" => TargetPlatform::AmdRyzen,
 _ => TargetPlatform::Kirin9020,
 },
 triple: self.target.triple,
 },
 toolchain_path: PathBuf::from(self.toolchain_path),
 cache_dir: PathBuf::from(self.cache_dir),
 config_dir: PathBuf::from(self.config_dir),
 log_level: match self.log_level.as_str() {
 "trace" => LogLevel::Trace,
 "debug" => LogLevel::Debug,
 "warn" => LogLevel::Warn,
 "error" => LogLevel::Error,
 _ => LogLevel::Info,
 },
 parallel_jobs: self.parallel_jobs,
 network: NetworkConfig {
 registry_url: self.network.registry_url,
 proxy: self.network.proxy,
 timeout_secs: self.network.timeout_secs,
 retries: self.network.retries,
 },
 }
 }

 fn from_sdk_config(config: &SdkConfig) -> Self {
 Self {
 version: config.version.clone(),
 target: TargetConfigToml {
 arch: config.target.arch.as_str().to_string(),
 platform: config.target.platform.as_str().to_string(),
 triple: config.target.triple.clone(),
 },
 toolchain_path: config.toolchain_path.to_string_lossy().to_string(),
 cache_dir: config.cache_dir.to_string_lossy().to_string(),
 config_dir: config.config_dir.to_string_lossy().to_string(),
 log_level: config.log_level.as_str().to_string(),
 parallel_jobs: config.parallel_jobs,
 network: NetworkConfigToml {
 registry_url: config.network.registry_url.clone(),
 proxy: config.network.proxy.clone(),
 timeout_secs: config.network.timeout_secs,
 retries: config.network.retries,
 },
 }
 }
}