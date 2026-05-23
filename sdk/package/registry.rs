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

// ! packetregisterforminterface

use super::meta::{Package, PackageSummary};
use crate::error::SdkError;

/// packetregisterform trait
pub trait PackageRegistry {
    /// getpacket
    fn fetch(&self, name: &str, version: &str) -> Result<Package, SdkError>;
    
    /// searchpacket
    fn search(&self, query: &str) -> Result<Vec<PackageSummary>, SdkError>;
    
    /// releasepacket
    fn publish(&self, pkg: &Package) -> Result<(), SdkError>;
    
    /// getpacket placefiniteversion
    fn versions(&self, name: &str) -> Result<Vec<String>, SdkError>;
}

/// Central registry
#[derive(Debug)]
pub struct CentralRegistry {
    /// Registry URL
    url: String,
    /// API endpoint
    api_endpoint: String,
    /// HTTP client
    client: reqwest::blocking::Client,
    /// Timeout in seconds
    timeout: u64,
}

impl Default for CentralRegistry {
    fn default() -> Self {
        Self::new("https://registry.nuva.io")
    }
}

impl CentralRegistry {
    pub fn new(url: impl Into<String>) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("nuva-sdk/0.1.0")
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            url: url.into(),
            api_endpoint: "/api/v1".to_string(),
            client,
            timeout: 30,
        }
    }

    fn package_url(&self, name: &str, version: &str) -> String {
        format!("{}/packages/{}/{}", self.url, name, version)
    }

    fn search_url(&self) -> String {
        format!("{}{}/search", self.url, self.api_endpoint)
    }

    /// Perform HTTP GET request
    fn http_get(&self, url: &str) -> Result<HttpResponse, SdkError> {
        log_debug!("HTTP GET: {}", url);

        let response = self.client
            .get(url)
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .map_err(|e| SdkError::NetworkError(format!("HTTP request failed: {}", e)))?;

        let status = response.status().as_u16();
        let body = response
            .text()
            .map_err(|e| SdkError::NetworkError(format!("Failed to read response: {}", e)))?;

        log_debug!("HTTP Response: {} ({} bytes)", status, body.len());

        Ok(HttpResponse {
            status,
            body,
        })
    }

    /// Perform HTTP POST request
    fn http_post(&self, url: &str, data: &str) -> Result<HttpResponse, SdkError> {
        log_debug!("HTTP POST: {} ({} bytes)", url, data.len());

        let response = self.client
            .post(url)
            .header("Content-Type", "application/json")
            .body(data.to_string())
            .timeout(std::time::Duration::from_secs(self.timeout))
            .send()
            .map_err(|e| SdkError::NetworkError(format!("HTTP request failed: {}", e)))?;

        let status = response.status().as_u16();
        let body = response
            .text()
            .map_err(|e| SdkError::NetworkError(format!("Failed to read response: {}", e)))?;

        log_debug!("HTTP Response: {} ({} bytes)", status, body.len());

        Ok(HttpResponse {
            status,
            body,
        })
    }

    /// Download package archive
    pub fn download_package(&self, name: &str, version: &str) -> Result<Vec<u8>, SdkError> {
        log_info!("Downloading package: {}@{}", name, version);

        let url = format!("{}/packages/{}/{}/download", self.url, name, version);

        let response = self.client
            .get(&url)
            .timeout(std::time::Duration::from_secs(300))
            .send()
            .map_err(|e| SdkError::NetworkError(format!("Download failed: {}", e)))?;

        if response.status() != 200 {
            return Err(SdkError::NetworkError(format!(
                "Download failed: HTTP {}",
                response.status()
            )));
        }

        let data = response
            .bytes()
            .map_err(|e| SdkError::NetworkError(format!("Failed to read download: {}", e)))?
            .to_vec();

        log_info!("Downloaded {}@{} ({} bytes)", name, version, data.len());

        Ok(data)
    }
}

