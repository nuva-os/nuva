/*
 * Nuva OS - Tools - DepAnalyzer - Main
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
 * Dependency Analyzer - Architecture Compliance Tool
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This tool analyzes module dependencies and enforces
 * architectural layer boundaries to prevent violations.
 *
 * Enhanced features:
 * - Correct syslib:: path mapping (was incorrectly lib::)
 * - L0 (HAL) zero-upward-dependency check
 * - HAL concrete implementation reference detection
 * - Deprecated module exemption mechanism
 * - L1 reverse dependency detection (kernel must not depend on L2/L3/L4)
 */

use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use alloc::format;
use alloc::vec::Vec;

/// Architecture layer enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Layer {
    Hal = 0,
    Kernel = 1,
    Syslib = 2,
    Services = 3,
    Application = 4,
}

impl Layer {
    /// Get layer from module path
    fn from_path(path: &str) -> Option<Self> {
        if path.starts_with("hal::") {
            Some(Self::Hal)
        } else if path.starts_with("kernel::") {
            Some(Self::Kernel)
        } else if path.starts_with("syslib::") {
            Some(Self::Syslib)
        } else if path.starts_with("services::") {
            Some(Self::Services)
        } else if path.starts_with("application::") {
            Some(Self::Application)
        } else {
            None
        }
    }

    /// Get layer name as string
    fn name(&self) -> &'static str {
        match self {
            Layer::Hal => "L0-HAL",
            Layer::Kernel => "L1-Kernel",
            Layer::Syslib => "L2-Syslib",
            Layer::Services => "L3-Services",
            Layer::Application => "L4-Application",
        }
    }
}

/// HAL concrete architecture paths that L1-L4 must not reference
const HAL_CONCRETE_PATHS: &[&str] = &[
    "hal::arm64::",
    "hal::x64::",
    "hal::loongarch64::",
    "hal::snapdragon::",
];

/// HAL trait paths that are allowed for L1-L4 references
const HAL_TRAIT_ALLOWED_PREFIXES: &[&str] = &[
    "hal::cpu::",
    "hal::gpu::",
    "hal::npu::",
    "hal::power::",
    "hal::quantum::",
    "hal::input::",
    "hal::ffi::",
    "hal::platform::",
    "hal::acpi::",
    "hal::dt::",
];

/// Kernel core modules that must not depend on POSIX
const KERNEL_CORE_MODULES: &[&str] = &[
    "kernel::core::",
    "kernel::ipc::",
    "kernel::sched::",
    "kernel::mm::",
    "kernel::security::",
    "kernel::capability::",
    "kernel::nv_process::",
    "kernel::nv_event::",
];

/// Dependency violation
#[derive(Debug, Clone)]
struct Violation {
    from_module: String,
    to_module: String,
    from_layer: Layer,
    to_layer: Layer,
    violation_type: ViolationType,
    severity: Severity,
}

/// Violation severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Severity {
    /// Hard error - must be fixed
    Error,
    /// Warning - should be fixed, exempted items allowed
    Warning,
}

/// Violation type
#[derive(Debug, Clone)]
enum ViolationType {
    /// Lower layer depending on higher layer
    LayerViolation,

    /// Circular dependency
    CircularDependency,

    /// Cross-layer dependency without abstraction
    DirectDependency,

    /// L0 (HAL) has upward dependency (zero-dependency violation)
    HalUpwardDependency,

    /// L1-L4 references HAL concrete implementation instead of trait
    HalConcreteImplReference,

    /// L1 depends on L2/L3/L4 (reverse dependency)
    KernelReverseDependency,

    /// Kernel core module depends on POSIX module (architecture independence violation)
    KernelPosixDependency,

    /// Kernel core module uses POSIX errno (architecture independence violation)
    KernelPosixErrnoUsage,
}

impl ViolationType {
    fn description(&self) -> &'static str {
        match self {
            ViolationType::LayerViolation => "Layer Violation",
            ViolationType::CircularDependency => "Circular Dependency",
            ViolationType::DirectDependency => "Direct Cross-Layer Dependency",
            ViolationType::HalUpwardDependency => "HAL Upward Dependency",
            ViolationType::HalConcreteImplReference => "HAL Concrete Impl Reference",
            ViolationType::KernelReverseDependency => "Kernel Reverse Dependency",
            ViolationType::KernelPosixDependency => "Kernel POSIX Dependency",
            ViolationType::KernelPosixErrnoUsage => "Kernel POSIX Errno Usage",
        }
    }
}

/// Exemption entry from configuration
#[derive(Debug, Clone)]
struct Exemption {
    module: String,
    reason: String,
    deadline: String,
    violation_types: HashSet<String>,
}

