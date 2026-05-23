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

//! Package management command

use crate::NuvaSdk;
use crate::error::SdkError;
use crate::cli::args::PkgCommand;
use crate::cli::output;

/// Execute package management command
pub fn execute(sdk: &mut NuvaSdk, cmd: PkgCommand) -> Result<(), SdkError> {
    match cmd {
        PkgCommand::Install { packages, dev } => install(sdk, packages, dev),
        PkgCommand::Uninstall { packages } => uninstall(sdk, packages),
        PkgCommand::Update { packages } => update(sdk, packages),
        PkgCommand::Search { query } => search(sdk, query),
        PkgCommand::Publish { dry_run } => publish(sdk, dry_run),
        PkgCommand::List { depth } => list(sdk, depth),
        PkgCommand::Lock => lock(sdk),
    }
}

/// Install dependencies
fn install(sdk: &mut NuvaSdk, packages: Vec<String>, dev: bool) -> Result<(), SdkError> {
    if packages.is_empty() {
        output::info("Installing all dependencies...");
    } else {
        let dep_type = if dev { "dev dependencies" } else { "dependencies" };
        output::info(&format!("Installing {} as {}...", packages.join(", "), dep_type));
    }
    
    // 1. Load manifest
    let manifest = sdk.load_manifest()?;
    output::debug(&format!("Loaded manifest for {}", manifest.name));
    
    // 2. Parse package specifications
    let mut deps_to_install = Vec::new();
    for pkg_spec in &packages {
        let dep = sdk.parse_package_spec(pkg_spec)?;
        deps_to_install.push(dep);
    }
    
    // 3. If no packages specified, install all from manifest
    if deps_to_install.is_empty() {
        deps_to_install = if dev {
            manifest.dev_dependencies.clone()
        } else {
            manifest.dependencies.clone()
        };
    }
    
    if deps_to_install.is_empty() {
        output::info("No dependencies to install");
        return Ok(());
    }
    
    // 4. Resolve dependencies
    output::info("Resolving dependencies...");
    let resolved = sdk.resolve_dependencies()?;
    output::debug(&format!("Resolved {} dependencies", resolved.len()));
    
    // 5. Download packages
    output::info("Downloading packages...");
    let download_start = std::time::Instant::now();
    
    for dep in &deps_to_install {
        output::progress(deps_to_install.iter().position(|d| d == dep).unwrap() + 1, 
                         deps_to_install.len(),
                         &format!("Downloading {}", dep.name));
        sdk.download_package(dep)?;
    }
    output::clear_line();
    
    let download_time = download_start.elapsed();
    output::debug(&format!("Download completed in {:?}", download_time));
    
    // 6. Install packages
    output::info("Installing packages...");
    let install_start = std::time::Instant::now();
    
    for dep in &deps_to_install {
        sdk.install_package(dep, dev)?;
    }
    
    let install_time = install_start.elapsed();
    output::debug(&format!("Installation completed in {:?}", install_time));
    
    // 7. Update manifest
    sdk.update_manifest_with_deps(&deps_to_install, dev)?;
    
    // 8. Generate lock file
    output::info("Updating lock file...");
    sdk.generate_lock_file()?;
    
    output::success("Dependencies installed successfully");
    Ok(())
}

/// Uninstall dependencies
fn uninstall(sdk: &mut NuvaSdk, packages: Vec<String>) -> Result<(), SdkError> {
    output::info(&format!("Uninstalling {}...", packages.join(", ")));
    
    // 1. Load manifest
    let manifest = sdk.load_manifest()?;
    
    // 2. Find dependencies to remove
    let mut deps_to_remove = Vec::new();
    for pkg_name in &packages {
        if let Some(dep) = manifest.dependencies.iter().find(|d| &d.name == pkg_name) {
            deps_to_remove.push(dep.clone());
        } else if let Some(dep) = manifest.dev_dependencies.iter().find(|d| &d.name == pkg_name) {
            deps_to_remove.push(dep.clone());
        } else {
            output::warning(&format!("Package '{}' not found in dependencies", pkg_name));
        }
    }
    
    if deps_to_remove.is_empty() {
        output::info("No packages to uninstall");
        return Ok(());
    }
    
    // 3. Check for dependents
    output::info("Checking for dependent packages...");
    for dep in &deps_to_remove {
        let dependents = sdk.find_dependents(&dep.name)?;
        if !dependents.is_empty() {
            output::warning(&format!("Package '{}' is required by: {}", 
                                     dep.name, dependents.join(", ")));
        }
    }
    
    // 4. Remove packages
    output::info("Removing packages...");
    for dep in &deps_to_remove {
        sdk.remove_package(dep)?;
    }
    
    // 5. Update manifest
    sdk.update_manifest_remove_deps(&deps_to_remove)?;
    
    // 6. Regenerate lock file
    output::info("Updating lock file...");
    sdk.generate_lock_file()?;
    
    output::success("Packages uninstalled successfully");
    Ok(())
}

