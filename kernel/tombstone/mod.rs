/*
 * Nuva OS - Kernel - Tombstone - Module Entry
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

//! Tombstone mechanism — crash record capture, storage, and query.
/*!*/
//! When a process or task crashes, the kernel generates a tombstone
//! record containing CPU context, stack backtrace, and metadata.
//! Records are persisted to the filesystem with atomic writes and
//! an in-memory ring buffer fallback for degraded operation.

// Re-export print macros from crate root
pub use crate::{pr_alert, pr_crit, pr_debug, pr_emerg, pr_err, pr_info, pr_notice, pr_warn};

pub mod arch_adapter;
pub mod config;
pub mod crash_context;
pub mod prune;
pub mod query;
pub mod record;
pub mod stats;
pub mod store;
pub mod syscall;

use config::TombstoneStoreConfig;
use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use crash_context::{collect_crash_context, mask_sensitive_registers, CrashContext, CrashSource};
use record::{
    ArchId, CrashReason, TombstoneError, TombstoneRecord, PROCESS_NAME_MAX_LEN,
    TOMBSTONE_FORMAT_VERSION,
};
use stats::TombstoneStats;
use store::TombstoneStore;

// ---------------------------------------------------------------------------
// DedupAction
// ---------------------------------------------------------------------------

/** Action to take for a crash event after deduplication check */
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DedupAction {
    /** Create a new tombstone record */
    NewRecord,
    /** Merge with the last record (increment crash_count) */
    MergeCount,
}

// ---------------------------------------------------------------------------
// TombstoneManager
// ---------------------------------------------------------------------------

/** Global tombstone subsystem manager.
 *  Coordinates crash context collection, deduplication,
 *  storage, statistics, and query/prune operations. */
pub struct TombstoneManager {
    /** Storage configuration */
    pub config: TombstoneStoreConfig,
    /** Persistent store */
    pub store: TombstoneStore,
    /** Statistics counters */
    pub stats: TombstoneStats,
    /** Whether the subsystem has been initialized */
    initialized: AtomicBool,
    /** Whether operating in degraded (memory-only) mode */
    degraded: AtomicBool,
    /** Last crash PID for deduplication */
    last_crash_pid: AtomicU32,
    /** Last crash timestamp for deduplication */
    last_crash_ts: AtomicU64,
    /** Crash count within the current dedup window */
    last_crash_count: AtomicU32,
}

impl TombstoneManager {
    /** Create an uninitialized TombstoneManager */
    pub fn new() -> Self {
        let config = TombstoneStoreConfig::default();
        TombstoneManager {
            config,
            store: TombstoneStore::new(TombstoneStoreConfig::default()),
            stats: TombstoneStats::new(),
            initialized: AtomicBool::new(false),
            degraded: AtomicBool::new(false),
            last_crash_pid: AtomicU32::new(0),
            last_crash_ts: AtomicU64::new(0),
            last_crash_count: AtomicU32::new(0),
        }
    }
}

// ---------------------------------------------------------------------------
// Global instance
// ---------------------------------------------------------------------------

/** Global TombstoneManager protected by a SpinLock */
static TOMBSTONE_SPINLOCK: crate::kernel::sync::spinlock::SpinLock =
    crate::kernel::sync::spinlock::SpinLock::new();

/** Wrapper that provides lock()/unlock() around TombstoneManager */
pub struct TombstoneManagerLock;

impl TombstoneManagerLock {
    /** Acquire the global lock and return a mutable reference */
    pub fn lock(&self) -> TombstoneManagerGuard {
        TOMBSTONE_SPINLOCK.lock();
        TombstoneManagerGuard { _inner: () }
    }
}

/** RAII guard for TombstoneManager access */
pub struct TombstoneManagerGuard {
    _inner: (),
}

impl TombstoneManagerGuard {
    /** Access the TombstoneManager */
    fn manager(&self) -> &TombstoneManager {
        // SAFETY: The SpinLock is held, so exclusive access is guaranteed.
        unsafe { &*TOMBSTONE_MANAGER_PTR }
    }

