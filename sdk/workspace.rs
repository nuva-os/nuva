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

// ! workmakeemptybetweenmanagementadministrationmodule

use std::path::{Path, PathBuf};
use std::collections::HashMap;
use crate::error::SdkError;

/// workmakeemptybetween
#[derive(Debug)]
pub struct Workspace {
 /// workmakeemptybetweenrootdirectory
 root: PathBuf,
 /// packetlist
 packages: HashMap<String, PackageInfo>,
 /// lockfile
 lock_file: Option<LockFile>,
 /// workmakeemptybetweenconfigure
 config: WorkspaceConfig,
}

impl Workspace {
 /// createnew workmakeemptybetween
 pub fn new(root: PathBuf) -> Result<Self, SdkError> {
 if !root.exists() {
 std::fs::create_dir_all(&root)
 .map_err(|e| SdkError::IoError(e.to_string()))?;
 }

 let config = WorkspaceConfig::default();
 let workspace = Self {
 root,
 packages: HashMap::new(),
 lock_file: None,
 config,
 };

 // createdefaultconfigurefile
 workspace.create_default_config()?;

 Ok(workspace)
 }

 /// loadfiniteworkmakeemptybetween
 pub fn load(root: PathBuf) -> Result<Self, SdkError> {
 if !root.exists() {
 return Err(SdkError::WorkspaceNotFound(root.to_string_lossy().to_string()));
 }

 let config_path = root.join("Nuva.toml");
 let config = if config_path.exists() {
 WorkspaceConfig::from_file(&config_path)?
 } else {
 WorkspaceConfig::default()
 };

 // scanpacket
 let packages = Self::scan_packages(&root)?;

 // loadlockfile
 let lock_path = root.join("nuva.lock");
 let lock_file = if lock_path.exists() {
 Some(LockFile::load(&lock_path)?)
 } else {
 None
 };

 Ok(Self {
 root,
 packages,
 lock_file,
 config,
 })
 }

 /// getrootdirectory
 pub fn root(&self) -> &Path {
 &self.root
 }

 /// getpacketlist
 pub fn packages(&self) -> &HashMap<String, PackageInfo> {
 &self.packages
 }

 /// getlockfile
 pub fn lock_file(&self) -> Option<&LockFile> {
 self.lock_file.as_ref()
 }

 /// getconfigure
 pub fn config(&self) -> &WorkspaceConfig {
 &self.config
 }

 /// addpacket
 pub fn add_package(&mut self, info: PackageInfo) {
 self.packages.insert(info.name.clone(), info);
 }

 /// removepacket
 pub fn remove_package(&mut self, name: &str) -> Option<PackageInfo> {
 self.packages.remove(name)
 }

 /// getpacket
 pub fn get_package(&self, name: &str) -> Option<&PackageInfo> {
 self.packages.get(name)
 }

 /// scanworkmakeemptybetweeninfix packet
 fn scan_packages(root: &Path) -> Result<HashMap<String, PackageInfo>, SdkError> {
 let mut packages = HashMap::new();

 // checkrootdirectoryiswhetherispacket
 let nuva_toml = root.join("Nuva.toml");
 if nuva_toml.exists() {
 if let Ok(info) = PackageInfo::from_file(&nuva_toml) {
 packages.insert(info.name.clone(), info);
 }
 }

 // scanchilddirectory
 for entry in std::fs::read_dir(root)
 .map_err(|e| SdkError::IoError(e.to_string()))?
 {
 let entry = entry.map_err(|e| SdkError::IoError(e.to_string()))?;
 let path = entry.path();
 
 if path.is_dir() {
 let nuva_toml = path.join("Nuva.toml");
 if nuva_toml.exists() {
 if let Ok(info) = PackageInfo::from_file(&nuva_toml) {
 packages.insert(info.name.clone(), info);
 }
 }
 }
 }

 Ok(packages)
 }

 /// createdefaultconfigurefile
 fn create_default_config(&self) -> Result<(), SdkError> {
 let config_path = self.root.join("Nuva.toml");
 if !config_path.exists() {
 self.config.save_to_file(&config_path)?;
 }
 Ok(())
 }

 /// generatelockfile
 pub fn generate_lock_file(&mut self) -> Result<(), SdkError> {
 let lock = LockFile::generate(&self.packages)?;
 let lock_path = self.root.join("nuva.lock");
 lock.save(&lock_path)?;
 self.lock_file = Some(lock);
 Ok(())
 }
}

/// workmakeemptybetweenconfigure
#[derive(Debug, Clone)]
pub struct WorkspaceConfig {
 /// workmakeemptybetweenname
 pub name: String,
 /// workmakeemptybetweenversion
 pub version: String,
 /// memberpacketlist
 pub members: Vec<String>,
 /// defaultmember
 pub default_members: Vec<String>,
 /// arrangementDivide Path
 pub exclude: Vec<String>,
}

impl Default for WorkspaceConfig {
 fn default() -> Self {
 Self {
 name: "nuva-workspace".to_string(),
 version: "0.1.0".to_string(),
 members: vec![".".to_string()],
 default_members: vec![".".to_string()],
 exclude: vec![],
 }
 }
}

