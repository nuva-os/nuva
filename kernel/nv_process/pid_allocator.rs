/*
 * Nuva OS - Kernel - NvProcess - PID Allocator
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

use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use crate::kernel::types::NuvaProcessId;
use crate::kernel::error::{KernelError, KernelResult};

const MAX_PIDS: usize = 65536;
const BITMAP_WORDS: usize = MAX_PIDS / 64;

pub struct NuvaPidAllocator {
    bitmap: [AtomicU64; BITMAP_WORDS],
    next_scan: AtomicU32,
    active_count: AtomicU32,
    thread_count: AtomicU32,
}

impl NuvaPidAllocator {
    pub const fn new() -> Self {
        const ZERO: AtomicU64 = AtomicU64::new(0);
        NuvaPidAllocator {
            bitmap: [ZERO; BITMAP_WORDS],
            next_scan: AtomicU32::new(1),
            active_count: AtomicU32::new(0),
            thread_count: AtomicU32::new(0),
        }
    }

    pub fn alloc_pid(&self) -> KernelResult<NuvaProcessId> {
        let start = self.next_scan.load(Ordering::Acquire) as usize;
        for i in 0..MAX_PIDS {
            let idx = ((start + i) % MAX_PIDS).max(1);
            let word = idx / 64;
            let bit = idx % 64;
            let mask = 1u64 << bit;
            let old = self.bitmap[word].load(Ordering::Acquire);
            if (old & mask) == 0 {
                match self.bitmap[word].compare_exchange_weak(
                    old,
                    old | mask,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => {
                        self.next_scan.store((idx + 1) as u32, Ordering::Release);
                        self.active_count.fetch_add(1, Ordering::AcqRel);
                        return Ok(NuvaProcessId::new(idx as u64));
                    }
                    Err(_) => continue,
                }
            }
        }
        Err(KernelError::DeviceBusy)
    }

    pub fn free_pid(&self, pid: NuvaProcessId) {
        let idx = pid.as_u64() as usize;
        if idx == 0 || idx >= MAX_PIDS {
            return;
        }
        let word = idx / 64;
        let bit = idx % 64;
        let mask = 1u64 << bit;
        self.bitmap[word].fetch_and(!mask, Ordering::AcqRel);
        let scan = self.next_scan.load(Ordering::Acquire) as usize;
        if idx < scan {
            self.next_scan.store(idx as u32, Ordering::Release);
        }
        self.active_count.fetch_sub(1, Ordering::AcqRel);
    }

    pub fn active_count(&self) -> u32 {
        self.active_count.load(Ordering::Acquire)
    }

    pub fn thread_count(&self) -> u32 {
        self.thread_count.load(Ordering::Acquire)
    }

    pub fn inc_thread_count(&self) {
        self.thread_count.fetch_add(1, Ordering::AcqRel);
    }

    pub fn dec_thread_count(&self) {
        self.thread_count.fetch_sub(1, Ordering::AcqRel);
    }
}

static PID_ALLOCATOR: spin::Once<NuvaPidAllocator> = spin::Once::new();

pub fn pid_allocator() -> &'static NuvaPidAllocator {
    PID_ALLOCATOR.call_once(NuvaPidAllocator::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_free_pid() {
        let alloc = NuvaPidAllocator::new();
        let pid = alloc.alloc_pid().unwrap();
        assert!(pid.as_u64() >= 1);
        alloc.free_pid(pid);
    }

    #[test]
    fn test_alloc_multiple_pids() {
        let alloc = NuvaPidAllocator::new();
        let p1 = alloc.alloc_pid().unwrap();
        let p2 = alloc.alloc_pid().unwrap();
        assert_ne!(p1, p2);
        assert_eq!(alloc.active_count(), 2);
        alloc.free_pid(p1);
        alloc.free_pid(p2);
        assert_eq!(alloc.active_count(), 0);
    }

    #[test]
    fn test_pid_reuse_after_free() {
        let alloc = NuvaPidAllocator::new();
        let p1 = alloc.alloc_pid().unwrap();
        alloc.free_pid(p1);
        let p2 = alloc.alloc_pid().unwrap();
        assert_eq!(p1, p2);
        alloc.free_pid(p2);
    }
}