# 网络协议栈

## 概述

`kernel/net_stack/` 模块提供内核内 TCP/IP 协议栈和 Socket API 实现。

## 模块结构

| 文件 | 描述 |
|------|------|
| `mod.rs` | 模块入口和网络协议栈初始化 |
| `tcpip.rs` | TCP/IP 协议栈实现 |
| `socket.rs` | Socket API（TCP/UDP/RAW 套接字） |

## 初始化

1. TCP/IP 协议栈初始化（`tcpip::init_tcpip`）
2. Socket API 初始化（`socket::init_socket_api`）

两者都在阶段 7（I/O 与网络）中初始化，在块设备和核心内核服务就绪之后进行。

## 依赖关系

- **内部依赖**：`kernel/core`（workqueue、time、mempool）、`kernel/storage`（block）、`kernel/irq_mgmt`（NIC 中断 IRQ）、`kernel/net`（协议实现）
- **上层被依赖**：网络应用程序、基于 Socket 的服务（L3）、NFS/SMB 网络客户端

## 公开接口

- `tcpip` 模块：TCP/IP 协议栈（`init_tcpip()`、`tcp_connect()`、`tcp_send()`、`tcp_recv()`、`udp_sendto()`、`udp_recvfrom()`）
- `socket` 模块：BSD 兼容的 Socket API（`init_socket_api()`、`socket()`、`bind()`、`listen()`、`accept()`、`connect()`、`send()`、`recv()`）

## 向后兼容重导出

以下项从 `kernel/` 重导出以保持向后兼容性：
- `socket`、`tcpip`

---

*最后更新：2026-05-20 | Nuva OS v1.0.0*
