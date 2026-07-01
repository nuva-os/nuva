/*
 * Nuva OS - Kernel - NvProcess - Thread Manager
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
use crate::kernel::types::{NuvaThreadId, NuvaProcessId};
use crate::kernel::error::{KernelError, KernelResult};

const MAX_TIDS: usize = 65536;
const BITMAP_WORDS: usize = MAX_TIDS / 64;
const KERNEL_STACK_SIZE: usize = 8192;
const USER_STACK_SIZE: u64 = 8 * 1024 * 1024;

pub struct NuvaThreadInfo {
    pub tid: NuvaThreadId,
    pub owner_pid: NuvaProcessId,
    pub kernel_stack: u64,
    pub user_stack: u64,
    pub state: AtomicU32,
}

pub struct NuvaThreadManager {
    bitmap: [AtomicU64; BITMAP_WORDS],
    next_scan: AtomicU32,
    active_count: AtomicU32,
}

impl NuvaThreadManager {
    pub const fn new() -> Self {
        const ZERO: AtomicU64 = AtomicU64::new(0);
        NuvaThreadManager {
            bitmap: [ZERO; BITMAP_WORDS],
            next_scan: AtomicU32::new(1),
            active_count: AtomicU32::new(0),
        }
    }

    fn alloc_tid(&self) -> KernelResult<NuvaThreadId> {
        let start = self.next_scan.load(Ordering::Acquire) as usize;
        for i in 0..MAX_TIDS {
            let idx = ((start + i) % MAX_TIDS).max(1);
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
                        return Ok(NuvaThreadId::new(idx as u64));
                    }
                    Err(_) => continue,
                }
            }
        }
        Err(KernelError::DeviceBusy)
    }

    fn free_tid(&self, tid: NuvaThreadId) {
        let idx = tid.as_u64() as usize;
        if idx == 0 || idx >= MAX_TIDS {
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

    pub fn create_thread(
        &self,
        owner_pid: NuvaProcessId,
        entry: u64,
        arg: u64,
    ) -> KernelResult<NuvaThreadInfo> {
        let tid = self.alloc_tid()?;

        let kernel_stack = Self::alloc_kernel_stack();
        if kernel_stack == 0 {
            self.free_tid(tid);
            return Err(KernelError::OutOfMemory);
        }

        let user_stack = Self::alloc_user_stack();
        if user_stack == 0 {
            Self::free_kernel_stack(kernel_stack);
            self.free_tid(tid);
            return Err(KernelError::OutOfMemory);
        }

        Self::setup_initial_context(kernel_stack, entry, arg);

        Ok(NuvaThreadInfo {
            tid,
            owner_pid,
            kernel_stack,
            user_stack,
            state: AtomicU32::new(2),
        })
    }

    pub fn exit_thread(&self, tid: NuvaThreadId, thread: &NuvaThreadInfo) {
        if thread.kernel_stack != 0 {
            Self::free_kernel_stack(thread.kernel_stack);
        }
        if thread.user_stack != 0 {
            Self::free_user_stack(thread.user_stack);
        }
        self.free_tid(tid);
    }

    pub fn find_thread(&self, _tid: NuvaThreadId) -> Option<NuvaThreadId> {
        None
    }

    pub fn detach_thread(&self, _tid: NuvaThreadId) -> KernelResult<()> {
        Ok(())
    }

    fn alloc_kernel_stack() -> u64 {
        0x1000
    }

    fn free_kernel_stack(_stack: u64) {}

    fn alloc_user_stack() -> u64 {
        0x7FFF_FFFF_F000 - USER_STACK_SIZE
    }

    fn free_user_stack(_stack: u64) {}

    fn setup_initial_context(_kernel_stack: u64, _entry: u64, _arg: u64) {}

    pub fn active_count(&self) -> u32 {
        self.active_count.load(Ordering::Acquire)
    }
}

static THREAD_MANAGER: spin::Once<NuvaThreadManager> = spin::Once::new();

pub fn thread_manager() -> &'static NuvaThreadManager {
    THREAD_MANAGER.call_once(NuvaThreadManager::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_free_tid() {
        let mgr = NuvaThreadManager::new();
        let t1 = mgr.alloc_tid().unwrap();
        let t2 = mgr.alloc_tid().unwrap();
        assert_ne!(t1, t2);
        mgr.free_tid(t1);
        mgr.free_tid(t2);
    }

    #[test]
    fn test_create_exit_thread() {
        let mgr = NuvaThreadManager::new();
        let thread = mgr.create_thread(NuvaProcessId::new(1), 0x1000, 0).unwrap();
        assert!(thread.tid.as_u64() >= 1);
        mgr.exit_thread(thread.tid, &thread);
    }
}