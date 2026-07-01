/*
 * Nuva OS - Kernel - Tombstone - System Call Interface
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

//! Tombstone system call interface.
/*!*/
//! Provides four system calls for user-space tombstone access:
//! SYS_TOMBSTONE_QUERY, SYS_TOMBSTONE_READ, SYS_TOMBSTONE_CLEAR,
//! and SYS_TOMBSTONE_STATS.

use super::prune::{TombstoneClearParams, TombstoneClearResult, TombstoneClearType};
use super::query::{
    query_tombstones, read_tombstone_detail, TombstoneQueryParams, TombstoneQueryResult,
};
use super::record::TombstoneError;
use super::stats::TombstoneStatsSnapshot;

// ---------------------------------------------------------------------------
// System call numbers
// ---------------------------------------------------------------------------

/** Query tombstone records */
pub const SYS_TOMBSTONE_QUERY: u64 = 500;

/** Read a single tombstone record detail */
pub const SYS_TOMBSTONE_READ: u64 = 501;

/** Clear (delete) tombstone records */
pub const SYS_TOMBSTONE_CLEAR: u64 = 502;

/** Get tombstone statistics */
pub const SYS_TOMBSTONE_STATS: u64 = 503;

// ---------------------------------------------------------------------------
// Capability checks
// ---------------------------------------------------------------------------

/** Check if the caller has CAP_SYS_PTRACE or CAP_SYS_ADMIN for read access */
fn check_read_capability() -> bool {
    crate::kernel::security::security_manager()
        .check_capability(crate::kernel::security::Capability::Debug)
        == crate::kernel::security::AccessResult::Allow
        || crate::kernel::security::security_manager()
            .check_capability(crate::kernel::security::Capability::SysAdmin)
        == crate::kernel::security::AccessResult::Allow
}

fn check_admin_capability() -> bool {
    crate::kernel::security::security_manager()
        .check_capability(crate::kernel::security::Capability::SysAdmin)
        == crate::kernel::security::AccessResult::Allow
}


// ---------------------------------------------------------------------------
// System call handlers
// ---------------------------------------------------------------------------

/** SYS_TOMBSTONE_QUERY: Query tombstone records.
 *  args[0] = pointer to TombstoneQueryParams
 *  Returns 0 on success, negative errno on failure. */
pub fn sys_tombstone_query(args: &[u64]) -> i64 {
    if !check_read_capability() {
        return -13i64; // EACCES
    }

    if args.is_empty() {
        return -22i64; // EINVAL
    }

    // SAFETY: The user-space pointer is validated by the syscall wrapper.
    // We read TombstoneQueryParams from the provided address.
    let params_ptr = args[0] as *const TombstoneQueryParams;
    if params_ptr.is_null() {
        return -22i64; // EINVAL
    }

    let params = unsafe { core::ptr::read(params_ptr) };
    match params.validate() {
        Ok(()) => {}
        Err(e) => return e.to_errno() as i64,
    }

    let manager = super::TOMBSTONE_MANAGER.lock();
    match query_tombstones(&manager.store, &params) {
        Ok(_result) => 0i64,
        Err(e) => e.to_errno() as i64,
    }
}

/** SYS_TOMBSTONE_READ: Read a single tombstone record.
 *  args[0] = file number (0-99)
 *  Returns 0 on success, negative errno on failure. */
pub fn sys_tombstone_read(args: &[u64]) -> i64 {
    if !check_read_capability() {
        return -13i64; // EACCES
    }

    if args.is_empty() {
        return -22i64; // EINVAL
    }

    let file_number = args[0] as u8;
    if file_number >= 100 {
        return -22i64; // EINVAL
    }

    let manager = super::TOMBSTONE_MANAGER.lock();
    match read_tombstone_detail(&manager.store, file_number) {
        Ok(_record) => 0i64,
        Err(e) => e.to_errno() as i64,
    }
}

/** SYS_TOMBSTONE_CLEAR: Clear tombstone records.
 *  args[0] = pointer to TombstoneClearParams
 *  Returns 0 on success, negative errno on failure. */
pub fn sys_tombstone_clear(args: &[u64]) -> i64 {
    if !check_admin_capability() {
        return -1i64; // EPERM
    }

    if args.is_empty() {
        return -22i64; // EINVAL
    }

    // SAFETY: The user-space pointer is validated by the syscall wrapper.
    let params_ptr = args[0] as *const TombstoneClearParams;
    if params_ptr.is_null() {
        return -22i64; // EINVAL
    }

    let params = unsafe { core::ptr::read(params_ptr) };
    let mut manager = super::TOMBSTONE_MANAGER.lock();
    match super::prune::prune_tombstones(&mut manager.store, &manager.stats, &params) {
        Ok(_result) => 0i64,
        Err(e) => e.to_errno() as i64,
    }
}

/** SYS_TOMBSTONE_STATS: Get tombstone statistics.
 *  Returns 0 on success, negative errno on failure. */
pub fn sys_tombstone_stats(_args: &[u64]) -> i64 {
    if !check_read_capability() {
        return -13i64; // EACCES
    }

    let manager = super::TOMBSTONE_MANAGER.lock();
    let _snapshot: TombstoneStatsSnapshot = manager.stats.snapshot();
    0i64
}
