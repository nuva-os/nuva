/*
 * Nuva OS - SystemLibrary - Brain
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


use core::sync::atomic::{AtomicU64, Ordering};

/// Memorypool
pub struct MemoryPool {
    /// baseAddress
    pub base_addr: u64,
    /// totalSize
    pub total_size: usize,
    /// alreadyuseSize
    pub used_size: AtomicU64,
    /// peakvaluemakeuse
    pub peak_usage: AtomicU64,
}

impl MemoryPool {
    pub const fn new(base_addr: u64, total_size: usize) -> Self {
        MemoryPool {
            base_addr,
            total_size,
            used_size: AtomicU64::new(0),
            peak_usage: AtomicU64::new(0),
        }
    }
    
    /// AllocateMemory
    pub fn alloc(&self, size: usize, alignment: usize) -> Option<u64> {
        // AlignmentSize
        let aligned_size = (size + alignment - 1) & !(alignment - 1);
        
        // Checkremainingremainderemptybetween
        let used = self.used_size.load(Ordering::Acquire) as usize;
        if used + aligned_size > self.total_size {
            return None;
        }
        
        // Allocate
        let addr = self.base_addr + used as u64;
        self.used_size.fetch_add(aligned_size as u64, Ordering::AcqRel);
        
        // Updatepeakvalue
        let new_used = self.used_size.load(Ordering::Acquire);
        let mut peak = self.peak_usage.load(Ordering::Acquire);
        while new_used > peak {
            match self.peak_usage.compare_exchange_weak(
                peak,
                new_used,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(current) => peak = current,
            }
        }
        
        Some(addr)
    }
    
    /// FreeMemory (simpleformImplementation,notSupportpartsplitFree)
    pub fn reset(&self) {
        self.used_size.store(0, Ordering::Release);
    }
    
    /// Getremainingremainderemptybetween
    pub fn available(&self) -> usize {
        let used = self.used_size.load(Ordering::Acquire) as usize;
        self.total_size - used
    }
    
    /// Getmakeuserate
    pub fn usage_ratio(&self) -> u32 {
        let used = self.used_size.load(Ordering::Acquire);
        (used * 100 / self.total_size as u64) as u32
    }
}

/// NPU MemoryManager
pub struct NpuMemoryManager {
    /// InputMemorypool
    pub input_pool: MemoryPool,
    /// OutputMemorypool
    pub output_pool: MemoryPool,
    /// WeightMemorypool
    pub weight_pool: MemoryPool,
    /// infixbetweenresultMemorypool
    pub workspace_pool: MemoryPool,
}

impl NpuMemoryManager {
    pub const fn new() -> Self {
        NpuMemoryManager {
            input_pool: MemoryPool::new(0, 16 * 1024 * 1024),      // 16 MB
            output_pool: MemoryPool::new(0, 16 * 1024 * 1024),     // 16 MB
            weight_pool: MemoryPool::new(0, 256 * 1024 * 1024),    // 256 MB
            workspace_pool: MemoryPool::new(0, 64 * 1024 * 1024),  // 64 MB
        }
    }
    
    /// Initialize
    pub fn init(&mut self) -> i32 {
        log_info!("NPU memory manager initialized");
        log_info!("  Input pool: 16 MB");
        log_info!("  Output pool: 16 MB");
        log_info!("  Weight pool: 256 MB");
        log_info!("  Workspace pool: 64 MB");
        0
    }
    
    /// AllocateInputBuffer
    pub fn alloc_input(&self, size: usize) -> Option<u64> {
        self.input_pool.alloc(size, 64)  // 64 ByteAlignment
    }
    
    /// AllocateOutputBuffer
    pub fn alloc_output(&self, size: usize) -> Option<u64> {
        self.output_pool.alloc(size, 64)
    }
    
    /// AllocateWeightBuffer
    pub fn alloc_weight(&self, size: usize) -> Option<u64> {
        self.weight_pool.alloc(size, 4096)  // 4KB Alignment
    }
    
    /// Allocateworkmakeemptybetween
    pub fn alloc_workspace(&self, size: usize) -> Option<u64> {
        self.workspace_pool.alloc(size, 64)
    }
    
    /// ResetplacefiniteMemorypool
    pub fn reset_all(&self) {
        self.input_pool.reset();
        self.output_pool.reset();
        self.workspace_pool.reset();
        // WeightnotReset
    }
    
    /// GettotalMemorymakeuse
    pub fn get_total_usage(&self) -> u64 {
        self.input_pool.used_size.load(Ordering::Acquire)
            + self.output_pool.used_size.load(Ordering::Acquire)
            + self.weight_pool.used_size.load(Ordering::Acquire)
            + self.workspace_pool.used_size.load(Ordering::Acquire)
    }
}

/// Global NPU MemoryManager
static NPU_MEMORY_MANAGER: crate::sync_oncelock::OnceLock<NpuMemoryManager> = crate::sync_oncelock::OnceLock::new();

pub fn get_npu_memory_manager() -> &'static mut NpuMemoryManager {
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { &mut NPU_MEMORY_MANAGER }
}

pub fn init_npu_memory() {
    let manager = get_npu_memory_manager();
    manager.init();
}