/// HTTP response
struct HttpResponse {
    status: u16,
    body: String,
}

impl PackageRegistry for CentralRegistry {
    fn fetch(&self, name: &str, version: &str) -> Result<Package, SdkError> {
        log_debug!("Fetching package: {}@{} from registry", name, version);
        
        // Build request URL
        let url = format!("{}/packages/{}/{}", self.url, name, version);
        log_debug!("Request URL: {}", url);
        
        // Send HTTP GET request
        let response = self.http_get(&url)?;
        
        if response.status == 404 {
            return Err(SdkError::NetworkError(format!(
                "Package {}@{} not found in registry",
                name, version
            )));
        }
        
        if response.status != 200 {
            return Err(SdkError::NetworkError(format!(
                "Failed to fetch package: HTTP {}",
                response.status
            )));
        }
        
        // Parse response
        let package_data = response.body;
        let pkg: Package = serde_json::from_str(&package_data)
            .map_err(|e| SdkError::ParseError(format!("Failed to parse package metadata: {}", e)))?;
        
        log_info!("Fetched package: {}@{}", pkg.name, pkg.version);
        
        Ok(pkg)
    }

    fn search(&self, query: &str) -> Result<Vec<PackageSummary>, SdkError> {
        log_debug!("Searching packages with query: {}", query);

        let url = format!("{}?q={}", self.search_url(), query);
        let response = self.http_get(&url)?;

        if response.status != 200 {
            return Err(SdkError::NetworkError(format!(
                "Search failed: HTTP {}",
                response.status
            )));
        }

        let results: Vec<PackageSummary> = serde_json::from_str(&response.body)
            .map_err(|e| SdkError::ParseError(format!("Failed to parse search results: {}", e)))?;

        log_info!("Found {} packages matching '{}'", results.len(), query);

        Ok(results)
    }

    fn publish(&self, pkg: &Package) -> Result<(), SdkError> {
        log_info!("Publishing package: {}@{}", pkg.name, pkg.version);

        if pkg.name.is_empty() {
            return Err(SdkError::InvalidArgument("Package name is empty".to_string()));
        }

        let url = format!("{}/packages", self.url);
        let data = serde_json::to_string(pkg)
            .map_err(|e| SdkError::ParseError(format!("Failed to serialize package: {}", e)))?;

        let response = self.http_post(&url, &data)?;

        if response.status != 200 && response.status != 201 {
            return Err(SdkError::NetworkError(format!(
                "Publish failed: HTTP {}",
                response.status
            )));
        }

        log_info!("Published package: {}@{}", pkg.name, pkg.version);

        Ok(())
    }

    fn versions(&self, name: &str) -> Result<Vec<String>, SdkError> {
        log_debug!("Fetching versions for package: {}", name);

        let url = format!("{}/packages/{}/versions", self.url, name);
        let response = self.http_get(&url)?;

        if response.status != 200 {
            return Err(SdkError::NetworkError(format!(
                "Versions query failed: HTTP {}",
                response.status
            )));
        }

        let versions: Vec<String> = serde_json::from_str(&response.body)
            .map_err(|e| SdkError::ParseError(format!("Failed to parse versions: {}", e)))?;

        log_info!("Found {} versions for {}", versions.len(), name);

        Ok(versions)
    }
}

/// Localregisterform
pub struct LocalRegistry {
    /// LocalPath
    path: std::path::PathBuf,
}

impl LocalRegistry {
    pub fn new(path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            path: path.into(),
        }
    }
}

impl PackageRegistry for LocalRegistry {
    fn fetch(&self, name: &str, version: &str) -> Result<Package, SdkError> {
        let pkg_path = self.path.join(name).join(version);
        if pkg_path.exists() {
            Package::from_file(&pkg_path.join("Nuva.toml"))
                .map_err(|e| SdkError::ParseError(e.to_string()))
        } else {
            Err(SdkError::PackageNotFound(format!("{}@{}", name, version)))
        }
    }