impl Exemption {
    fn matches(&self, from_module: &str, violation_type: &ViolationType) -> bool {
        if !from_module.starts_with(&self.module) && from_module != self.module {
            return false;
        }
        let type_str = violation_type.description();
        self.violation_types.contains(type_str) || self.violation_types.contains("*")
    }
}

/// Load exemptions from dep_analyzer.toml if present
fn load_exemptions(root: &Path) -> Vec<Exemption> {
    let config_path = root.join("dep_analyzer.toml");
    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut exemptions = Vec::new();
    let mut current_module = String::new();
    let mut current_reason = String::new();
    let mut current_deadline = String::new();
    let mut current_types = HashSet::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        if line.starts_with("[[exemption]]") {
            if !current_module.is_empty() {
                exemptions.push(Exemption {
                    module: current_module.clone(),
                    reason: current_reason.clone(),
                    deadline: current_deadline.clone(),
                    violation_types: current_types.clone(),
                });
            }
            current_module.clear();
            current_reason.clear();
            current_deadline.clear();
            current_types.clear();
        } else if let Some(value) = line.strip_prefix("module = ") {
            current_module = value.trim_matches('"').to_string();
        } else if let Some(value) = line.strip_prefix("reason = ") {
            current_reason = value.trim_matches('"').to_string();
        } else if let Some(value) = line.strip_prefix("deadline = ") {
            current_deadline = value.trim_matches('"').to_string();
        } else if let Some(value) = line.strip_prefix("types = ") {
            for t in value.trim_matches('[').trim_matches(']').split(',') {
                let t = t.trim().trim_matches('"');
                if !t.is_empty() {
                    current_types.insert(t.to_string());
                }
            }
        }
    }

    if !current_module.is_empty() {
        exemptions.push(Exemption {
            module: current_module,
            reason: current_reason,
            deadline: current_deadline,
            violation_types: current_types,
        });
    }

    exemptions
}

/// Dependency graph
struct DependencyGraph {
    /// Module -> dependencies (raw use paths)
    dependencies: HashMap<String, HashSet<String>>,

    /// Module -> layer
    layers: HashMap<String, Layer>,

    /// Exemptions
    exemptions: Vec<Exemption>,
}

impl DependencyGraph {
    fn new(exemptions: Vec<Exemption>) -> Self {
        Self {
            dependencies: HashMap::new(),
            layers: HashMap::new(),
            exemptions,
        }
    }

    /// Add module
    fn add_module(&mut self, module: String, layer: Layer) {
        self.layers.insert(module.clone(), layer);
        self.dependencies.entry(module).or_insert_with(HashSet::new);
    }

    /// Add dependency
    fn add_dependency(&mut self, from: String, to: String) {
        self.dependencies
            .entry(from)
            .or_insert_with(HashSet::new)
            .insert(to);
    }

    /// Check if a violation is exempted
    fn is_exempted(&self, from_module: &str, violation_type: &ViolationType) -> bool {
        for exemption in &self.exemptions {
            if exemption.matches(from_module, violation_type) {
                return true;
            }
        }
        false
    }

    /// Check for all violation types
    fn check_violations(&self) -> Vec<Violation> {
        let mut violations = Vec::new();

        self.check_layer_violations(&mut violations);
        self.check_hal_zero_dependency(&mut violations);
        self.check_kernel_reverse_dependency(&mut violations);
        self.check_hal_concrete_impl(&mut violations);
        self.check_circular_dependencies(&mut violations);
        self.check_kernel_posix_dependency(&mut violations);
        self.check_kernel_posix_errno(&mut violations);

        violations
    }

    /// Check layer boundary violations (lower -> higher)
    fn check_layer_violations(&self, violations: &mut Vec<Violation>) {
        for (from_module, deps) in &self.dependencies {
            let from_layer = match self.layers.get(from_module) {
                Some(layer) => layer,
                None => continue,
            };

            for to_module in deps {
                let to_layer = match self.layers.get(to_module) {
                    Some(layer) => layer,
                    None => continue,
                };

                if from_layer < to_layer {
                    let severity = if self.is_exempted(from_module, &ViolationType::LayerViolation)
                    {
                        Severity::Warning
                    } else {
                        Severity::Error
                    };

                    violations.push(Violation {
                        from_module: from_module.clone(),
                        to_module: to_module.clone(),
                        from_layer: *from_layer,
                        to_layer: *to_layer,
                        violation_type: ViolationType::LayerViolation,
                        severity,
                    });
                }
            }
        }
    }

