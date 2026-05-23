/*
 * Nuva OS - System Library - Lang Runtime GC
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

/// GC algorithm
#[derive(Debug, Clone, Copy)]
pub enum GcAlgorithm {
    /// Mark-sweep
    MarkSweep = 0,
    /// Copying collector
    Copying = 1,
    /// Mark-compact
    MarkCompact = 2,
    /// Generational collector
    Generational = 3,
}

/// GC state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GcState {
    /// Idle
    Idle = 0,
    /// Marking in progress
    Marking = 1,
    /// Sweeping in progress
    Sweeping = 2,
    /// Compacting in progress
    Compacting = 3,
}

/// GC object header
#[repr(C)]
pub struct GcObjectHeader {
    /// Object size
    pub size: u32,
    /// Mark bit
    pub marked: AtomicU32,
    /// Type info
    pub type_info: u64,
}

/// GC statistics
pub struct GcStats {
    /// GC cycle count
    pub gc_count: AtomicU64,
    /// Total GC time (microseconds)
    pub total_time: AtomicU64,
    /// Reclaimed byte count
    pub reclaimed_bytes: AtomicU64,
    /// Live object count
    pub live_objects: AtomicU64,
}

/// Garbage collector
pub struct GarbageCollector {
    /// GC algorithm
    pub algorithm: GcAlgorithm,
    /// State
    pub state: AtomicU32,
    /// Statistics
    pub stats: GcStats,
    /// Heap start address
    pub heap_start: u64,
    /// Heap size
    pub heap_size: usize,
}

impl GarbageCollector {
    pub const fn new(heap_start: u64, heap_size: usize) -> Self {
        GarbageCollector {
            algorithm: GcAlgorithm::MarkSweep,
            state: AtomicU32::new(GcState::Idle as u32),
            stats: GcStats {
                gc_count: AtomicU64::new(0),
                total_time: AtomicU64::new(0),
                reclaimed_bytes: AtomicU64::new(0),
                live_objects: AtomicU64::new(0),
            },
            heap_start,
            heap_size,
        }
    }

    /// Initialize the garbage collector
    pub fn init(&mut self) {
        log_info!("Garbage collector initialized");
        log_info!("  Algorithm: {:?}", self.algorithm);
        log_info!("  Heap: {:#x} - {:#x}", self.heap_start, self.heap_start + self.heap_size as u64);
    }

    /// Execute a garbage collection cycle
    pub fn collect(&mut self) -> i32 {
        if self.state.load(Ordering::Acquire) != GcState::Idle as u32 {
            return -1;
        }

        log_debug!("GC started");

        // Mark phase
        self.state.store(GcState::Marking as u32, Ordering::Release);
        self.mark_phase();

        // Sweep phase
        self.state.store(GcState::Sweeping as u32, Ordering::Release);
        self.sweep_phase();

        // Update statistics
        self.stats.gc_count.fetch_add(1, Ordering::AcqRel);

        // Return to idle state
        self.state.store(GcState::Idle as u32, Ordering::Release);

        log_debug!("GC completed");
        0
    }

    /// Mark phase: trace reachable objects from roots
    fn mark_phase(&mut self) {
        let heap_end = self.heap_start + self.heap_size as u64;
        let header_size = core::mem::size_of::<GcObjectHeader>() as u64;

        // Mark the root object (first object on the heap)
        if self.heap_start + header_size <= heap_end {
            // SAFETY: heap_start is within valid heap range
            let root_header = unsafe { &*(self.heap_start as *const GcObjectHeader) };
            if root_header.size > 0 {
                root_header.marked.store(1, Ordering::Release);
            }
        }

        // Iteratively trace references from marked objects until fixed point
        let mut changed = true;
        while changed {
            changed = false;
            let mut addr = self.heap_start;

            while addr + header_size <= heap_end {
                // SAFETY: addr is within heap bounds [heap_start, heap_end)
                let header = unsafe { &*(addr as *const GcObjectHeader) };

                if header.marked.load(Ordering::Acquire) != 0 && header.size > 0 {
                    // Scan object data for pointers to other heap objects
                    let obj_start = addr + header_size;
                    let obj_end = obj_start + header.size as u64;
                    let mut scan = obj_start;

                    while scan + 8 <= obj_end && scan + 8 <= heap_end {
                        // SAFETY: scan is within object data bounds
                        let ptr_val = unsafe { *(scan as *const u64) };
                        if ptr_val >= self.heap_start && ptr_val < heap_end {
                            // SAFETY: ptr_val points within the heap range
                            let ref_header = unsafe { &*(ptr_val as *const GcObjectHeader) };
                            if ref_header.size > 0
                                && ref_header.marked.load(Ordering::Acquire) == 0
                            {
                                ref_header.marked.store(1, Ordering::Release);
                                changed = true;
                            }
                        }
                        scan += 8;
                    }
                }

                let next = addr + header_size + header.size as u64;
                if next <= addr || next > heap_end {
                    break;
                }
                addr = next;
            }
        }
    }

    /// Sweep phase: reclaim unreachable objects and clear mark bits
    fn sweep_phase(&mut self) {
        let heap_end = self.heap_start + self.heap_size as u64;
        let header_size = core::mem::size_of::<GcObjectHeader>() as u64;
        let mut addr = self.heap_start;

        while addr + header_size <= heap_end {
            // SAFETY: addr is within heap bounds
            let header = unsafe { &*(addr as *const GcObjectHeader) };

            if header.size == 0 {
                break;
            }

            if header.marked.load(Ordering::Acquire) == 0 {
                // Object is unreachable, reclaim it
                self.stats.reclaimed_bytes.fetch_add(header.size as u64, Ordering::AcqRel);
                self.stats.live_objects.fetch_sub(1, Ordering::AcqRel);
            }

            // Clear mark bit for next GC cycle
            header.marked.store(0, Ordering::Release);

            let next = addr + header_size + header.size as u64;
            if next <= addr || next > heap_end {
                break;
            }
            addr = next;
        }
    }

    /// Allocate an object on the managed heap
    pub fn alloc(&mut self, size: usize) -> Option<u64> {
        // Check if GC is needed
        if self.should_collect() {
            self.collect();
        }

        // Allocate object on the managed heap
        // 1. Add header size to the requested size
        let total_size = core::mem::size_of::<GcObjectHeader>() + size;

        // 2. Find free space in the heap (first-fit or best-fit)
        // In a real implementation, this would search the free list
        // and split blocks if necessary
        if total_size > self.heap_size {
            return None; // Out of heap space
        }

        // 3. Initialize the object header
        let addr = self.heap_start; // Simplified: always allocate from start
        self.stats.live_objects.fetch_add(1, Ordering::AcqRel);

        Some(addr)
    }

    /// Check if garbage collection should be triggered based on heuristics
    fn should_collect(&self) -> bool {
        let live = self.stats.live_objects.load(Ordering::Acquire);
        let gc_count = self.stats.gc_count.load(Ordering::Acquire);

        // Trigger when live objects exceed a threshold (4096 objects)
        if live > 4096 {
            return true;
        }

        // Trigger when GC has not run after significant allocation activity
        if live > 512 && gc_count == 0 {
            return true;
        }

        false
    }

    /// Get the GC state
    pub fn get_state(&self) -> GcState {
        match self.state.load(Ordering::Acquire) {
            0 => GcState::Idle,
            1 => GcState::Marking,
            2 => GcState::Sweeping,
            3 => GcState::Compacting,
            _ => GcState::Idle,
        }
    }

    /// Get statistics
    pub fn get_stats(&self) -> (u64, u64, u64) {
        let gc_count = self.stats.gc_count.load(Ordering::Acquire);
        let total_time = self.stats.total_time.load(Ordering::Acquire);
        let reclaimed = self.stats.reclaimed_bytes.load(Ordering::Acquire);
        (gc_count, total_time, reclaimed)
    }

    /// Set the GC algorithm
    pub fn set_algorithm(&mut self, algorithm: GcAlgorithm) {
        self.algorithm = algorithm;
        log_info!("GC algorithm set to: {:?}", algorithm);
    }
}

/// Global garbage collector instance
static mut GARBAGE_COLLECTOR: GarbageCollector = GarbageCollector::new(0, 16 * 1024 * 1024);

/// Get the global garbage collector instance
pub fn get_gc() -> &'static mut GarbageCollector {
    // SAFETY: Single-threaded access; synchronized externally.
    unsafe { &mut GARBAGE_COLLECTOR }
}

/// Initialize the garbage collector
pub fn init_gc() {
    let gc = get_gc();
    gc.init();
}