    fn search(&self, query: &str) -> Result<Vec<PackageSummary>, SdkError> {
        let mut results = vec![];
        
        for entry in std::fs::read_dir(&self.path)
            .map_err(|e| SdkError::IoError(e.to_string()))?
        {
            let entry = entry.map_err(|e| SdkError::IoError(e.to_string()))?;
            let name = entry.file_name().to_string_lossy().to_string();
            
            if name.contains(query) {
                results.push(PackageSummary {
                    name,
                    version: "local".to_string(),
                    description: None,
                });
            }
        }
        
        Ok(results)
    }

    fn publish(&self, pkg: &Package) -> Result<(), SdkError> {
        let pkg_dir = self.path.join(&pkg.name).join(pkg.version.to_string());
        std::fs::create_dir_all(&pkg_dir)
            .map_err(|e| SdkError::IoError(e.to_string()))?;

        let manifest_content = toml::to_string_pretty(pkg)
            .map_err(|e| SdkError::ParseError(e.to_string()))?;
        std::fs::write(pkg_dir.join("Nuva.toml"), manifest_content)
            .map_err(|e| SdkError::IoError(e.to_string()))?;

        Ok(())
    }

    fn versions(&self, name: &str) -> Result<Vec<String>, SdkError> {
        let pkg_dir = self.path.join(name);
        if !pkg_dir.exists() {
            return Ok(vec![]);
        }
        
        let mut versions = vec![];
        for entry in std::fs::read_dir(&pkg_dir)
            .map_err(|e| SdkError::IoError(e.to_string()))?
        {
            let entry = entry.map_err(|e| SdkError::IoError(e.to_string()))?;
            versions.push(entry.file_name().to_string_lossy().to_string());
        }
        
        Ok(versions)
    }
}

/// Git registerform
pub struct GitRegistry {
    /// Git repolibrary URL
    url: String,
}

impl GitRegistry {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
        }
    }
}

impl PackageRegistry for GitRegistry {
    fn fetch(&self, name: &str, version: &str) -> Result<Package, SdkError> {
        let temp_dir = std::env::temp_dir().join(format!("nuva-git-{}-{}", name, version));
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir)
                .map_err(|e| SdkError::IoError(e.to_string()))?;
        }

        let clone_result = std::process::Command::new("git")
            .args(&["clone", &self.url, &temp_dir.to_string_lossy()])
            .output()
            .map_err(|e| SdkError::ExecutionError(format!("Failed to run git clone: {}", e)))?;

        if !clone_result.status.success() {
            return Err(SdkError::NetworkError(format!(
                "Git clone failed for {}: {}",
                self.url,
                String::from_utf8_lossy(&clone_result.stderr)
            )));
        }

        if !version.is_empty() && version != "latest" {
            let checkout_result = std::process::Command::new("git")
                .args(&["-C", &temp_dir.to_string_lossy(), "checkout", version])
                .output()
                .map_err(|e| SdkError::ExecutionError(format!("Failed to run git checkout: {}", e)))?;

            if !checkout_result.status.success() {
                return Err(SdkError::NetworkError(format!(
                    "Git checkout failed for tag/branch {}: {}",
                    version,
                    String::from_utf8_lossy(&checkout_result.stderr)
                )));
            }
        }

        let manifest_path = temp_dir.join("Nuva.toml");
        Package::from_file(&manifest_path)
            .map_err(|e| SdkError::ParseError(e.to_string()))
    }

    fn search(&self, _query: &str) -> Result<Vec<PackageSummary>, SdkError> {
        Ok(vec![])
    }

    fn publish(&self, _pkg: &Package) -> Result<(), SdkError> {
        Err(SdkError::Unsupported("Cannot publish to Git registry".to_string()))
    }

    fn versions(&self, _name: &str) -> Result<Vec<String>, SdkError> {
        Ok(vec![])
    }
}