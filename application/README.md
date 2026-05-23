# Application — Application Framework Layer (L4)

## Overview

The Application layer (Layer 4) is the topmost layer of Nuva OS, providing the application-facing UI framework and rendering engine. It offers interfaces for window management, event handling, UI components, and resource management to application developers.

## Submodules

| Submodule | Description |
|-----------|-------------|
| ui/ | UI framework: adaptive layout, layout engine, style system, component library |
| window/ | Window management: manager, window, Surface |
| event/ | Event system: dispatcher, event types, handlers |
| render/ | Rendering engine: compositor, render context, painter |
| resource/ | Resource management: cache, loader, decoder (JPEG/PNG/TTF/WAV) |

## Dependencies

- **Lower dependencies**: hal (L0), kernel (L1), syslib (L2), services (L3)
- **Depended by**: None (topmost layer)

## Build Configuration

The application framework layer is compiled together with the kernel, and the form factor adapts via `services::form_factor`:
- Phone/Tablet: touch-optimized UI, compact layout
- Desktop: windowed UI, multi-display support
- Server: no UI, API-only interface

## Public Interface

- `Application` — Application entry and lifecycle
- `Window` — Window creation and management
- `View` — UI component base class
- `EventDispatcher` — Event dispatch and handling
- `Renderer` — Render context and drawing