    /** Access the TombstoneManager mutably */
    fn manager_mut(&self) -> &mut TombstoneManager {
        // SAFETY: The SpinLock is held, so exclusive access is guaranteed.
        unsafe { &mut *TOMBSTONE_MANAGER_PTR }
    }
}

impl Drop for TombstoneManagerGuard {
    fn drop(&mut self) {
        TOMBSTONE_SPINLOCK.unlock();
    }
}

/** Static TombstoneManager instance */
static mut TOMBSTONE_MANAGER_PTR: *mut TombstoneManager = 0 as *mut TombstoneManager;
static mut TOMBSTONE_MANAGER_MEM: core::mem::MaybeUninit<TombstoneManager> =
    core::mem::MaybeUninit::uninit();

/** Global accessor for the TombstoneManager lock */
pub static TOMBSTONE_MANAGER: TombstoneManagerLock = TombstoneManagerLock;

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

/** Initialize the tombstone subsystem.
 *  Called during kernel init_subsystems() after the filesystem is ready.
 *  Initialization failure does not block kernel boot — the subsystem
 *  enters degraded mode instead. */
pub fn init_tombstone() {
    log_info!("Initializing tombstone subsystem...");

    // SAFETY: This is called once during kernel init before any concurrent access.
    unsafe {
        TOMBSTONE_MANAGER_MEM = core::mem::MaybeUninit::new(TombstoneManager::new());
        TOMBSTONE_MANAGER_PTR = TOMBSTONE_MANAGER_MEM.as_mut_ptr();
    }

    let guard = TOMBSTONE_MANAGER.lock();
    let mgr = guard.manager_mut();

    // Validate configuration
    if let Err(e) = mgr.config.validate() {
        log_err!("Tombstone config validation failed: {}", e);
        mgr.degraded.store(true, Ordering::Relaxed);
        return;
    }

    // Re-initialize store with the validated config
    mgr.store = TombstoneStore::new(mgr.config);

    // Rebuild index from existing files
    mgr.store.rebuild_index();

    // Check if we're in degraded mode
    if mgr.store.is_degraded() {
        mgr.degraded.store(true, Ordering::Relaxed);
        log_warn!("Tombstone subsystem initialized in degraded mode (FS unavailable)");
    } else {
        log_info!("Tombstone subsystem initialized successfully");
    }

    mgr.initialized.store(true, Ordering::Relaxed);
}

// ---------------------------------------------------------------------------
// Crash event callbacks
// ---------------------------------------------------------------------------

/** Task crash event callback — called by kernel::sched when a task
 *  transitions to Zombie/Dead due to an abnormal cause. */
pub fn on_task_crash(pid: u32, tid: u32, reason: CrashReason) {
    generate_tombstone(pid, tid, 0, reason, CrashSource::TaskCrash);
}

/** Fatal signal delivery callback — called by kernel::signal before
 *  terminating the process with a fatal signal. */
pub fn on_fatal_signal(pid: u32, tid: u32, signal: u8, reason: CrashReason) {
    generate_tombstone(pid, tid, signal, reason, CrashSource::FatalSignal);
}

/** Watchdog timeout callback — called by kernel::sched when a
 *  task's watchdog expires. */
pub fn on_watchdog_timeout(pid: u32, tid: u32) {
    generate_tombstone(pid, tid, 0, CrashReason::Watchdog, CrashSource::Watchdog);
}

// ---------------------------------------------------------------------------
// Deduplication
// ---------------------------------------------------------------------------

/** Deduplication window: 5 seconds in nanoseconds */
const DEDUP_WINDOW_NS: u64 = 5_000_000_000;

/** Check whether this crash should be deduplicated with a recent one.
 *  Same PID within 5 seconds → MergeCount; otherwise → NewRecord. */
fn check_dedup(pid: u32, timestamp: u64, mgr: &TombstoneManager) -> DedupAction {
    let last_pid = mgr.last_crash_pid.load(Ordering::Relaxed);
    let last_ts = mgr.last_crash_ts.load(Ordering::Relaxed);

    if pid == last_pid && timestamp > last_ts && timestamp - last_ts < DEDUP_WINDOW_NS {
        DedupAction::MergeCount
    } else {
        DedupAction::NewRecord
    }
}

