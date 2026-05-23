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

//! Nuva OS SDK
/*!*/
// ! system openToollinkinterface, packetpacketmanagementadministration、build、debug、performanceanalyzeetcfeature

pub mod config;
pub mod workspace;
pub mod error;
pub mod cli;
pub mod package;
pub mod debug;
pub mod profiler;
pub mod build;

use config::SdkConfig;
use workspace::Workspace;

/// Nuva SDK mainstruct
pub struct NuvaSdk {
 /// SDK configure
 config: SdkConfig,
 /// workmakeemptybetween
 workspace: Option<Workspace>,
}

impl NuvaSdk {
 /// createnew SDK realexample
 pub fn new(config: SdkConfig) -> Self {
 Self {
 config,
 workspace: None,
 }
 }

 /// makeusedefaultconfigurecreate SDK
 pub fn with_defaults() -> Self {
 Self::new(SdkConfig::default())
 }

 /// initializeworkmakeemptybetween
 pub fn init_workspace(&mut self, root: impl Into<std::path::PathBuf>) -> Result<(), error::SdkError> {
 let root_path = root.into();
 let workspace = Workspace::new(root_path)?;
 self.workspace = Some(workspace);
 Ok(())
 }

 /// loadfiniteworkmakeemptybetween
 pub fn load_workspace(&mut self, root: impl Into<std::path::PathBuf>) -> Result<(), error::SdkError> {
 let root_path = root.into();
 let workspace = Workspace::load(root_path)?;
 self.workspace = Some(workspace);
 Ok(())
 }

 /// getconfigure
 pub fn config(&self) -> &SdkConfig {
 &self.config
 }

 /// getworkmakeemptybetween
 pub fn workspace(&self) -> Option<&Workspace> {
 self.workspace.as_ref()
 }

 /// getworkmakeemptybetween(canchange)
 pub fn workspace_mut(&mut self) -> Option<&mut Workspace> {
 self.workspace.as_mut()
 }

 /// run CLI
 pub fn run_cli(&mut self) -> Result<(), error::SdkError> {
 cli::run(self)
 }
}

/// SDK versioninformation
pub const SDK_VERSION: &str = "0.1.0";

/// get SDK version
pub fn version() -> &'static str {
 SDK_VERSION
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_sdk_creation() {
 let sdk = NuvaSdk::with_defaults();
 assert_eq!(sdk.config().version, SDK_VERSION);
 }
}