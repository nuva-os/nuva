/*
 * Nuva OS - System Library - Brain AI Service Server
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

/// Server state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerState {
    /// Stopped
    Stopped = 0,
    /// Running
    Running = 1,
    /// Suspended
    Suspended = 2,
    /// Error
    Error = 3,
}

/// AI service server
pub struct AiServer {
    /// Service name
    pub name: &'static str,
    /// State
    state: AtomicU32,
    /// Client count
    num_clients: AtomicU32,
    /// Processed request count
    processed_requests: AtomicU64,
    /// Service handle
    handle: AtomicU32,
}

impl AiServer {
    pub const fn new(name: &'static str) -> Self {
        AiServer {
            name,
            state: AtomicU32::new(ServerState::Stopped as u32),
            num_clients: AtomicU32::new(0),
            processed_requests: AtomicU64::new(0),
            handle: AtomicU32::new(0),
        }
    }

    /// Start the service
    pub fn start(&mut self) -> i32 {
        if self.state.load(Ordering::Acquire) == ServerState::Running as u32 {
            return -1;
        }

        // TODO: Register IPC service
        // 1. Create IPC service port
        // 2. Register service name
        // 3. Start request handling thread

        self.state.store(ServerState::Running as u32, Ordering::Release);

        log_info!("AI server '{}' started", self.name);
        0
    }

    /// Stop the service
    pub fn stop(&mut self) -> i32 {
        if self.state.load(Ordering::Acquire) != ServerState::Running as u32 {
            return -1;
        }

        // Stop IPC service:
        // 1. Disconnect all connected clients
        let num_clients = self.num_clients.load(Ordering::Acquire);
        if num_clients > 0 {
            log_info!("AI server '{}': disconnecting {} clients", self.name, num_clients);
        }

        // 2. Unregister service from IPC name server
        let handle = self.handle.load(Ordering::Acquire);
        if handle != 0 {
            crate::kernel::ipc::nuvaipc::unregister_service(handle);
        }

        // 3. Stop request handling thread
        // The handler thread would be signaled to exit

        self.state.store(ServerState::Stopped as u32, Ordering::Release);

        log_info!("AI server '{}' stopped", self.name);
        0
    }

    /// Suspend the service
    pub fn suspend(&mut self) -> i32 {
        if self.state.load(Ordering::Acquire) != ServerState::Running as u32 {
            return -1;
        }

        self.state.store(ServerState::Suspended as u32, Ordering::Release);

        log_info!("AI server '{}' suspended", self.name);
        0
    }

    /// Resume the service
    pub fn resume(&mut self) -> i32 {
        if self.state.load(Ordering::Acquire) != ServerState::Suspended as u32 {
            return -1;
        }

        self.state.store(ServerState::Running as u32, Ordering::Release);

        log_info!("AI server '{}' resumed", self.name);
        0
    }

    /// Get the server state
    pub fn get_state(&self) -> ServerState {
        match self.state.load(Ordering::Acquire) {
            0 => ServerState::Stopped,
            1 => ServerState::Running,
            2 => ServerState::Suspended,
            3 => ServerState::Error,
            _ => ServerState::Stopped,
        }
    }

    /// Process a client request
    pub fn process_request(&mut self, _client_id: u32, _request: &[u8], _response: &mut [u8]) -> i32 {
        if self.state.load(Ordering::Acquire) != ServerState::Running as u32 {
            return -1;
        }

        // TODO: Handle client request
        // 1. Parse request
        // 2. Call the corresponding API
        // 3. Build response

        self.processed_requests.fetch_add(1, Ordering::AcqRel);

        0
    }

    /// Handle client connection
    pub fn on_client_connect(&mut self) {
        self.num_clients.fetch_add(1, Ordering::AcqRel);
        log_debug!("Client connected, total: {}", self.num_clients.load(Ordering::Acquire));
    }

    /// Handle client disconnection
    pub fn on_client_disconnect(&mut self) {
        self.num_clients.fetch_sub(1, Ordering::AcqRel);
        log_debug!("Client disconnected, total: {}", self.num_clients.load(Ordering::Acquire));
    }

    /// Get statistics
    pub fn get_stats(&self) -> (u32, u64) {
        let num_clients = self.num_clients.load(Ordering::Acquire);
        let processed = self.processed_requests.load(Ordering::Acquire);
        (num_clients, processed)
    }
}

/// Server manager
pub struct ServerManager {
    /// Server array
    servers: [Option<AiServer>; 4],
    /// Server count
    num_servers: u32,
}

impl ServerManager {
    pub const fn new() -> Self {
        ServerManager {
            servers: [None; 4],
            num_servers: 0,
        }
    }

    /// Create a server
    pub fn create_server(&mut self, name: &'static str) -> Option<u32> {
        for (i, slot) in self.servers.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(AiServer::new(name));
                self.num_servers += 1;
                return Some(i as u32);
            }
        }
        None
    }

    /// Destroy a server
    pub fn destroy_server(&mut self, server_id: u32) -> i32 {
        if (server_id as usize) < self.servers.len() {
            self.servers[server_id as usize] = None;
            self.num_servers -= 1;
            return 0;
        }
        -1
    }

    /// Get a server by ID
    pub fn get_server(&mut self, server_id: u32) -> Option<&mut AiServer> {
        self.servers.get_mut(server_id as usize)?.as_mut()
    }
}

static SERVER_MANAGER: crate::sync_oncelock::OnceLock<ServerManager> = crate::sync_oncelock::OnceLock::new();

/// Get the global server manager instance
pub fn get_server_manager() -> &'static mut ServerManager {
    // SAFETY: Single-threaded access; synchronized externally.
    unsafe { &mut SERVER_MANAGER }
}

/// Initialize the AI server
pub fn init_ai_server() {
    let manager = get_server_manager();

    // Create default AI service
    if let Some(server_id) = manager.create_server("nuva.ai") {
        if let Some(server) = manager.get_server(server_id) {
            server.start();
        }
    }
}
