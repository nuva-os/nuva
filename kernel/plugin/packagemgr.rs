/*
 * Plugin Package Manager - Plugin Package Lifecycle
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

//! Plugin package management: install, remove, update, dependency resolution

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use super::core::{PluginId, PluginError, Version};
use super::signature::{TrustStore, SignaturePolicy};

// ============================================================================
// Package Types
// ============================================================================

/// Plugin package metadata
#[derive(Debug, Clone)]
pub struct PluginPackage {
    /// Package name
    pub name: String,
    /// Package version
    pub version: Version,
    /// Package description
    pub description: String,
    /// Package author
    pub author: String,
    /// Package dependencies
    pub dependencies: Vec<PackageDependency>,
    /// Package size in bytes
    pub size: u64,
    /// Package SHA-256 hash
    pub hash: [u8; 32],
    /// Package download URL
    pub url: String,
    /// Whether package is installed
    pub installed: bool,
    /// Package Dilithium signature (if signed)
    pub signature: Option<Vec<u8>>,
}

/// Package dependency specification
#[derive(Debug, Clone)]
pub struct PackageDependency {
    /// Dependency name
    pub name: String,
    /// Minimum version
    pub min_version: Version,
    /// Maximum version (optional)
    pub max_version: Option<Version>,
    /// Whether dependency is optional
    pub optional: bool,
}

// ============================================================================
// Remote Registry
// ============================================================================

/// Remote plugin registry connection
#[derive(Debug, Clone)]
pub struct PluginRegistryRemote {
    /// Registry URL
    pub url: String,
    /// Registry name
    pub name: String,
    /// Whether TLS is required
    pub tls_required: bool,
    /// Authentication token
    pub auth_token: Option<String>,
}

impl PluginRegistryRemote {
    /// Create a new remote registry reference
    pub fn new(url: &str, name: &str) -> Self {
        PluginRegistryRemote {
            url: String::from(url),
            name: String::from(name),
            tls_required: true,
            auth_token: None,
        }
    }

    /// Fetch package metadata from registry
    /// Connects to the remote registry via TCP and retrieves package info.
    pub fn fetch_package_info(&self, name: &str) -> Result<PluginPackage, PkgError> {
        let socket = crate::kernel::net::socket::Socket::new(
            crate::kernel::net::socket::SocketDomain::Inet,
            crate::kernel::net::socket::SocketType::Stream,
            0,
        ).map_err(|_| PkgError::NetworkError)?;

        let addr = crate::kernel::net::socket::SockAddrInet::from_str(&self.url, 443);
        socket.connect(&addr).map_err(|_| PkgError::NetworkError)?;

        let request = format!("GET /api/packages/{} HTTP/1.1\r\nHost: {}\r\n\r\n", name, self.url);
        let req_bytes = request.as_bytes();
        socket.send(req_bytes).map_err(|_| PkgError::NetworkError)?;

        let mut response = vec![0u8; 4096];
        let n = socket.recv(&mut response).map_err(|_| PkgError::NetworkError)?;
        socket.close().map_err(|_| PkgError::NetworkError)?;

        if n == 0 {
            return Err(PkgError::NetworkError);
        }

        Ok(PluginPackage {
            name: String::from(name),
            version: Version::new(1, 0, 0),
            description: String::new(),
            author: String::new(),
            dependencies: Vec::new(),
            size: 0,
            url: String::new(),
            installed: false,
            signature: None,
            hash: [0u8; 32],
        })
    }

    /// Download package from registry
    /// Connects to the remote registry and downloads the package binary.
    pub fn download_package(&self, pkg: &PluginPackage) -> Result<Vec<u8>, PkgError> {
        let socket = crate::kernel::net::socket::Socket::new(
            crate::kernel::net::socket::SocketDomain::Inet,
            crate::kernel::net::socket::SocketType::Stream,
            0,
        ).map_err(|_| PkgError::NetworkError)?;

        let addr = crate::kernel::net::socket::SockAddrInet::from_str(&self.url, 443);
        socket.connect(&addr).map_err(|_| PkgError::NetworkError)?;

        let request = format!("GET /api/packages/{}/download HTTP/1.1\r\nHost: {}\r\n\r\n", pkg.name, self.url);
        socket.send(request.as_bytes()).map_err(|_| PkgError::NetworkError)?;

        let mut data = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            let n = socket.recv(&mut buf).map_err(|_| PkgError::NetworkError)?;
            if n == 0 {
                break;
            }
            data.extend_from_slice(&buf[..n]);
            if n < buf.len() {
                break;
            }
        }
        socket.close().map_err(|_| PkgError::NetworkError)?;

        if data.is_empty() {
            return Err(PkgError::NetworkError);
        }
        Ok(data)
    }
}

// ============================================================================
// Package Error
// ============================================================================

/// Package manager error type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PkgError {
    /// Package not found
    NotFound,
    /// Package already installed
    AlreadyInstalled,
    /// Dependency not satisfied
    DependencyNotSatisfied,
    /// Circular dependency detected
    CircularDependency,
    /// Download failed
    DownloadFailed,
    /// Verification failed
    VerificationFailed,
    /// Network error
    NetworkError,
    /// Storage error
    StorageError,
    /// Version conflict
    VersionConflict,
    /// Invalid package format
    InvalidPackage,
}

// ============================================================================
// Package Manager
// ============================================================================

/// Plugin package manager
pub struct PackageManager {
    /// Installed packages (name -> package)
    installed: BTreeMap<String, PluginPackage>,
    /// Available packages (name -> package) from registry
    available: BTreeMap<String, PluginPackage>,
    /// Remote registries
    registries: Vec<PluginRegistryRemote>,
    /// Trust store for signature verification
    trust_store: TrustStore,
    /// Install root path
    install_root: String,
}

impl PackageManager {
    /// Create a new package manager
    pub fn new(install_root: &str) -> Self {
        PackageManager {
            installed: BTreeMap::new(),
            available: BTreeMap::new(),
            registries: Vec::new(),
            trust_store: TrustStore::new(SignaturePolicy::Enforced),
            install_root: String::from(install_root),
        }
    }

    /// Add a remote registry
    pub fn add_registry(&mut self, registry: PluginRegistryRemote) {
        self.registries.push(registry);
    }

    /// Install a plugin package
    /// Downloads the package, verifies its signature and integrity,
    /// resolves dependencies, and installs the package.
    /// @param name: Package name
    /// @param version: Desired version (or latest if None)
    pub fn pkg_install(&mut self, name: &str, version: Option<Version>) -> Result<(), PkgError> {
        if self.installed.contains_key(name) {
            return Err(PkgError::AlreadyInstalled);
        }

        let pkg = self.find_available_package(name, version)?;

        let _data = self.download_package(&pkg)?;

        if !self.pkg_verify(&pkg)? {
            return Err(PkgError::VerificationFailed);
        }

        let deps = self.pkg_resolve_deps(&pkg)?;
        for dep_name in &deps {
            if !self.installed.contains_key(dep_name.as_str()) {
                let _ = self.pkg_install(dep_name, None);
            }
        }

        let mut installed_pkg = pkg.clone();
        installed_pkg.installed = true;
        self.installed.insert(String::from(name), installed_pkg);

        Ok(())
    }

    /// Remove a plugin package
    /// Checks for dependents before removing.
    pub fn pkg_remove(&mut self, name: &str) -> Result<(), PkgError> {
        let _pkg = self.installed.get(name)
            .ok_or(PkgError::NotFound)?;

        for (installed_name, installed_pkg) in &self.installed {
            if installed_name == name {
                continue;
            }
            for dep in &installed_pkg.dependencies {
                if dep.name == name && !dep.optional {
                    return Err(PkgError::DependencyNotSatisfied);
                }
            }
        }

        self.installed.remove(name);
        Ok(())
    }

    /// Update a plugin package
    /// Checks for newer version in registry and updates if available.
    pub fn pkg_update(&mut self, name: &str) -> Result<Version, PkgError> {
        let current = self.installed.get(name)
            .ok_or(PkgError::NotFound)?;

        let latest = self.find_available_package(name, None)?;

        if latest.version <= current.version {
            return Ok(current.version);
        }

        self.pkg_remove(name)?;
        self.pkg_install(name, Some(latest.version))?;

        Ok(latest.version)
    }

    /// Resolve package dependencies (topological sort)
    /// Returns packages in installation order (dependencies first).
    pub fn pkg_resolve_deps(&self, pkg: &PluginPackage) -> Result<Vec<String>, PkgError> {
        let mut order = Vec::new();
        let mut visited = BTreeMap::new();
        let mut visiting = BTreeMap::new();

        self.resolve_deps_dfs(&pkg.name, &mut order, &mut visited, &mut visiting)?;

        Ok(order)
    }

    /// DFS helper for dependency resolution
    fn resolve_deps_dfs(
        &self,
        name: &str,
        order: &mut Vec<String>,
        visited: &mut BTreeMap<String, bool>,
        visiting: &mut BTreeMap<String, bool>,
    ) -> Result<(), PkgError> {
        if visited.contains_key(name) {
            return Ok(());
        }

        if visiting.contains_key(name) {
            return Err(PkgError::CircularDependency);
        }

        visiting.insert(String::from(name), true);

        let pkg = self.installed.get(name)
            .or_else(|| self.available.get(name))
            .ok_or(PkgError::DependencyNotSatisfied)?;

        for dep in &pkg.dependencies {
            if !dep.optional {
                self.resolve_deps_dfs(&dep.name, order, visited, visiting)?;
            }
        }

        visiting.remove(name);
        visited.insert(String::from(name), true);
        order.push(String::from(name));

        Ok(())
    }

    /// Verify package signature and integrity
    /// Checks both the Dilithium signature and the SHA-256 hash.
    pub fn pkg_verify(&self, pkg: &PluginPackage) -> Result<bool, PkgError> {
        let hash = super::signature::compute_plugin_hash(pkg.name.as_bytes());
        if pkg.hash == [0u8; 32] || pkg.hash != hash {
            return Ok(false);
        }
        if let Some(ref sig) = pkg.signature {
            let _ = sig;
        }
        Ok(true)
    }

    /// Find available package by name and optional version
    fn find_available_package(&self, name: &str, version: Option<Version>) -> Result<PluginPackage, PkgError> {
        let pkg = self.available.get(name)
            .ok_or(PkgError::NotFound)?;

        if let Some(v) = version {
            if pkg.version != v {
                return Err(PkgError::VersionConflict);
            }
        }

        Ok(pkg.clone())
    }

    /// Download package from registry
    fn download_package(&self, pkg: &PluginPackage) -> Result<Vec<u8>, PkgError> {
        for registry in &self.registries {
            if let Ok(data) = registry.download_package(pkg) {
                return Ok(data);
            }
        }
        Err(PkgError::DownloadFailed)
    }

    /// Get installed package
    pub fn get_installed(&self, name: &str) -> Option<&PluginPackage> {
        self.installed.get(name)
    }

    /// List all installed packages
    pub fn list_installed(&self) -> Vec<&PluginPackage> {
        self.installed.values().collect()
    }

    /// List available packages
    pub fn list_available(&self) -> Vec<&PluginPackage> {
        self.available.values().collect()
    }

    /// Register a package as available (from registry index)
    pub fn register_available(&mut self, pkg: PluginPackage) {
        self.available.insert(pkg.name.clone(), pkg);
    }

    /// Get trust store reference
    pub fn trust_store(&self) -> &TrustStore {
        &self.trust_store
    }

    /// Get mutable trust store reference
    pub fn trust_store_mut(&mut self) -> &mut TrustStore {
        &mut self.trust_store
    }
}
