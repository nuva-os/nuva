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

// ! packetdata

use std::collections::HashMap;
use std::path::PathBuf;

/// packetdata
#[derive(Debug, Clone)]
pub struct Package {
 /// packetname
 pub name: String,
 /// version
 pub version: Version,
 /// description
 pub description: Option<String>,
 /// makeer
 pub authors: Vec<String>,
 /// License
 pub license: Option<String>,
 /// dependency
 pub dependencies: Vec<Dependency>,
 /// opendependency
 pub dev_dependencies: Vec<Dependency>,
 /// builddependency
 pub build_dependencies: Vec<Dependency>,
 /// ity
 pub features: HashMap<String, Vec<String>>,
 /// defaultity
 pub default_features: Vec<String>,
 /// target
 pub targets: Vec<Target>,
 /// packetPath
 pub path: Option<PathBuf>,
 /// packagedata
 pub data: Option<Vec<u8>>,
}

impl Package {
 /// secondary Nuva.toml load
 pub fn from_file(path: &PathBuf) -> Result<Self, toml::de::Error> {
 let content = std::fs::read_to_string(path).unwrap_or_default();
 Self::from_toml(&content)
 }

 /// secondary TOML Stringparse
 pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
 let config: PackageToml = toml::from_str(content)?;
 Ok(config.into_package())
 }
}

/// languagemeaningversion
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
 pub major: u32,
 pub minor: u32,
 pub patch: u32,
 pub pre: Option<String>,
 pub build: Option<String>,
}

impl Version {
 pub fn new(major: u32, minor: u32, patch: u32) -> Self {
 Self {
 major,
 minor,
 patch,
 pre: None,
 build: None,
 }
 }

 pub fn parse(s: &str) -> Option<Self> {
 let parts: Vec<&str> = s.split('.').collect();
 if parts.len() < 3 {
 return None;
 }
 
 Some(Self {
 major: parts[0].parse().ok()?,
 minor: parts[1].parse().ok()?,
 patch: parts[2].parse().ok()?,
 pre: None,
 build: None,
 })
 }
}

impl std::fmt::Display for Version {
 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
 write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
 if let Some(ref pre) = self.pre {
 write!(f, "-{}", pre)?;
 }
 if let Some(ref build) = self.build {
 write!(f, "+{}", build)?;
 }
 Ok(())
 }
}

/// dependency
#[derive(Debug, Clone)]
pub struct Dependency {
 /// packetname
 pub name: String,
 /// versionwant
 pub version_req: VersionReq,
 /// comesource
 pub source: DependencySource,
 /// ity
 pub features: Vec<String>,
 /// optional
 pub optional: bool,
}

/// versionwant
#[derive(Debug, Clone)]
pub struct VersionReq {
 pub comparator: Comparator,
 pub version: Version,
}

#[derive(Debug, Clone, Copy)]
pub enum Comparator {
 Exact, // =
 Minimum, // >=
 Caret, // ^
 Tilde, // ~
 Any, // *
}

/// dependencycomesource
#[derive(Debug, Clone)]
pub enum DependencySource {
 /// registerform
 Registry(String),
 /// Git repolibrary
 Git {
 url: String,
 rev: Option<String>,
 tag: Option<String>,
 branch: Option<String>,
 },
 /// LocalPath
 Path(PathBuf),
}

/// target
#[derive(Debug, Clone)]
pub struct Target {
 /// targetclasstype
 pub kind: TargetKind,
 /// name
 pub name: String,
 /// Path
 pub path: PathBuf,
 /// dependency
 pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum TargetKind {
 Lib,
 Bin,
 Test,
 Bench,
 Example,
}

/// packetsummarywant(usesearchresult)
#[derive(Debug, Clone)]
pub struct PackageSummary {
 pub name: String,
 pub version: String,
 pub description: Option<String>,
}

// TOML configurestruct
#[derive(Debug, serde::Deserialize)]
struct PackageToml {
 package: PackageSection,
 dependencies: Option<HashMap<String, String>>,
 #[serde(rename = "dev-dependencies")]
 dev_dependencies: Option<HashMap<String, String>>,
 features: Option<HashMap<String, Vec<String>>>,
 lib: Option<TargetSection>,
 bin: Option<Vec<TargetSection>>,
}

#[derive(Debug, serde::Deserialize)]
struct PackageSection {
 name: String,
 version: String,
 description: Option<String>,
 authors: Option<Vec<String>>,
 license: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct TargetSection {
 name: Option<String>,
 path: Option<String>,
}

impl PackageToml {
 fn into_package(self) -> Package {
 Package {
 name: self.package.name,
 version: Version::parse(&self.package.version).unwrap_or_else(|| Version::new(0, 1, 0)),
 description: self.package.description,
 authors: self.package.authors.unwrap_or_default(),
 license: self.package.license,
 dependencies: parse_deps(self.dependencies),
 dev_dependencies: parse_deps(self.dev_dependencies),
 build_dependencies: vec![],
 features: self.features.unwrap_or_default(),
 default_features: vec![],
 targets: vec![],
 path: None,
 data: None,
 }
 }
}

fn parse_deps(deps: Option<HashMap<String, String>>) -> Vec<Dependency> {
 deps.map(|d| d.into_iter().map(|(name, version)| Dependency {
 name,
 version_req: VersionReq {
 comparator: Comparator::Caret,
 version: Version::parse(&version).unwrap_or_else(|| Version::new(0, 1, 0)),
 },
 source: DependencySource::Registry("default".to_string()),
 features: vec![],
 optional: false,
 }).collect()).unwrap_or_default()
}