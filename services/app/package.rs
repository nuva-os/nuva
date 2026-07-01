/*
 * Nuva OS - SystemService - App
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


/// PackageState
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageState {
    /// alreadyinstall
    Installed = 0,
    /// alreadyEnable
    Enabled = 1,
    /// Already disabled
    Disabled = 2,
}

/// PackageInfo
pub struct PackageInfo {
    /// Packagename
    pub package_name: &'static str,
    /// Versionname
    pub version_name: &'static str,
    /// Versionsignal
    pub version_code: u32,
    /// Min SDK Version
    pub min_sdk_version: u32,
    /// target SDK Version
    pub target_sdk_version: u32,
    /// State
    pub state: PackageState,
    /// Data Catalog
    pub data_dir: &'static str,
    /// sourceDirectory
    pub source_dir: &'static str,
}

/// PackagemanagementadministrationService
pub struct PackageManager {
    /// PackageArray
    packages: [Option<PackageInfo>; 64],
    /// Packagecount
    num_packages: u32,
}

impl PackageManager {
    pub const fn new() -> Self {
        PackageManager {
            packages: [None; 64],
            num_packages: 0,
        }
    }
    
    /// Initialize
    pub fn init(&mut self) -> i32 {
        log_info!("Package manager initialized");
        0
    }
    
    /// installPackage
    pub fn install(&mut self, package: PackageInfo) -> i32 {
        for slot in self.packages.iter_mut() {
            if slot.is_none() {
                *slot = Some(package);
                self.num_packages += 1;
                
                log_info!("Package installed: {}", package.package_name);
                return 0;
            }
        }
        -1
    }
    
    /// uninstallPackage
    pub fn uninstall(&mut self, package_name: &str) -> i32 {
        for slot in self.packages.iter_mut() {
            if let Some(ref pkg) = slot {
                if pkg.package_name == package_name {
                    *slot = None;
                    self.num_packages -= 1;
                    
                    log_info!("Package uninstalled: {}", package_name);
                    return 0;
                }
            }
        }
        -1
    }
    
    /// EnablePackage
    pub fn enable(&mut self, package_name: &str) -> i32 {
        for slot in self.packages.iter_mut() {
            if let Some(ref mut pkg) = slot {
                if pkg.package_name == package_name {
                    pkg.state = PackageState::Enabled;
                    return 0;
                }
            }
        }
        -1
    }
    
    /// DisablePackage
    pub fn disable(&mut self, package_name: &str) -> i32 {
        for slot in self.packages.iter_mut() {
            if let Some(ref mut pkg) = slot {
                if pkg.package_name == package_name {
                    pkg.state = PackageState::Disabled;
                    return 0;
                }
            }
        }
        -1
    }
    
    /// FindPackage
    pub fn get_package(&self, package_name: &str) -> Option<&PackageInfo> {
        for slot in self.packages.iter() {
            if let Some(ref pkg) = slot {
                if pkg.package_name == package_name {
                    return Some(pkg);
                }
            }
        }
        None
    }
    
    /// CheckPackageifexist
    pub fn package_exists(&self, package_name: &str) -> bool {
        self.get_package(package_name).is_some()
    }
}

static PACKAGE_MANAGER: crate::sync_oncelock::OnceLock<PackageManager> = crate::sync_oncelock::OnceLock::new();

pub fn get_package_manager() -> &'static mut PackageManager {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut PACKAGE_MANAGER }
}

pub fn init_package_manager() {
    let manager = get_package_manager();
    manager.init();
}