// ---------------------------------------------------------------------------
// generate_tombstone (internal)
// ---------------------------------------------------------------------------

/** Main tombstone generation flow:
 *  1. Check initialization
 *  2. Get timestamp
 *  3. Deduplication check
 *  4. Collect crash context from HAL
 *  5. Mask sensitive registers
 *  6. Assemble TombstoneRecord
 *  7. Write to store
 *  8. Update statistics
 *  Any failure logs an error but never panics. */
fn generate_tombstone(pid: u32, tid: u32, signal: u8, reason: CrashReason, source: CrashSource) {
    let guard = TOMBSTONE_MANAGER.lock();
    let mgr = guard.manager_mut();

    // Check initialization
    if !mgr.initialized.load(Ordering::Relaxed) {
        return;
    }

    // Get current timestamp
    let timestamp = crate::kernel::arch::current_arch().timer().now();

    // Deduplication check
    let dedup = check_dedup(pid, timestamp, mgr);
    match dedup {
        DedupAction::MergeCount => {
            mgr.last_crash_count.fetch_add(1, Ordering::Relaxed);
            mgr.stats.update_last_crash(pid, timestamp);
            return;
        }
        DedupAction::NewRecord => {}
    }

    // Flush previous dedup count if any
    let prev_count = mgr.last_crash_count.swap(1, Ordering::Relaxed);
    mgr.last_crash_pid.store(pid, Ordering::Relaxed);
    mgr.last_crash_ts.store(timestamp, Ordering::Relaxed);

    // Collect crash context
    let crash_ctx: CrashContext = match collect_crash_context(
        source,
        pid,
        tid,
        if signal != 0 { Some(signal) } else { None },
    ) {
        Ok(ctx) => ctx,
        Err(e) => {
            log_err!("Crash context collection failed: {}", e);
            mgr.stats.increment_failure();
            CrashContext::minimal()
        }
    };

    // Mask sensitive registers
    let mut registers = crash_ctx.registers;
    mask_sensitive_registers(&mut registers, crash_ctx.arch_id);

    // Get process name (best effort)
    let mut process_name = [0u8; PROCESS_NAME_MAX_LEN];
    if let Some(task) = unsafe { crate::kernel::sched::task_by_pid(pid).as_ref() } {
        let name = task.name();
        let len = name.len().min(PROCESS_NAME_MAX_LEN);
        process_name[..len].copy_from_slice(&name.as_bytes()[..len]);
    }

    // Assemble TombstoneRecord
    let mut record = TombstoneRecord::new();
    record.version = TOMBSTONE_FORMAT_VERSION;
    record.timestamp = timestamp;
    record.pid = pid;
    record.tid = tid;
    record.process_name = process_name;
    record.crash_reason = reason;
    record.signal_number = signal;
    record.arch_id = crash_ctx.arch_id;
    record.registers = registers;
    record.sp = crash_ctx.sp;
    record.pc = crash_ctx.pc;
    record.fault_addr = crash_ctx.fault_addr;
    record.esr = crash_ctx.esr;
    record.pstate = crash_ctx.pstate;
    record.stack_frames = crash_ctx.stack_frames;
    record.truncated = crash_ctx.stack_frames.count > 0
        && crash_ctx.stack_frames.truncate_reason
            != crate::kernel::tombstone::record::UnwindTruncateReason::None;
    record.context_incomplete = crash_ctx.context_incomplete;
    record.crash_count = prev_count;
    record.checksum = record.compute_checksum();

    // Write to store
    match mgr.store.write(&record, &mgr.stats) {
        Ok(()) => {
            log_info!(
                "Tombstone generated: pid={} tid={} reason={:?} arch={:?}",
                pid,
                tid,
                reason,
                crash_ctx.arch_id
            );
        }
        Err(e) => {
            log_err!("Tombstone write failed: {}", e);
            mgr.stats.increment_failure();
        }
    }

    // Update statistics
    mgr.stats.update_last_crash(pid, timestamp);
}
