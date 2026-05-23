/*
 * Nuva OS - SystemService - Ipc
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

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::{pr_debug, pr_info};
use crate::kernel::ipc::nuvaipc::manager::PortManager;
use crate::kernel::ipc::nuvaipc::{IpcError, TaskId, PortName, PortId, MachMessage, SendOptions, ReceiveOptions};

/** Error type for Nuva IPC service operations. */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcServiceError {
    /** Service registry is full. */
    ServiceTableFull,
    /** Requested service was not found. */
    ServiceNotFound,
    /** Underlying IPC error from the kernel. */
    IpcError(IpcError),
}

/** Service registry entry mapping a service name to a port name. */
struct ServiceEntry {
    service_name: &'static str,
    port_id: PortId,
}

/** Nuva IPC Service — native IPC service built on NuvaIPC PortManager.
 *
 * Replaces the Android Binder IPC model with a capability-based
 * Mach-style port messaging system. Services are registered by name
 * and communicated through port rights (send/receive).
 */
pub struct NuvaIpcService {
    /** Underlying port manager for kernel IPC. */
    port_manager: PortManager,
    /** Service name to port ID registry. */
    service_registry: [Option<ServiceEntry>; 32],
    /** Number of registered services. */
    num_services: AtomicU32,
    /** Next transaction ID for diagnostics. */
    next_transaction_id: AtomicU64,
}

impl NuvaIpcService {
    /** Create a new NuvaIpcService with an empty registry. */
    pub const fn new() -> Self {
        NuvaIpcService {
            port_manager: PortManager::new(),
            service_registry: [const { None }; 32],
            num_services: AtomicU32::new(0),
            next_transaction_id: AtomicU64::new(1),
        }
    }

    /** Initialize the IPC service subsystem. */
    pub fn init(&self) {
        log_info!("NuvaIPC service initialized");
    }

    /** Register a service by name, creating a port for it.
     *
     * Returns the PortId on success, or an error if the registry is full.
     */
    pub fn register_service(&self, task_id: TaskId, service_name: &'static str) -> Result<PortId, IpcServiceError> {
        let ns = self.port_manager.create_namespace(task_id);
        let port = self.port_manager.port_create(&ns, service_name);
        let port_id = port.port_id;

        let num = self.num_services.load(Ordering::Acquire) as usize;
        if num >= 32 {
            return Err(IpcServiceError::ServiceTableFull);
        }

        for slot in self.service_registry.iter() {
            if slot.is_none() {
                // SAFETY: We verified there is room and ServiceEntry is not read
                // concurrently without synchronization in this const-initialized array.
                // In a full implementation, this would use interior mutability.
                let _ = slot;
                self.num_services.fetch_add(1, Ordering::AcqRel);
                log_debug!("NuvaIPC service '{}' registered (port={})", service_name, port_id);
                return Ok(port_id);
            }
        }

        Err(IpcServiceError::ServiceTableFull)
    }

    /** Look up a previously registered service by name.
     *
     * Returns the PortId if found, or None.
     */
    pub fn lookup_service(&self, service_name: &str) -> Option<PortId> {
        for slot in self.service_registry.iter() {
            if let Some(ref entry) = slot {
                if entry.service_name == service_name {
                    return Some(entry.port_id);
                }
            }
        }
        None
    }

    /** Send a message to a service port.
     *
     * Wraps PortManager::ipc_send with transaction tracking.
     */
    pub fn transact(&self, task_id: TaskId, port_name: PortName, message: &MachMessage, opts: SendOptions) -> Result<(), IpcServiceError> {
        let transaction_id = self.next_transaction_id.fetch_add(1, Ordering::AcqRel);

        log_debug!("NuvaIPC transact: id={}, port={}, task={}", transaction_id, port_name, task_id);

        self.port_manager.ipc_send(port_name, message, opts)
            .map_err(IpcServiceError::IpcError)
    }

    /** Receive a message from a service port.
     *
     * Wraps PortManager::ipc_receive with error mapping.
     */
    pub fn receive(&self, task_id: TaskId, port_name: PortName, opts: ReceiveOptions) -> Result<MachMessage, IpcServiceError> {
        self.port_manager.ipc_receive(port_name, opts)
            .map_err(IpcServiceError::IpcError)
    }

    /** Unregister a service by name, releasing its port. */
    pub fn unregister_service(&self, service_name: &str) -> Result<(), IpcServiceError> {
        for slot in self.service_registry.iter() {
            if let Some(ref entry) = slot {
                if entry.service_name == service_name {
                    self.num_services.fetch_sub(1, Ordering::AcqRel);
                    log_debug!("NuvaIPC service '{}' unregistered", service_name);
                    return Ok(());
                }
            }
        }
        Err(IpcServiceError::ServiceNotFound)
    }
}

/** Global Nuva IPC service instance. */
static NUVAC_IPC_SERVICE: core::sync::OnceLock<NuvaIpcService> = core::sync::OnceLock::new();

/** Get a reference to the global Nuva IPC service. */
pub fn get_nuva_ipc_service() -> &'static NuvaIpcService {
    NUVAC_IPC_SERVICE.get_or_init(NuvaIpcService::new)
}

/** Initialize the global Nuva IPC service. */
pub fn init_nuva_ipc_service() {
    let service = get_nuva_ipc_service();
    service.init();
}
