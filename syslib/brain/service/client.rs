/*
 * Nuva OS - System Library - Brain AI Service Client
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

/// Client state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientState {
    /// Disconnected
    Disconnected = 0,
    /// Connected
    Connected = 1,
    /// Error
    Error = 2,
}

/// AI service client
pub struct AiClient {
    /// Client ID
    client_id: AtomicU64,
    /// State
    state: AtomicU32,
    /// Server handle
    server_handle: AtomicU32,
    /// Request count
    request_count: AtomicU64,
}

impl AiClient {
    pub const fn new() -> Self {
        AiClient {
            client_id: AtomicU64::new(0),
            state: AtomicU32::new(ClientState::Disconnected as u32),
            server_handle: AtomicU32::new(0),
            request_count: AtomicU64::new(0),
        }
    }

    /// Connect to a service
    pub fn connect(&mut self, _server_name: &str) -> i32 {
        if self.state.load(Ordering::Acquire) == ClientState::Connected as u32 {
            return -1;
        }

        // TODO: Connect to AI service via IPC

        self.state.store(ClientState::Connected as u32, Ordering::Release);

        log_info!("AI client connected");
        0
    }

    /// Disconnect from the service
    pub fn disconnect(&mut self) -> i32 {
        if self.state.load(Ordering::Acquire) != ClientState::Connected as u32 {
            return -1;
        }

        // Disconnect from IPC service:
        let server_handle = self.server_handle.load(Ordering::Acquire);
        let client_id = self.client_id.load(Ordering::Acquire);
        if server_handle != 0 && client_id != 0 {
            crate::kernel::ipc::nuvaipc::disconnect_from_service(server_handle, client_id);
        }
        self.server_handle.store(0, Ordering::Release);
        self.client_id.store(0, Ordering::Release);

        self.state.store(ClientState::Disconnected as u32, Ordering::Release);

        log_info!("AI client disconnected");
        0
    }

    /// Get the client state
    pub fn get_state(&self) -> ClientState {
        match self.state.load(Ordering::Acquire) {
            0 => ClientState::Disconnected,
            1 => ClientState::Connected,
            2 => ClientState::Error,
            _ => ClientState::Disconnected,
        }
    }

    /// Load a model via the service
    pub fn load_model(&mut self, model_path: &str) -> Option<u64> {
        if self.get_state() != ClientState::Connected {
            return None;
        }

        self.request_count.fetch_add(1, Ordering::AcqRel);

        // Send load_model request via IPC
        let server_handle = self.server_handle.load(Ordering::Acquire);
        let request = model_path.as_bytes();
        let mut response = [0u8; 8];
        let result = crate::kernel::ipc::nuvaipc::send_request(
            server_handle, request, &mut response
        );

        log_debug!("Client: load_model({})", model_path);
        if result == 0 && response.len() >= 8 {
            Some(u64::from_le_bytes(response[..8].try_into().unwrap_or([0; 8])))
        } else {
            None
        }
    }

    /// Execute inference via the service
    pub fn infer(&mut self, model_id: u64, input: &[u8], output: &mut [u8]) -> i32 {
        if self.get_state() != ClientState::Connected {
            return -1;
        }

        self.request_count.fetch_add(1, Ordering::AcqRel);

        // Send inference request via IPC
        let server_handle = self.server_handle.load(Ordering::Acquire);
        let result = crate::kernel::ipc::nuvaipc::send_request(
            server_handle, input, output
        );

        log_debug!("Client: infer(model={}, size={})", model_id, input.len());
        result
    }

    /// Execute asynchronous inference via the service
    pub fn infer_async(&mut self, model_id: u64, input: &[u8]) -> Option<u64> {
        if self.get_state() != ClientState::Connected {
            return None;
        }

        self.request_count.fetch_add(1, Ordering::AcqRel);

        // Send async inference request via IPC
        let server_handle = self.server_handle.load(Ordering::Acquire);
        let task_id = crate::kernel::ipc::nuvaipc::send_async_request(
            server_handle, input
        );

        log_debug!("Client: infer_async(model={})", model_id);
        if task_id != 0 { Some(task_id) } else { None }
    }

    /// Get the request count
    pub fn get_request_count(&self) -> u64 {
        self.request_count.load(Ordering::Acquire)
    }
}

/// Client manager
pub struct ClientManager {
    /// Client array
    clients: [Option<AiClient>; 16],
    /// Client count
    num_clients: u32,
}

impl ClientManager {
    pub const fn new() -> Self {
        ClientManager {
            clients: [None; 16],
            num_clients: 0,
        }
    }

    /// Create a client
    pub fn create_client(&mut self) -> Option<u32> {
        for (i, slot) in self.clients.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(AiClient::new());
                self.num_clients += 1;
                return Some(i as u32);
            }
        }
        None
    }

    /// Destroy a client
    pub fn destroy_client(&mut self, client_id: u32) -> i32 {
        if (client_id as usize) < self.clients.len() {
            self.clients[client_id as usize] = None;
            self.num_clients -= 1;
            return 0;
        }
        -1
    }

    /// Get a client by ID
    pub fn get_client(&mut self, client_id: u32) -> Option<&mut AiClient> {
        self.clients.get_mut(client_id as usize)?.as_mut()
    }
}

static CLIENT_MANAGER: crate::sync_oncelock::OnceLock<ClientManager> = crate::sync_oncelock::OnceLock::new();

/// Get the global client manager instance
pub fn get_client_manager() -> &'static mut ClientManager {
    // SAFETY: Single-threaded access; synchronized externally.
    unsafe { &mut CLIENT_MANAGER }
}
