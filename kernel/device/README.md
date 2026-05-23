# Device Model and Plugins

## Overview

The `kernel/device/` module provides the device model framework, driver/feature plugin systems, module loader, and notifier chains for device event propagation.

## Module Structure

| File | Description |
|------|-------------|
| `mod.rs` | Module entry point |
| `device_model.rs` | Device model abstraction and bus/class/driver model |
| `driver_plugin.rs` | Driver plugin system for extensible driver registration |
| `feature_plugin.rs` | Feature plugin system for runtime feature loading |
| `module.rs` | Kernel module loader |
| `notifier.rs` | Notifier chain for device event propagation |

## Initialization Order

1. Device model initialization (`device_model::init_device_model`)
2. Driver plugin system (`driver_plugin::init_driver_plugin`)
3. Feature plugin system (`feature_plugin::init_feature_plugin`)
4. Module loader (`module::init_module`)
5. Notifier chain (`notifier::init_notifier`)

All device subsystem components initialize in Phase 3 (Device & Plugin), after memory management and IRQ are operational.

## Dependencies

- **Internal dependencies**: `kernel/core` (CPU, mempool, workqueue), `kernel/init` (elf), `kernel/irq_mgmt` (IRQ), `kernel/sync`
- **Depended by**: All device drivers, power management, storage, networking, plugin extensions

## Public Interface

- `device_model` module: Unified device model with bus/class/driver abstractions (`init_device_model()`, `register_device()`, `register_driver()`, `device_create()`)
- `driver_plugin` module: Driver plugin system for extensible driver registration (`init_driver_plugin()`, `register_driver_plugin()`)
- `feature_plugin` module: Feature plugin system for runtime feature loading and activation (`init_feature_plugin()`, `load_feature_plugin()`)
- `module` module: Kernel module loader supporting dynamic module insertion and removal (`init_module()`, `load_module()`, `unload_module()`)
- `notifier` module: Notifier chain for device event propagation (`init_notifier()`, `register_notifier()`, `notify_event()`)

## Backward-Compatible Re-exports

The following items are re-exported from `kernel/` for backward compatibility:
- `device_model`, `driver_plugin`, `feature_plugin`, `module`, `notifier`

---

*Last updated: 2026-05-20 | Nuva OS v1.0.0*
