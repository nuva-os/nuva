/*
 * Nuva OS - Nuva OS
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

//! NuvaFS File System
//! Nuva OS native file system, supports journal, snapshot, compression and other features

pub mod superblock;
pub mod inode;
pub mod dir;
pub mod journal;
pub mod file;
pub mod snapshot;
pub mod posix;
pub mod tests;

// WAL (Write-Ahead Log) modules
pub mod wal_types;
pub mod wal;
pub mod wal_appender;
pub mod checkpoint;
pub mod recovery;
pub mod wal_stats;

// COW enhancement modules
pub mod cow_map;
pub mod snapshot_chain;

// Capability-based access control
pub mod capability;

// Plugin integration
pub mod plugin;

// Audit logging
pub mod audit;

pub use superblock::*;
pub use inode::*;
pub use dir::*;
pub use journal::*;
pub use file::*;
pub use snapshot::*;
pub use posix::*;
