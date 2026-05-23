# Services — System Services Layer (L3)

## Overview

The Services layer (Layer 3) provides OS-level system services, running in user space under the microkernel architecture. It includes core services such as application management, IPC, networking, power, and security.

## Submodules

| Submodule | Description |
|-----------|-------------|
| app/ | Application service: Activity lifecycle, installer, package manager |
| ipc/ | IPC service: Binder, channels, shared memory |
| net/ | Network service: DNS resolution, interface management, TCP/UDP |
| power/ | Power service: power manager, policies, suspend, wake locks |
| security/ | Security service: Gatekeeper, Keymaster, permission management, TEE client |
| form_factor.rs | Form factor manager (phone/tablet/desktop/server adaptive) |

## Dependencies

- **Lower dependencies**: hal (L0), kernel (L1), syslib (L2)
- **Depended by**: application (L4)

## Build Configuration

The services layer is compiled together with the kernel and supports different device types via conditional compilation:

- Mobile devices: enable app, power, security services
- Server devices: enable ipc, net services

## Public Interface

Each service exposes its interface through Binder IPC, supporting both synchronous and asynchronous calls:
- `AppService` — Application lifecycle management
- `PowerService` — Power policy and state management
- `SecurityService` — Permission and key management
- `NetService` — Network configuration and connection management
