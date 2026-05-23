/*
 * Nuva OS - SystemService - CoreProcessing - Shared Memory Transfer
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

//! Zero-copy shared memory transfer framework for large data passing
//! between services and callers via Nuva IPC shared memory regions.

use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};

use super::error::ServiceError;

/// Shared memory region identifier
pub type ShmRegionId = u64;

/// Shared memory access mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShmAccess {
    /// Read only
    ReadOnly = 0,
    /// Read write
    ReadWrite = 1,
}

/// Shared memory descriptor for zero-copy data transfer
#[derive(Debug, Clone, Copy)]
pub struct ShmDescriptor {
    /// Region ID assigned by the transfer manager
    pub region_id: ShmRegionId,
    /// Size in bytes
    pub size: u64,
    /// Access mode
    pub access: ShmAccess,
    /// Owner PID
    pub owner_pid: u32,
}

/// Shared memory region state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShmRegionState {
    /// Region is free
    Free = 0,
    /// Region is allocated and active
    Active = 1,
    /// Region is pending release
    PendingRelease = 2,
}

/// Maximum concurrent shared memory regions
pub const MAX_SHM_REGIONS: usize = 256;

/// Shared memory region entry
pub struct ShmRegion {
    /// Region ID
    pub id: ShmRegionId,
    /// Size in bytes
    pub size: u64,
    /// Access mode
    pub access: ShmAccess,
    /// Owner PID
    pub owner_pid: u32,
    /// Region state
    pub state: AtomicU32,
}

impl ShmRegion {
    /// Create a free region entry
    pub const fn new() -> Self {
        ShmRegion {
            id: 0,
            size: 0,
            access: ShmAccess::ReadOnly,
            owner_pid: 0,
            state: AtomicU32::new(ShmRegionState::Free as u32),
        }
    }

    /// Get region state
    pub fn get_state(&self) -> ShmRegionState {
        match self.state.load(Ordering::Acquire) {
            0 => ShmRegionState::Free,
            1 => ShmRegionState::Active,
            2 => ShmRegionState::PendingRelease,
            _ => ShmRegionState::Free,
        }
    }
}

/// Shared memory transfer manager - manages zero-copy regions
pub struct ShmTransferManager {
    /// Region pool
    pub regions: [ShmRegion; MAX_SHM_REGIONS],
    /// Next region ID
    pub next_id: AtomicU64,
    /// Total allocated bytes
    pub allocated_bytes: AtomicU64,
    /// Peak allocated bytes
    pub peak_bytes: AtomicU64,
}

impl ShmTransferManager {
    /// Create a new transfer manager
    pub fn new() -> Self {
        // SAFETY: ShmRegion contains AtomicU32 which is zero-initializable.
        // All fields have valid zero representations.
        let regions: [ShmRegion; MAX_SHM_REGIONS] = unsafe {
            core::mem::zeroed()
        };
        ShmTransferManager {
            regions,
            next_id: AtomicU64::new(1),
            allocated_bytes: AtomicU64::new(0),
            peak_bytes: AtomicU64::new(0),
        }
    }

    /// Allocate a shared memory region for zero-copy transfer
    pub fn allocate(
        &mut self,
        size: u64,
        access: ShmAccess,
        owner_pid: u32,
    ) -> Result<ShmDescriptor, ServiceError> {
        let region_id = self.next_id.fetch_add(1, Ordering::AcqRel);

        for i in 0..MAX_SHM_REGIONS {
            let state = self.regions[i].get_state();
            if state == ShmRegionState::Free {
                self.regions[i].id = region_id;
                self.regions[i].size = size;
                self.regions[i].access = access;
                self.regions[i].owner_pid = owner_pid;
                self.regions[i].state.store(
                    ShmRegionState::Active as u32,
                    Ordering::Release,
                );

                let prev = self.allocated_bytes.fetch_add(size, Ordering::AcqRel);
                let peak = self.peak_bytes.load(Ordering::Acquire);
                if prev + size > peak {
                    self.peak_bytes.store(prev + size, Ordering::Release);
                }

                return Ok(ShmDescriptor {
                    region_id,
                    size,
                    access,
                    owner_pid,
                });
            }
        }

        Err(ServiceError::OutOfMemory)
    }

    /// Release a shared memory region
    pub fn release(&mut self, region_id: ShmRegionId) -> Result<(), ServiceError> {
        for i in 0..MAX_SHM_REGIONS {
            if self.regions[i].id == region_id
                && self.regions[i].get_state() == ShmRegionState::Active
            {
                let size = self.regions[i].size;
                self.regions[i].state.store(
                    ShmRegionState::Free as u32,
                    Ordering::Release,
                );
                self.regions[i].id = 0;
                self.allocated_bytes.fetch_sub(size, Ordering::AcqRel);
                return Ok(());
            }
        }
        Err(ServiceError::InvalidArgument)
    }

    /// Get total allocated bytes
    pub fn allocated(&self) -> u64 {
        self.allocated_bytes.load(Ordering::Acquire)
    }
}