/// Update dependencies
fn update(sdk: &mut NuvaSdk, packages: Vec<String>) -> Result<(), SdkError> {
    if packages.is_empty() {
        output::info("Updating all dependencies...");
    } else {
        output::info(&format!("Updating {}...", packages.join(", ")));
    }
    
    // 1. Load manifest
    let manifest = sdk.load_manifest()?;
    
    // 2. Determine which packages to update
    let deps_to_update = if packages.is_empty() {
        manifest.dependencies.clone()
    } else {
        manifest.dependencies.iter()
            .filter(|d| packages.contains(&d.name))
            .cloned()
            .collect()
    };
    
    if deps_to_update.is_empty() {
        output::info("No packages to update");
        return Ok(());
    }
    
    // 3. Check for available updates
    output::info("Checking for updates...");
    let mut updates_available = Vec::new();
    
    for dep in &deps_to_update {
        if let Some(latest_version) = sdk.check_for_update(&dep.name)? {
            if latest_version > dep.version {
                updates_available.push((dep.clone(), latest_version));
            }
        }
    }
    
    if updates_available.is_empty() {
        output::info("All packages are up to date");
        return Ok(());
    }
    
    output::info(&format!("Found {} update(s):", updates_available.len()));
    for (dep, new_version) in &updates_available {
        println!("  {} {} -> {}", dep.name, dep.version, new_version);
    }
    
    // 4. Download and install updates
    output::info("Downloading updates...");
    for (dep, new_version) in &updates_available {
        output::debug(&format!("Downloading {} {}...", dep.name, new_version));
        sdk.download_package_with_version(&dep.name, new_version)?;
    }
    
    // 5. Install updates
    output::info("Installing updates...");
    for (dep, new_version) in &updates_available {
        sdk.install_package_with_version(&dep.name, new_version)?;
    }
    
    // 6. Update manifest
    sdk.update_manifest_with_updates(&updates_available)?;
    
    // 7. Regenerate lock file
    output::info("Updating lock file...");
    sdk.generate_lock_file()?;
    
    output::success("Dependencies updated successfully");
    Ok(())
}

