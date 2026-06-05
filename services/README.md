# Services — System Services Layer (L3)

## Overview

The Services layer (Layer 3) provides OS-level system services, running in user space under the microkernel architecture. It includes core services such as application management, IPC, networking, power, and security.

## Submodules

| Submodule | Description |
|-----------|-------------|
| app/ | Application management service (install, lifecycle, package, screen) |
| ipc/ | IPC service (Binder, Channel, Shared Memory) |
| net/ | Network service (DNS, TCP/UDP, Interface management) |
| power/ | Power management service (PM policy, wake lock, suspend) |
| security/ | Security service (Gatekeeper, Keymaster, Permission, TEE client) |
| form_factor/ | Form factor adaptation service (mobile/tablet/PC detection) |
| audio/ | Audio service (codec, mixer, resampler, volume, power management) |
| video/ | Video service (H.264/H.265/VP9/AV1 codec, HW acceleration, frame buffer) |
| web/ | Web service (HTML/CSS parser, JS engine, DOM, layout, page rendering) |
| opengl/ | OpenGL service (GPU rendering, pipeline, resource management, software fallback) |
| sqlite/ | SQLite database service (B-tree, WAL, connection pool, query executor) |
| image/ | Image service (JPEG/PNG/GIF/WebP/BMP codec, transform, HW acceleration) |
| core_processing/ | Core processing service (format detection, shared memory transfer, power coordination) |

## Dependencies

- **Lower dependencies**: hal (L0), kernel (L1), syslib (L2)
- **Depended by**: application (L4)

### Build Configuration

Services are compiled with the kernel. Feature flags control service inclusion:
- `mobile` profile: Enables all 13 core services for mobile/tablet deployment
- `server` profile: Enables network, security, sqlite, and web services
- Services communicate via Nuva IPC ports with capability-gated access

## Public Interface

Provides the following service interfaces: `AppService`, `IpcService`, `NetService`, `PowerService`, `SecurityService`, `AudioService`, `VideoService`, `WebService`, `OpenGLService`, `SqliteService`, `ImageService`, `CoreProcessingService`, `FormFactorService`.
