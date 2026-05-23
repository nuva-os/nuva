# Network Stack

## Overview

The `kernel/net_stack/` module provides the in-kernel TCP/IP protocol stack and socket API implementation.

## Module Structure

| File | Description |
|------|-------------|
| `mod.rs` | Module entry point and network stack initialization |
| `tcpip.rs` | TCP/IP protocol stack implementation |
| `socket.rs` | Socket API (TCP/UDP/RAW sockets) |

## Initialization

1. TCP/IP stack initialization (`tcpip::init_tcpip`)
2. Socket API initialization (`socket::init_socket_api`)

Both initialize in Phase 7 (I/O & Networking), after block devices and the core kernel services are ready.

## Dependencies

- **Internal dependencies**: `kernel/core` (workqueue, time, mempool), `kernel/storage` (block), `kernel/irq_mgmt` (IRQ for NIC interrupts), `kernel/net` (protocol implementations)
- **Depended by**: Network applications, socket-based services (L3), NFS/SMB network clients

## Public Interface

- `tcpip` module: TCP/IP protocol stack (`init_tcpip()`, `tcp_connect()`, `tcp_send()`, `tcp_recv()`, `udp_sendto()`, `udp_recvfrom()`)
- `socket` module: BSD-compatible Socket API (`init_socket_api()`, `socket()`, `bind()`, `listen()`, `accept()`, `connect()`, `send()`, `recv()`)

## Backward-Compatible Re-exports

The following items are re-exported from `kernel/` for backward compatibility:
- `socket`, `tcpip`

---

*Last updated: 2026-05-20 | Nuva OS v1.0.0*
