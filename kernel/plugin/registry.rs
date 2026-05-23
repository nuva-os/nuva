/*
 * Plugin Registry - Plugin Tracking and Lookup
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module maintains the registry of all loaded plugins
 * and provides efficient lookup operations.
 */

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use super::core::{PluginId, PluginMeta, PluginType};

/// Plugin registry
/// Maintains metadata and lookup tables for all loaded plugins.
pub struct PluginRegistry {
    /// Plugin metadata by ID
    plugins: BTreeMap<PluginId, PluginMeta>,

    /// Name to ID mapping
    name_index: BTreeMap<String, PluginId>,

    /// Type to IDs mapping
    type_index: BTreeMap<PluginType, Vec<PluginId>>,

    /// Dependency graph
    dependency_graph: DependencyGraph,
}

impl PluginRegistry {
    /// Create new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: BTreeMap::new(),
            name_index: BTreeMap::new(),
            type_index: BTreeMap::new(),
            dependency_graph: DependencyGraph::new(),
        }
    }

    /// Register plugin
    /// @param id: Plugin ID
    /// @param meta: Plugin metadata
    pub fn register(&mut self, id: PluginId, meta: PluginMeta) {
        // Add to main map
        self.plugins.insert(id, meta.clone());

        // Add to name index
        self.name_index.insert(String::from(meta.name), id);

        // Add to type index
        self.type_index
            .entry(meta.plugin_type)
            .or_insert_with(Vec::new)
            .push(id);

        // Add to dependency graph
        self.dependency_graph.add_plugin(id, &meta);
    }

    /// Unregister plugin
    /// @param id: Plugin ID
    pub fn unregister(&mut self, id: PluginId) {
        // Remove from main map
        if let Some(meta) = self.plugins.remove(&id) {
            // Remove from name index
            self.name_index.remove(meta.name);

            // Remove from type index
            if let Some(ids) = self.type_index.get_mut(&meta.plugin_type) {
                ids.retain(|&x| x != id);
            }
        }

        // Remove from dependency graph
        self.dependency_graph.remove_plugin(id);
    }

    /// Get plugin metadata
    /// @param id: Plugin ID
    /// @return: Plugin metadata
    pub fn get_meta(&self, id: PluginId) -> Option<&PluginMeta> {
        self.plugins.get(&id)
    }

    /// Find plugin by name
    /// @param name: Plugin name
    /// @return: Plugin ID
    pub fn find_by_name(&self, name: &str) -> Option<PluginId> {
        self.name_index.get(name).copied()
    }

    /// List plugins by type
    /// @param plugin_type: Plugin type
    /// @return: List of plugin IDs
    pub fn list_by_type(&self, plugin_type: PluginType) -> Vec<PluginId> {
        self.type_index
            .get(&plugin_type)
            .cloned()
            .unwrap_or_default()
    }

    /// Check if plugin is registered
    /// @param id: Plugin ID
    /// @return: true if registered
    pub fn is_registered(&self, id: PluginId) -> bool {
        self.plugins.contains_key(&id)
    }

    /// Get total number of plugins
    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    /// Iterate over all plugins
    pub fn iter(&self) -> impl Iterator<Item = (&PluginId, &PluginMeta)> {
        self.plugins.iter()
    }

    /// Get dependency graph
    pub fn dependency_graph(&self) -> &DependencyGraph {
        &self.dependency_graph
    }

    /// Get plugins that depend on a given plugin
    /// @param id: Plugin ID
    /// @return: List of dependent plugin IDs
    pub fn get_dependents(&self, id: PluginId) -> Vec<PluginId> {
        self.dependency_graph.get_dependents(id)
    }

    /// Get dependencies of a given plugin
    /// @param id: Plugin ID
    /// @return: List of dependency plugin IDs
    pub fn get_dependencies(&self, id: PluginId) -> Vec<PluginId> {
        self.dependency_graph.get_dependencies(id)
    }

    /// Check for circular dependencies
    /// @return: true if circular dependencies exist
    pub fn has_circular_dependencies(&self) -> bool {
        self.dependency_graph.has_cycles()
    }

    /// Get load order (topological sort)
    /// @return: Plugin IDs in load order
    pub fn get_load_order(&self) -> Result<Vec<PluginId>, &'static str> {
        self.dependency_graph.topological_sort()
    }
}

/// Dependency graph
/// Tracks plugin dependencies and provides graph operations.
pub struct DependencyGraph {
    /// Adjacency list: plugin -> dependencies
    dependencies: BTreeMap<PluginId, Vec<PluginId>>,

