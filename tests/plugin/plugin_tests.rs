/*
 * Plugin System Tests
 *
 * Copyright (C) 2026 Nuva OS Team
 */

use crate::kernel::plugin::*;

/// Mock plugin for testing
struct MockPlugin {
    meta: PluginMeta,
    state: PluginState,
}

impl MockPlugin {
    fn new(name: &str) -> Self {
        Self {
            meta: PluginMeta {
                name: String::from(name),
                version: String::from("1.0.0"),
                plugin_type: PluginType::Driver,
                description: String::from("Mock plugin for testing"),
                author: String::from("Test"),
                dependencies: Vec::new(),
            },
            state: PluginState::Unloaded,
        }
    }
}

impl Plugin for MockPlugin {
    fn meta(&self) -> &PluginMeta {
        &self.meta
    }

    fn init(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        self.state = PluginState::Initialized;
        Ok(())
    }

    fn start(&mut self) -> Result<(), PluginError> {
        if self.state != PluginState::Initialized {
            return Err(PluginError::InvalidState);
        }
        self.state = PluginState::Running;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), PluginError> {
        if self.state != PluginState::Running {
            return Err(PluginError::InvalidState);
        }
        self.state = PluginState::Stopped;
        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), PluginError> {
        self.state = PluginState::Unloaded;
        Ok(())
    }
}

#[test]
fn test_plugin_lifecycle() {
    let mut plugin = MockPlugin::new("test_plugin");
    
    // Initial state
    assert_eq!(plugin.meta().name, "test_plugin");
    
    // Initialize
    let ctx = PluginContext::default();
    plugin.init(&ctx).unwrap();
    
    // Start
    plugin.start().unwrap();
    
    // Stop
    plugin.stop().unwrap();
    
    // Cleanup
    plugin.cleanup().unwrap();
}

#[test]
fn test_plugin_registry() {
    let registry = PluginRegistry::new();
    
    // Register plugin
    let plugin = MockPlugin::new("test_plugin");
    registry.register(plugin).unwrap();
    
    // Check registration
    assert!(registry.is_registered("test_plugin"));
    
    // Unregister
    registry.unregister("test_plugin").unwrap();
    assert!(!registry.is_registered("test_plugin"));
}

#[test]
fn test_plugin_dependencies() {
    let mut registry = PluginRegistry::new();
    
    // Create plugins with dependencies
    let plugin1 = MockPlugin::new("plugin1");
    let mut plugin2 = MockPlugin::new("plugin2");
    plugin2.meta.dependencies.push(String::from("plugin1"));
    
    // Register in order
    registry.register(plugin1).unwrap();
    registry.register(plugin2).unwrap();
    
    // Check dependency order
    let order = registry.get_load_order().unwrap();
    assert_eq!(order[0], "plugin1");
    assert_eq!(order[1], "plugin2");
}