/// Search packages
fn search(sdk: &mut NuvaSdk, query: String) -> Result<(), SdkError> {
    output::info(&format!("Searching for '{}'...", query));
    
    // 1. Search package registry
    let results = sdk.search_packages(&query)?;
    
    if results.is_empty() {
        println!("No packages found matching '{}'", query);
        return Ok(());
    }
    
    // 2. Display results
    println!("
Found {} package(s):
", results.len());
    
    for (i, pkg) in results.iter().enumerate() {
        println!("{}. {} v{}", i + 1, pkg.name, pkg.version);
        println!("   Description: {}", pkg.description);
        
        if !pkg.keywords.is_empty() {
            println!("   Keywords: {}", pkg.keywords.join(", "));
        }
        
        println!("   Downloads: {}", pkg.downloads);
        println!("   License: {}", pkg.license);
        println!();
    }
    
    output::success(&format!("Found {} package(s)", results.len()));
    Ok(())
}

/// Publish package
fn publish(sdk: &mut NuvaSdk, dry_run: bool) -> Result<(), SdkError> {
    if dry_run {
        output::info("Dry run: checking package...");
    } else {
        output::info("Publishing package...");
    }
    
    // 1. Load and validate manifest
    let manifest = sdk.load_manifest()?;
    output::debug(&format!("Loaded manifest for {}", manifest.name));
    
    // 2. Validate package
    output::info("Validating package...");
    sdk.validate_package(&manifest)?;
    output::success("Package validation passed");
    
    // 3. Check if version already exists
    output::info("Checking version availability...");
    if sdk.version_exists(&manifest.name, &manifest.version)? {
        return Err(SdkError::PublishError(format!(
            "Version {} of {} already exists",
            manifest.version, manifest.name
        )));
    }
    
    // 4. Prepare package archive
    output::info("Preparing package archive...");
    let archive_path = sdk.prepare_package_archive()?;
    output::debug(&format!("Archive created: {}", archive_path.display()));
    
    // 5. Calculate checksum
    output::info("Calculating checksum...");
    let checksum = sdk.calculate_checksum(&archive_path)?;
    output::debug(&format!("Checksum: {}", checksum));
    
    if dry_run {
        output::success("Package is ready to publish");
        output::info(&format!("Package: {} v{}", manifest.name, manifest.version));
        output::info(&format!("Archive: {}", archive_path.display()));
        output::info(&format!("Checksum: {}", checksum));
        return Ok(());
    }
    
    // 6. Upload package
    output::info("Uploading package...");
    let upload_start = std::time::Instant::now();
    
    sdk.upload_package(&archive_path, &checksum)?;
    
    let upload_time = upload_start.elapsed();
    output::debug(&format!("Upload completed in {:?}", upload_time));
    
    // 7. Verify publication
    output::info("Verifying publication...");
    sdk.verify_publication(&manifest.name, &manifest.version)?;
    
    output::success(&format!("Package {} v{} published successfully", 
                             manifest.name, manifest.version));
    Ok(())
}

/// List dependencies
fn list(sdk: &mut NuvaSdk, depth: Option<usize>) -> Result<(), SdkError> {
    output::info("Listing dependencies...");
    
    // 1. Load manifest
    let manifest = sdk.load_manifest()?;
    
    // 2. Get dependency tree
    let dep_tree = sdk.get_dependency_tree(depth.unwrap_or(1))?;
    
    // 3. Display dependencies
    println!("
Dependencies for {} v{}:", manifest.name, manifest.version);
    println!();
    
    for (name, info) in &dep_tree {
        println!("└── {} v{}", name, info.version);
        
        if let Some(ref deps) = info.dependencies {
            display_dependencies(deps, 1, depth.unwrap_or(1));
        }
    }
    
    println!();
    
    // 4. Display statistics
    let total_deps = dep_tree.len();
    let dev_deps = manifest.dev_dependencies.len();
    
    println!("Total dependencies: {}", total_deps);
    println!("Dev dependencies: {}", dev_deps);
    
    output::success(&format!("Listed {} dependencies", total_deps));
    Ok(())
}

/// Display dependencies with indentation
fn display_dependencies(deps: &[(String, crate::package::DependencyInfo)], 
                         level: usize, max_depth: usize) {
    if level >= max_depth {
        return;
    }
    
    let indent = "    ".repeat(level);
    
    for (name, info) in deps {
        println!("{}└── {} v{}", indent, name, info.version);
        
        if let Some(ref sub_deps) = info.dependencies {
            display_dependencies(sub_deps, level + 1, max_depth);
        }
    }
}

/// Lock dependencies
fn lock(sdk: &mut NuvaSdk) -> Result<(), SdkError> {
    output::info("Generating lock file...");
    
    // 1. Load manifest
    let manifest = sdk.load_manifest()?;
    
    // 2. Resolve all dependencies
    output::info("Resolving dependencies...");
    let resolved = sdk.resolve_dependencies()?;
    
    // 3. Generate lock file
    output::info("Writing lock file...");
    sdk.generate_lock_file()?;
    
    // 4. Verify lock file
    output::info("Verifying lock file...");
    sdk.verify_lock_file()?;
    
    output::success(&format!("Lock file generated for {} v{}", 
                             manifest.name, manifest.version));
    Ok(())
}