    /// Check L0 (HAL) zero upward dependency
    fn check_hal_zero_dependency(&self, violations: &mut Vec<Violation>) {
        for (from_module, deps) in &self.dependencies {
            let from_layer = match self.layers.get(from_module) {
                Some(Layer::Hal) => Layer::Hal,
                _ => continue,
            };

            for to_module in deps {
                let to_layer = match self.layers.get(to_module) {
                    Some(layer) => layer,
                    None => continue,
                };

                if *to_layer != Layer::Hal {
                    let severity =
                        if self.is_exempted(from_module, &ViolationType::HalUpwardDependency) {
                            Severity::Warning
                        } else {
                            Severity::Error
                        };

                    violations.push(Violation {
                        from_module: from_module.clone(),
                        to_module: to_module.clone(),
                        from_layer,
                        to_layer: *to_layer,
                        violation_type: ViolationType::HalUpwardDependency,
                        severity,
                    });
                }
            }
        }
    }

    /// Check L1 (Kernel) reverse dependency - must not depend on L2/L3/L4
    fn check_kernel_reverse_dependency(&self, violations: &mut Vec<Violation>) {
        let forbidden_layers = [Layer::Syslib, Layer::Services, Layer::Application];

        for (from_module, deps) in &self.dependencies {
            let from_layer = match self.layers.get(from_module) {
                Some(Layer::Kernel) => Layer::Kernel,
                _ => continue,
            };

            for to_module in deps {
                let to_layer = match self.layers.get(to_module) {
                    Some(layer) => layer,
                    None => continue,
                };

                if forbidden_layers.contains(to_layer) {
                    let severity =
                        if self.is_exempted(from_module, &ViolationType::KernelReverseDependency) {
                            Severity::Warning
                        } else {
                            Severity::Error
                        };

                    violations.push(Violation {
                        from_module: from_module.clone(),
                        to_module: to_module.clone(),
                        from_layer,
                        to_layer: *to_layer,
                        violation_type: ViolationType::KernelReverseDependency,
                        severity,
                    });
                }
            }
        }
    }

    /// Check L1-L4 references to HAL concrete implementations
    fn check_hal_concrete_impl(&self, violations: &mut Vec<Violation>) {
        for (from_module, deps) in &self.dependencies {
            let from_layer = match self.layers.get(from_module) {
                Some(Layer::Hal) => continue,
                Some(layer) => layer,
                None => continue,
            };

            for to_module in deps {
                if !to_module.starts_with("hal::") {
                    continue;
                }

                let is_concrete = HAL_CONCRETE_PATHS.iter().any(|p| to_module.starts_with(p));
                let is_allowed_trait = HAL_TRAIT_ALLOWED_PREFIXES
                    .iter()
                    .any(|p| to_module.starts_with(p));

                if is_concrete && !is_allowed_trait {
                    let severity = if self
                        .is_exempted(from_module, &ViolationType::HalConcreteImplReference)
                    {
                        Severity::Warning
                    } else {
                        Severity::Error
                    };

                    violations.push(Violation {
                        from_module: from_module.clone(),
                        to_module: to_module.clone(),
                        from_layer: *from_layer,
                        to_layer: Layer::Hal,
                        violation_type: ViolationType::HalConcreteImplReference,
                        severity,
                    });
                }
            }
        }
    }

    /// Check kernel core modules for POSIX dependency
    fn check_kernel_posix_dependency(&self, violations: &mut Vec<Violation>) {
        for (from_module, deps) in &self.dependencies {
            let is_core = KERNEL_CORE_MODULES.iter().any(|m| from_module.starts_with(m));
            if !is_core {
                continue;
            }

            for to_module in deps {
                if to_module.starts_with("posix::") {
                    let severity =
                        if self.is_exempted(from_module, &ViolationType::KernelPosixDependency) {
                            Severity::Warning
                        } else {
                            Severity::Error
                        };

                    let from_layer = self.layers.get(from_module).copied().unwrap_or(Layer::Kernel);
                    violations.push(Violation {
                        from_module: from_module.clone(),
                        to_module: to_module.clone(),
                        from_layer,
                        to_layer: Layer::Syslib,
                        violation_type: ViolationType::KernelPosixDependency,
                        severity,
                    });
                }
            }
        }
    }