    /// Reverse adjacency list: plugin -> dependents
    dependents: BTreeMap<PluginId, Vec<PluginId>>,
}

impl DependencyGraph {
    /// Create new dependency graph
    pub fn new() -> Self {
        Self {
            dependencies: BTreeMap::new(),
            dependents: BTreeMap::new(),
        }
    }

    /// Add plugin to graph
    /// @param id: Plugin ID
    /// @param meta: Plugin metadata
    pub fn add_plugin(&mut self, id: PluginId, meta: &PluginMeta) {
        // Initialize adjacency lists
        self.dependencies.entry(id).or_insert_with(Vec::new);
        self.dependents.entry(id).or_insert_with(Vec::new);

        // Note: Actual dependency edges are added when
        // dependency plugins are registered
    }

    /// Remove plugin from graph
    /// @param id: Plugin ID
    pub fn remove_plugin(&mut self, id: PluginId) {
        // Remove from dependencies
        if let Some(deps) = self.dependencies.remove(&id) {
            // Remove from dependents of dependencies
            for dep_id in deps {
                if let Some(deps) = self.dependents.get_mut(&dep_id) {
                    deps.retain(|&x| x != id);
                }
            }
        }

        // Remove from dependents
        if let Some(deps) = self.dependents.remove(&id) {
            // Remove from dependencies of dependents
            for dep_id in deps {
                if let Some(deps) = self.dependencies.get_mut(&dep_id) {
                    deps.retain(|&x| x != id);
                }
            }
        }
    }

    /// Add dependency edge
    /// @param from: Plugin that has dependency
    /// @param to: Dependency plugin
    pub fn add_dependency(&mut self, from: PluginId, to: PluginId) {
        self.dependencies
            .entry(from)
            .or_insert_with(Vec::new)
            .push(to);

        self.dependents
            .entry(to)
            .or_insert_with(Vec::new)
            .push(from);
    }

    /// Get dependencies of a plugin
    /// @param id: Plugin ID
    /// @return: List of dependency IDs
    pub fn get_dependencies(&self, id: PluginId) -> Vec<PluginId> {
        self.dependencies.get(&id).cloned().unwrap_or_default()
    }

    /// Get dependents of a plugin
    /// @param id: Plugin ID
    /// @return: List of dependent IDs
    pub fn get_dependents(&self, id: PluginId) -> Vec<PluginId> {
        self.dependents.get(&id).cloned().unwrap_or_default()
    }

    /// Check for cycles using DFS
    /// @return: true if cycles exist
    pub fn has_cycles(&self) -> bool {
        let mut visited = BTreeMap::new();

        for id in self.dependencies.keys() {
            if self.dfs_has_cycle(*id, &mut visited) {
                return true;
            }
        }

        false
    }

    /// DFS helper for cycle detection
    fn dfs_has_cycle(&self, id: PluginId, visited: &mut BTreeMap<PluginId, VisitState>) -> bool {
        use VisitState::*;

        match visited.get(&id) {
            Some(&Visiting) => return true, // Found cycle
            Some(&Visited) => return false, // Already processed
            None => {}
        }

        visited.insert(id, Visiting);

        if let Some(deps) = self.dependencies.get(&id) {
            for dep_id in deps {
                if self.dfs_has_cycle(*dep_id, visited) {
                    return true;
                }
            }
        }

        visited.insert(id, Visited);
        false
    }

    /// Topological sort for load order
    /// @return: Plugin IDs in load order
    pub fn topological_sort(&self) -> Result<Vec<PluginId>, &'static str> {
        if self.has_cycles() {
            return Err("Circular dependencies detected");
        }

        let mut result = Vec::new();
        let mut visited = BTreeMap::new();

        for id in self.dependencies.keys() {
            self.dfs_topo(*id, &mut visited, &mut result);
        }

        // Reverse to get correct order
        result.reverse();
        Ok(result)
    }

    /// DFS helper for topological sort
    fn dfs_topo(
        &self,
        id: PluginId,
        visited: &mut BTreeMap<PluginId, bool>,
        result: &mut Vec<PluginId>,
    ) {
        if visited.get(&id) == Some(&true) {
            return;
        }

        visited.insert(id, true);

        if let Some(deps) = self.dependencies.get(&id) {
            for dep_id in deps {
                self.dfs_topo(*dep_id, visited, result);
            }
        }

        result.push(id);
    }
}

/// Visit state for DFS
enum VisitState {
    Visiting,
    Visited,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
