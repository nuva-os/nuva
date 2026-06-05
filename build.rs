/*
 * Nuva OS - Build
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
/*
 * Build Script - Dependency Check Integration
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This build script runs dependency analysis during build process
 * to enforce architectural layer boundaries.
 */

use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    // Skip dependency check for specific features
    if env::var("CARGO_FEATURE_SKIP_DEP_CHECK").is_ok() {
        println!("cargo:warning=Skipping dependency check");
        return;
    }

    // Only run in release builds or when explicitly requested
    let is_release = env::var("PROFILE").unwrap_or_default() == "release";
    let check_deps = env::var("NUVA_CHECK_DEPS").is_ok();

    if !is_release && !check_deps {
        println!(
            "cargo:warning=Dependency check skipped in dev build. Set NUVA_CHECK_DEPS=1 to enable."
        );
        return;
    }

    println!("cargo:rerun-if-changed=build.rs");

    // Get project root directory
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = Path::new(&manifest_dir);

    // Check if dependency analyzer exists
    let dep_analyzer = project_root
        .join("tools")
        .join("dep_analyzer")
        .join("target")
        .join("release")
        .join("dep_analyzer");

    if !dep_analyzer.exists() {
        println!("cargo:warning=Dependency analyzer not found. Building it first...");

        // Build dependency analyzer
        let build_result = Command::new("cargo")
            .args(&["build", "--release"])
            .current_dir(project_root.join("tools").join("dep_analyzer"))
            .status()
            .expect("Failed to build dependency analyzer");

        if !build_result.success() {
            panic!("Failed to build dependency analyzer");
        }
    }

    // Run dependency analyzer
    println!("cargo:warning=Running dependency analysis...");

    let result = Command::new(&dep_analyzer)
        .arg(project_root)
        .status()
        .expect("Failed to run dependency analyzer");

    if !result.success() {
        panic!("Dependency violations found! See output above for details.");
    }

    println!("cargo:warning=✅ Dependency check passed");
}
