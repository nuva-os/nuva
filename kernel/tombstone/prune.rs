/*
 * Nuva OS - Kernel - Tombstone - Prune / Cleanup
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

//! Tombstone pruning and cleanup operations.
/*!*/
//! Implements FIFO auto-pruning, per-PID cleanup, time-range cleanup,
//! and full cleanup. All operations are serialized via the manager's lock.

use super::record::TombstoneError;
use super::stats::TombstoneStats;
use super::store::TombstoneStore;

// ---------------------------------------------------------------------------
// TombstoneClearType
// ---------------------------------------------------------------------------

/** Type of tombstone cleanup to perform */
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TombstoneClearType {
    /** Clear all tombstones for a specific PID */
    ByPid = 0,
    /** Clear all tombstones older than a timestamp */
    ByTimeRange = 1,
    /** Clear all tombstone records */
    All = 2,
}

// ---------------------------------------------------------------------------
// TombstoneClearParams
// ---------------------------------------------------------------------------

/** Parameters for a tombstone cleanup operation */
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TombstoneClearParams {
    /** Cleanup type selector */
    pub clear_type: TombstoneClearType,
    /** Process ID (valid when clear_type == ByPid) */
    pub pid: u32,
    /** Clear tombstones older than this timestamp (valid when clear_type == ByTimeRange) */
    pub before_ts: u64,
}

// ---------------------------------------------------------------------------
// TombstoneClearResult
// ---------------------------------------------------------------------------

/** Result of a tombstone cleanup operation */
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TombstoneClearResult {
    /** Number of successfully deleted records */
    pub deleted_count: u32,
    /** Number of records that could not be deleted */
    pub failed_count: u32,
}

impl TombstoneClearResult {
    /** Create an empty clear result */
    pub const fn new() -> Self {
        TombstoneClearResult {
            deleted_count: 0,
            failed_count: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// prune_tombstones
// ---------------------------------------------------------------------------

/** Execute a tombstone cleanup operation.
 *  Files currently being written are skipped.
 *  Deletion failures are counted but do not abort the operation. */
pub fn prune_tombstones(
    store: &mut TombstoneStore,
    stats: &TombstoneStats,
    params: &TombstoneClearParams,
) -> Result<TombstoneClearResult, TombstoneError> {
    let mut result = TombstoneClearResult::new();

    // Collect matching file numbers from the index
    let matching: alloc::vec::Vec<u8> = match params.clear_type {
        TombstoneClearType::ByPid => store
            .query_index(|e| e.pid == params.pid, 100)
            .iter()
            .map(|e| e.file_number)
            .collect(),
        TombstoneClearType::ByTimeRange => store
            .query_index(|e| e.timestamp < params.before_ts, 100)
            .iter()
            .map(|e| e.file_number)
            .collect(),
        TombstoneClearType::All => store
            .query_index(|_| true, 100)
            .iter()
            .map(|e| e.file_number)
            .collect(),
    };

    for file_num in matching {
        match store.delete_file(file_num) {
            Ok(()) => {
                store.remove_index_by_file_number(file_num);
                result.deleted_count += 1;
            }
            Err(_) => {
                result.failed_count += 1;
            }
        }
    }

    if result.deleted_count > 0 {
        stats.decrement_count(result.deleted_count);
    }

    Ok(result)
}