    /// Check kernel core modules for POSIX errno usage
    fn check_kernel_posix_errno(&self, violations: &mut Vec<Violation>) {
        for (from_module, deps) in &self.dependencies {
            let is_core = KERNEL_CORE_MODULES.iter().any(|m| from_module.starts_with(m));
            if !is_core {
                continue;
            }

            for to_module in deps {
                if to_module.contains("posix::errno") {
                    let severity =
                        if self.is_exempted(from_module, &ViolationType::KernelPosixErrnoUsage) {
                            Severity::Warning
                        } else {
                            Severity::Error
                        };

                    let from_layer = self.layers.get(from_module).copied().unwrap_or(Layer::Kernel);
                    violations.push(Violation {
                        from_module: from_module.clone(),
                        to_module: to_module.clone(),
                        from_layer,
                        to_layer: Layer::Syslib,
                        violation_type: ViolationType::KernelPosixErrnoUsage,
                        severity,
                    });
                }
            }
        }
    }

    /// Check circular dependencies using DFS
    fn check_circular_dependencies(&self, violations: &mut Vec<Violation>) {
        if let Some(cycles) = self.find_cycles() {
            for cycle in cycles {
                for i in 0..cycle.len() {
                    let from = &cycle[i];
                    let to = &cycle[(i + 1) % cycle.len()];

                    let from_layer = self.layers.get(from).copied().unwrap_or(Layer::Hal);
                    let to_layer = self.layers.get(to).copied().unwrap_or(Layer::Hal);

                    let severity = if self.is_exempted(from, &ViolationType::CircularDependency) {
                        Severity::Warning
                    } else {
                        Severity::Error
                    };

                    violations.push(Violation {
                        from_module: from.clone(),
                        to_module: to.clone(),
                        from_layer,
                        to_layer,
                        violation_type: ViolationType::CircularDependency,
                        severity,
                    });
                }
            }
        }
    }

    /// Find circular dependencies using DFS
    fn find_cycles(&self) -> Option<Vec<Vec<String>>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();
        let mut cycles = Vec::new();

        for module in self.dependencies.keys() {
            if !visited.contains(module) {
                self.dfs_find_cycles(module, &mut visited, &mut rec_stack, &mut path, &mut cycles);
            }
        }

        if cycles.is_empty() {
            None
        } else {
            Some(cycles)
        }
    }

    /// DFS helper for finding cycles
    fn dfs_find_cycles(
        &self,
        module: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(module.to_string());
        rec_stack.insert(module.to_string());
        path.push(module.to_string());

        if let Some(deps) = self.dependencies.get(module) {
            for dep in deps {
                if !visited.contains(dep) {
                    self.dfs_find_cycles(dep, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(dep) {
                    if let Some(start) = path.iter().position(|x| x == dep) {
                        let cycle: Vec<String> = path[start..].to_vec();
                        cycles.push(cycle);
                    }
                }
            }
        }

        path.pop();
        rec_stack.remove(module);
    }
}

/// Analyze Rust source file for dependencies
fn analyze_file(path: &Path) -> Vec<String> {
    let mut dependencies = Vec::new();

    if let Ok(content) = fs::read_to_string(path) {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("use ") {
                if let Some(dep) = parse_use_statement(trimmed) {
                    dependencies.push(dep);
                }
            }
        }
    }

    dependencies
}

/// Parse use statement
fn parse_use_statement(line: &str) -> Option<String> {
    let line = line.trim();
    if !line.starts_with("use ") {
        return None;
    }

    let line = &line[4..];
    let line = line.trim_end_matches(';');
    let line = line.trim();

    if line.starts_with("crate::") {
        let module = &line[7..];

        let module = if module.contains(" as ") {
            module.split(" as ").next().unwrap_or(module)
        } else if module.contains("::") && module.contains('{') {
            if let Some(pos) = module.rfind("::") {
                if module[pos..].starts_with("::{") {
                    &module[..pos]
                } else {
                    module
                }
            } else {
                module
            }
        } else {
            module
        };

        return Some(module.to_string());
    }

    if !line.starts_with("super::") && !line.starts_with("self::") && !line.starts_with('{') {
        if let Some(first_seg) = line.split("::").next() {
            if !first_seg.is_empty()
                && first_seg
                    .chars()
                    .next()
                    .map_or(true, |c| c.is_ascii_lowercase())
            {
                return Some(format!("external::{}", first_seg));
            }
        }
    }

    None
}

/// Main analysis function
fn analyze_project(root: &Path) -> Result<Vec<Violation>, Box<dyn std::error::Error>> {
    let exemptions = load_exemptions(root);
    if !exemptions.is_empty() {
        println!(
            "Loaded {} exemption(s) from dep_analyzer.toml",
            exemptions.len()
        );
        for ex in &exemptions {
            println!(
                "  - {} (reason: {}, deadline: {})",
                ex.module, ex.reason, ex.deadline
            );
        }
        println!();
    }

    let mut graph = DependencyGraph::new(exemptions);

    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.extension().map_or(false, |ext| ext == "rs") {
            let rel_path = path.strip_prefix(root)?;
            let module_path = path_to_module(rel_path);

            if let Some(layer) = Layer::from_path(&module_path) {
                graph.add_module(module_path.clone(), layer);

                let deps = analyze_file(path);
                for dep in deps {
                    graph.add_dependency(module_path.clone(), dep);
                }
            }
        }
    }

    Ok(graph.check_violations())
}