impl WorkspaceConfig {
 /// secondaryfileload
 pub fn from_file(path: &Path) -> Result<Self, SdkError> {
 let content = std::fs::read_to_string(path)
 .map_err(|e| SdkError::IoError(e.to_string()))?;
 
 let config: WorkspaceConfigToml = toml::from_str(&content)
 .map_err(|e| SdkError::ParseError(e.to_string()))?;
 
 Ok(Self {
 name: config.name.unwrap_or_else(|| "nuva-workspace".to_string()),
 version: config.version.unwrap_or_else(|| "0.1.0".to_string()),
 members: config.members.unwrap_or_default(),
 default_members: config.default_members.unwrap_or_default(),
 exclude: config.exclude.unwrap_or_default(),
 })
 }

 /// savetofile
 pub fn save_to_file(&self, path: &Path) -> Result<(), SdkError> {
 let config = WorkspaceConfigToml {
 name: Some(self.name.clone()),
 version: Some(self.version.clone()),
 members: Some(self.members.clone()),
 default_members: Some(self.default_members.clone()),
 exclude: Some(self.exclude.clone()),
 };
 
 let content = toml::to_string_pretty(&config)
 .map_err(|e| SdkError::ParseError(e.to_string()))?;
 
 std::fs::write(path, content)
 .map_err(|e| SdkError::IoError(e.to_string()))?;
 
 Ok(())
 }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct WorkspaceConfigToml {
 name: Option<String>,
 version: Option<String>,
 members: Option<Vec<String>>,
 default_members: Option<Vec<String>>,
 exclude: Option<Vec<String>>,
}

/// packetinformation
#[derive(Debug, Clone)]
pub struct PackageInfo {
 /// packetname
 pub name: String,
 /// version
 pub version: String,
 /// Path
 pub path: PathBuf,
 /// dependency
 pub dependencies: Vec<String>,
}

impl PackageInfo {
 /// secondary Nuva.toml fileload
 pub fn from_file(path: &Path) -> Result<Self, SdkError> {
 let content = std::fs::read_to_string(path)
 .map_err(|e| SdkError::IoError(e.to_string()))?;
 
 let config: PackageConfigToml = toml::from_str(&content)
 .map_err(|e| SdkError::ParseError(e.to_string()))?;
 
 Ok(Self {
 name: config.name.unwrap_or_else(|| "unnamed".to_string()),
 version: config.version.unwrap_or_else(|| "0.1.0".to_string()),
 path: path.parent().unwrap_or(Path::new(".")).to_path_buf(),
 dependencies: config.dependencies
 .map(|d| d.keys().cloned().collect())
 .unwrap_or_default(),
 })
 }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PackageConfigToml {
 name: Option<String>,
 version: Option<String>,
 dependencies: Option<HashMap<String, String>>,
}

/// lockfile
#[derive(Debug, Clone)]
pub struct LockFile {
 /// version
 pub version: u32,
 /// packetlog
 pub packages: Vec<LockPackage>,
}

impl LockFile {
 /// generatelockfile
 pub fn generate(packages: &HashMap<String, PackageInfo>) -> Result<Self, SdkError> {
 let lock_packages = packages.values().map(|info| {
 let checksum = calculate_checksum(&info.name, &info.version);
 LockPackage {
 name: info.name.clone(),
 version: info.version.clone(),
 checksum,
 source: "path".to_string(),
 }
 }).collect();

 Ok(Self {
 version: 1,
 packages: lock_packages,
 })
 }

 /// calculatechecksum
 fn calculate_checksum(name: &str, version: &str) -> String {
 use core::hash::{Hash, Hasher};
 use core::collections::hash_map::DefaultHasher;

 let mut hasher = DefaultHasher::new();
 name.hash(&mut hasher);
 version.hash(&mut hasher);
 format!("{:x}", hasher.finish())
 }

 /// secondaryfileload
 pub fn load(path: &Path) -> Result<Self, SdkError> {
 let content = std::fs::read_to_string(path)
 .map_err(|e| SdkError::IoError(e.to_string()))?;
 
 let lock: LockFileToml = toml::from_str(&content)
 .map_err(|e| SdkError::ParseError(e.to_string()))?;
 
 Ok(Self {
 version: lock.version,
 packages: lock.package.into_iter().map(|p| LockPackage {
 name: p.name,
 version: p.version,
 checksum: p.checksum,
 source: p.source,
 }).collect(),
 })
 }

 /// savelockfile
 pub fn save(&self, path: &Path) -> Result<(), SdkError> {
 let lock = LockFileToml {
 version: self.version,
 package: self.packages.iter().map(|p| LockPackageToml {
 name: p.name.clone(),
 version: p.version.clone(),
 checksum: p.checksum.clone(),
 source: p.source.clone(),
 }).collect(),
 };
 
 let content = toml::to_string_pretty(&lock)
 .map_err(|e| SdkError::ParseError(e.to_string()))?;
 
 std::fs::write(path, content)
 .map_err(|e| SdkError::IoError(e.to_string()))?;
 
 Ok(())
 }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct LockFileToml {
 version: u32,
 package: Vec<LockPackageToml>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct LockPackageToml {
 name: String,
 version: String,
 checksum: String,
 source: String,
}

/// lockfileinfix packetlog
#[derive(Debug, Clone)]
pub struct LockPackage {
 pub name: String,
 pub version: String,
 pub checksum: String,
 pub source: String,
}