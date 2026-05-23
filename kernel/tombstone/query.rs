/*
 * Nuva OS - Kernel - Tombstone - Query Engine
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

//! Tombstone query engine.
/*!*/
//! Supports multi-dimensional queries: by PID, time range, crash reason,
//! latest N records, and single record by file number.

use super::record::{CrashReason, TombstoneError, TombstoneRecord};
use super::store::{TombstoneIndexEntry, TombstoneStore};

// ---------------------------------------------------------------------------
// TombstoneQueryType
// ---------------------------------------------------------------------------

/** Type of tombstone query to perform */
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TombstoneQueryType {
    /** Query by process ID */
    ByPid = 0,
    /** Query by time range [start_ts, end_ts) */
    ByTimeRange = 1,
    /** Query by crash reason */
    ByCrashReason = 2,
    /** Query the latest N records */
    LatestN = 3,
    /** Query a single record by file number */
    ByFileNumber = 4,
}

// ---------------------------------------------------------------------------
// TombstoneQueryParams
// ---------------------------------------------------------------------------

/** Parameters for a tombstone query */
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TombstoneQueryParams {
    /** Query type selector */
    pub query_type: TombstoneQueryType,
    /** Process ID filter (valid when query_type == ByPid) */
    pub pid: u32,
    /** Start timestamp for time range query (nanoseconds) */
    pub start_ts: u64,
    /** End timestamp for time range query (nanoseconds) */
    pub end_ts: u64,
    /** Crash reason filter (valid when query_type == ByCrashReason) */
    pub crash_reason: CrashReason,
    /** Maximum number of records to return (valid when query_type == LatestN) */
    pub limit: u32,
    /** File number (valid when query_type == ByFileNumber) */
    pub file_number: u8,
}

impl TombstoneQueryParams {
    /** Validate query parameters */
    pub fn validate(&self) -> Result<(), TombstoneError> {
        match self.query_type {
            TombstoneQueryType::ByPid => {
                if self.pid == 0 {
                    return Err(TombstoneError::InvalidParam);
                }
            }
            TombstoneQueryType::ByTimeRange => {
                if self.start_ts >= self.end_ts {
                    return Err(TombstoneError::InvalidParam);
                }
            }
            TombstoneQueryType::LatestN => {
                if self.limit == 0 {
                    return Err(TombstoneError::InvalidParam);
                }
            }
            TombstoneQueryType::ByFileNumber => {
                if self.file_number >= 100 {
                    return Err(TombstoneError::InvalidParam);
                }
            }
            TombstoneQueryType::ByCrashReason => {}
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// TombstoneQueryResult
// ---------------------------------------------------------------------------

/** Maximum number of results returned in a single query */
pub const MAX_QUERY_RESULTS: usize = 32;

/** Result of a tombstone query */
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TombstoneQueryResult {
    /** Matching index entries */
    pub entries: [Option<TombstoneIndexEntry>; MAX_QUERY_RESULTS],
    /** Number of valid entries in the result */
    pub count: u32,
    /** Total number of matching records (may exceed count if has_more) */
    pub total_matched: u32,
    /** Whether there are more matching records beyond the returned set */
    pub has_more: bool,
}

impl TombstoneQueryResult {
    /** Create an empty query result */
    pub fn new() -> Self {
        TombstoneQueryResult {
            entries: [None; MAX_QUERY_RESULTS],
            count: 0,
            total_matched: 0,
            has_more: false,
        }
    }
}

// ---------------------------------------------------------------------------
// query_tombstones
// ---------------------------------------------------------------------------

/** Execute a tombstone query against the in-memory index.
 *  Only reads from the index for speed; use read_tombstone_detail()
 *  to load the full record. */
pub fn query_tombstones(
    store: &TombstoneStore,
    params: &TombstoneQueryParams,
) -> Result<TombstoneQueryResult, TombstoneError> {
    params.validate()?;

    let limit = match params.query_type {
        TombstoneQueryType::LatestN => params.limit.min(MAX_QUERY_RESULTS as u32),
        _ => MAX_QUERY_RESULTS as u32,
    };

    let mut result = TombstoneQueryResult::new();
    let mut all_entries: alloc::vec::Vec<TombstoneIndexEntry> = alloc::vec::Vec::new();

    match params.query_type {
        TombstoneQueryType::ByPid => {
            all_entries = store.query_index(|e| e.pid == params.pid, limit);
        }
        TombstoneQueryType::ByTimeRange => {
            all_entries = store.query_index(
                |e| e.timestamp >= params.start_ts && e.timestamp < params.end_ts,
                limit,
            );
        }
        TombstoneQueryType::ByCrashReason => {
            all_entries = store.query_index(|e| e.crash_reason == params.crash_reason, limit);
        }
        TombstoneQueryType::LatestN => {
            all_entries = store.query_index(|_| true, limit);
            // Sort by timestamp descending
            all_entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        }
        TombstoneQueryType::ByFileNumber => {
            all_entries = store.query_index(|e| e.file_number == params.file_number, 1);
        }
    }

    result.total_matched = all_entries.len() as u32;
    let copy_len = all_entries.len().min(MAX_QUERY_RESULTS);
    for i in 0..copy_len {
        result.entries[i] = Some(all_entries[i]);
    }
    result.count = copy_len as u32;
    result.has_more = all_entries.len() > MAX_QUERY_RESULTS;

    Ok(result)
}

/** Read the full detail of a tombstone record from storage */
pub fn read_tombstone_detail(
    store: &TombstoneStore,
    file_number: u8,
) -> Result<TombstoneRecord, TombstoneError> {
    store.read_record(file_number)
}