/// Convert file path to module path
fn path_to_module(path: &Path) -> String {
    let mut module = String::new();

    for component in path.components() {
        if let std::path::Component::Normal(s) = component {
            if let Some(s) = s.to_str() {
                if !module.is_empty() {
                    module.push_str("::");
                }
                let name = s.strip_suffix(".rs").unwrap_or(s);
                if name == "mod" {
                    continue;
                }
                module.push_str(name);
            }
        }
    }

    module
}

/// Main entry point
fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <project_root>", args[0]);
        eprintln!();
        eprintln!("Dependency Analyzer for Nuva OS Architecture Compliance");
        eprintln!();
        eprintln!("Checks:");
        eprintln!("  - Layer boundary violations (L0-L4)");
        eprintln!("  - HAL zero upward dependency");
        eprintln!("  - Kernel reverse dependency (L1 -> L2/L3/L4)");
        eprintln!("  - HAL concrete implementation references");
        eprintln!("  - Circular dependencies");
        eprintln!("  - Kernel core POSIX dependency");
        eprintln!("  - Kernel core POSIX errno usage");
        eprintln!();
        eprintln!("Configuration: dep_analyzer.toml (optional, for exemptions)");
        std::process::exit(1);
    }

    let root = PathBuf::from(&args[1]);

    println!("Nuva OS Dependency Analyzer");
    println!("==========================");
    println!("Analyzing dependencies in: {:?}", root);
    println!();

    match analyze_project(&root) {
        Ok(violations) => {
            let errors: Vec<&Violation> = violations
                .iter()
                .filter(|v| v.severity == Severity::Error)
                .collect();
            let warnings: Vec<&Violation> = violations
                .iter()
                .filter(|v| v.severity == Severity::Warning)
                .collect();

            if violations.is_empty() {
                println!("No dependency violations found!");
                println!("All modules comply with architectural layer boundaries.");
            } else {
                println!(
                    "Found {} violation(s): {} error(s), {} warning(s)",
                    violations.len(),
                    errors.len(),
                    warnings.len()
                );
                println!();

                for violation in &violations {
                    let severity_marker = match violation.severity {
                        Severity::Error => "ERROR",
                        Severity::Warning => "WARN ",
                    };

                    match violation.violation_type {
                        ViolationType::LayerViolation => {
                            println!(
                                "[{}] {} ({} -> {}):",
                                severity_marker,
                                violation.violation_type.description(),
                                violation.from_layer.name(),
                                violation.to_layer.name()
                            );
                            println!("  {} -> {}", violation.from_module, violation.to_module);
                        }
                        ViolationType::CircularDependency => {
                            println!(
                                "[{}] {}:",
                                severity_marker,
                                violation.violation_type.description()
                            );
                            println!("  {} -> {}", violation.from_module, violation.to_module);
                        }
                        ViolationType::DirectDependency => {
                            println!(
                                "[{}] {}:",
                                severity_marker,
                                violation.violation_type.description()
                            );
                            println!("  {} -> {}", violation.from_module, violation.to_module);
                        }
                        ViolationType::HalUpwardDependency => {
                            println!(
                                "[{}] {} ({} -> {}):",
                                severity_marker,
                                violation.violation_type.description(),
                                violation.from_layer.name(),
                                violation.to_layer.name()
                            );
                            println!("  {} -> {}", violation.from_module, violation.to_module);
                        }
                        ViolationType::HalConcreteImplReference => {
                            println!(
                                "[{}] {} ({} -> {}):",
                                severity_marker,
                                violation.violation_type.description(),
                                violation.from_layer.name(),
                                violation.to_layer.name()
                            );
                            println!(
                                "  {} -> {} (use HAL trait instead)",
                                violation.from_module, violation.to_module
                            );
                        }
                        ViolationType::KernelReverseDependency => {
                            println!(
                                "[{}] {} ({} -> {}):",
                                severity_marker,
                                violation.violation_type.description(),
                                violation.from_layer.name(),
                                violation.to_layer.name()
                            );
                            println!("  {} -> {}", violation.from_module, violation.to_module);
                        }
                    }
                    println!();
                }

                if !errors.is_empty() {
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error analyzing project: {}", e);
            std::process::exit(1);
        }
    }
}
