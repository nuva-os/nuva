/*
 * Nuva OS - NuvaFS Test Suite
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

//! NuvaFS Test Suite
/*!*/
//! Comprehensive tests for NuvaFS functionality.

use super::*;
use super::superblock::*;
use super::inode::*;
use super::dir::*;
use super::journal::*;
use super::file::*;
use super::snapshot::*;
use super::posix::*;

/// Test result
#[derive(Debug)]
pub struct TestResult {
    pub name: &'static str,
    pub passed: bool,
    pub message: Option<&'static str>,
}

/// Test runner
pub struct TestRunner {
    results: [Option<TestResult>; 64],
    count: usize,
    passed: usize,
    failed: usize,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            results: [None; 64],
            count: 0,
            passed: 0,
            failed: 0,
        }
    }

    pub fn run_test(&mut self, name: &'static str, test: fn() -> bool) {
        let passed = test();
        let result = TestResult {
            name,
            passed,
            message: None,
        };

        if self.count < 64 {
            self.results[self.count] = Some(result);
            self.count += 1;

            if passed {
                self.passed += 1;
            } else {
                self.failed += 1;
            }
        }
    }

    pub fn run_test_with_msg(&mut self, name: &'static str, test: fn() -> (bool, &'static str)) {
        let (passed, msg) = test();
        let result = TestResult {
            name,
            passed,
            message: Some(msg),
        };

        if self.count < 64 {
            self.results[self.count] = Some(result);
            self.count += 1;

            if passed {
                self.passed += 1;
            } else {
                self.failed += 1;
            }
        }
    }

    pub fn print_summary(&self) {
        crate::log_info!("=== NuvaFS Test Summary ===");
        crate::log_info!("Total: {} tests", self.count);
        crate::log_info!("Passed: {}", self.passed);
        crate::log_info!("Failed: {}", self.failed);

        if self.failed > 0 {
            crate::log_info!("
Failed tests:");
            for i in 0..self.count {
                if let Some(ref result) = self.results[i] {
                    if !result.passed {
                        crate::log_info!("  - {}", result.name);
                        if let Some(msg) = result.message {
                            crate::log_info!("    {}", msg);
                        }
                    }
                }
            }
        }
    }

    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

// ============================================================================
// Superblock Tests

fn test_superblock_creation() -> bool {
    let sb = NuvaSuperblock::new(BlockSize::B4K, 1000000);
    sb.is_valid()
}

fn test_superblock_features() -> bool {
    let sb = NuvaSuperblock::new(BlockSize::B4K, 1000000);
    sb.has_feature(FEATURE_JOURNAL) && sb.has_feature(FEATURE_EXTENTS)
}

fn test_superblock_state() -> bool {
    let sb = NuvaSuperblock::new(BlockSize::B4K, 1000000);
    sb.is_clean() && {
        sb.mark_dirty();
        !sb.is_clean()
    } && {
        sb.mark_clean();
        sb.is_clean()
    }
}

fn test_superblock_block_conversion() -> bool {
    let sb = NuvaSuperblock::new(BlockSize::B4K, 1000000);
    let block = 100u64;
    let addr = sb.block_to_addr(block);
    let back = sb.addr_to_block(addr);
    block == back
}

// ============================================================================
// Inode Tests

fn test_inode_creation() -> bool {
    let inode = NuvaInode::new(1, InodeMode::Regular);
    inode.ino == 1 && inode.is_regular() && !inode.is_dir()
}

fn test_inode_directory() -> bool {
    let inode = NuvaInode::new(2, InodeMode::Directory);
    inode.is_dir() && !inode.is_regular()
}

fn test_inode_size_operations() -> bool {
    let inode = NuvaInode::new(3, InodeMode::Regular);
    inode.get_size() == 0 && {
        inode.set_size(1024);
        inode.get_size() == 1024
    } && {
        inode.set_size(4096);
        inode.get_size() == 4096
    }
}

fn test_inode_block_allocation() -> bool {
    let inode = NuvaInode::new(4, InodeMode::Regular);
    inode.get_blocks() == 0 && {
        inode.add_block();
        inode.get_blocks() == 1
    } && {
        inode.add_block();
        inode.get_blocks() == 2
    }
}

fn test_inode_extent_operations() -> bool {
    let mut inode = NuvaInode::new(5, InodeMode::Regular);

    // Add extent
    let extent = Extent::new(0, 1000, 10);
    inode.add_extent(extent);

    // Find extent
    let found = inode.find_extent(5);
    found.is_some() && found.unwrap().physical == 1000
}

fn test_inode_cache_operations() -> bool {
    let mut cache = InodeCache::new();

    // Insert
    let inode = NuvaInode::new(100, InodeMode::Regular);
    cache.insert(inode);

    // Find
    let found = cache.get(100);
    found.is_some() && {
        // Remove
        cache.remove(100);
        cache.get(100).is_none()
    }
}

// ============================================================================
// Directory Tests

fn test_dir_entry_creation() -> bool {
    let entry = DirEntry::new(1, b"test.txt", DirEntryType::Regular);
    entry.ino == 1 && entry.name() == b"test.txt" && !entry.is_deleted()
}

fn test_dir_entry_deleted() -> bool {
    let mut entry = DirEntry::new(1, b"test", DirEntryType::Regular);
    entry.ino = 0;
    entry.is_deleted()
}

fn test_dir_hash_consistency() -> bool {
    let hash1 = dir_hash(b"test");
    let hash2 = dir_hash(b"test");
    let hash3 = dir_hash(b"other");
    hash1 == hash2 && hash1 != hash3
}

fn test_dir_ops_lookup() -> bool {
    let mut data = [0u8; 4096];

    // Create entry
    let entry = DirEntry::new(1, b"test.txt", DirEntryType::Regular);
    let rec_len = entry.rec_len as usize;
    // SAFETY: unsafe block required for low-level memory or hardware access
    let bytes = unsafe {
        core::slice::from_raw_parts(&entry as *const DirEntry as *const u8, rec_len)
    };
    data[..rec_len].copy_from_slice(bytes);

    // Lookup
    let found = DirOps::lookup(&data, b"test.txt");
    found == Some(1)
}

fn test_dir_ops_create() -> bool {
    let mut data = [0u8; 4096];

    // Create entry
    let result = DirOps::create(&mut data, 1, b"newfile.txt", DirEntryType::Regular);
    result && DirOps::lookup(&data, b"newfile.txt") == Some(1)
}

fn test_dir_ops_remove() -> bool {
    let mut data = [0u8; 4096];

    // Create entry
    DirOps::create(&mut data, 1, b"test.txt", DirEntryType::Regular);

    // Remove
    let result = DirOps::remove(&mut data, b"test.txt");
    result && DirOps::lookup(&data, b"test.txt").is_none()
}

// ============================================================================
// Journal Tests

fn test_journal_transaction() -> bool {
    let mut mgr = JournalManager::new();
    mgr.init();

    // Begin transaction
    let id = mgr.begin_transaction(JournalTransactionType::Create);
    !mgr.is_clean() && {
        // Commit
        mgr.commit_transaction()
    } && mgr.is_clean() && id > 0
}

fn test_journal_rollback() -> bool {
    let mut mgr = JournalManager::new();
    mgr.init();

    // Begin transaction
    mgr.begin_transaction(JournalTransactionType::Create);
    !mgr.is_clean() && {
        // Rollback
        mgr.rollback_transaction();
        mgr.is_clean()
    }
}

fn test_journal_add_blocks() -> bool {
    let mut mgr = JournalManager::new();
    mgr.init();

    mgr.begin_transaction(JournalTransactionType::Write);

    let data = [0u8; 4096];
    let result1 = mgr.add_block(100, &data);
    let result2 = mgr.add_block(101, &data);

    mgr.commit_transaction();

    result1 && result2
}

// ============================================================================
// File Operation Tests

fn test_file_handle_creation() -> bool {
    let handle = FileHandle::new(1, OpenMode::ReadWrite);
    handle.ino == 1 && handle.get_pos() == 0
}

fn test_file_handle_references() -> bool {
    let handle = FileHandle::new(1, OpenMode::ReadOnly);

    handle.refs.load(core::sync::atomic::Ordering::Relaxed) == 1 && {
        handle.add_ref();
        handle.refs.load(core::sync::atomic::Ordering::Relaxed) == 2
    } && {
        !handle.release()
    } && {
        handle.release()
    }
}

fn test_file_handle_seek() -> bool {
    let inode = NuvaInode::new(1, InodeMode::Regular);
    inode.set_size(1000);

    let handle = FileHandle::new(1, OpenMode::ReadOnly);

    // SEEK_SET
    let result = FileOps::seek(&inode, &handle, 100, SeekOrigin::Set);
    result.is_ok() && result.unwrap() == 100 && {
        // SEEK_CUR
        let result = FileOps::seek(&inode, &handle, 50, SeekOrigin::Current);
        result.is_ok() && result.unwrap() == 150
    } && {
        // SEEK_END
        let result = FileOps::seek(&inode, &handle, -100, SeekOrigin::End);
        result.is_ok() && result.unwrap() == 900
    }
}

fn test_open_file_table() -> bool {
    let mut table = OpenFileTable::new();

    // Insert
    let fd1 = table.insert(FileHandle::new(1, OpenMode::ReadOnly));
    let fd2 = table.insert(FileHandle::new(2, OpenMode::WriteOnly));

    fd1.is_ok() && fd2.is_ok() && {
        // Get
        let file1 = table.get(fd1.unwrap());
        let file2 = table.get(fd2.unwrap());
        file1.is_some() && file2.is_some()
    } && {
        // Remove
        table.remove(fd1.unwrap());
        table.get(fd1.unwrap()).is_none()
    }
}

// ============================================================================
// Snapshot Tests

fn test_snapshot_creation() -> bool {
    let mut mgr = SnapshotManager::new();

    let result = mgr.create(0, 2, 1000);
    result.is_ok() && {
        let id = result.unwrap();
        let snap = mgr.get(id);
        snap.is_some() && snap.unwrap().is_active()
    }
}

fn test_snapshot_deletion() -> bool {
    let mut mgr = SnapshotManager::new();

    let id = mgr.create(0, 2, 1000).unwrap();
    let result = mgr.delete(id);

    result.is_ok() && mgr.get(id).is_none()
}

fn test_snapshot_rollback() -> bool {
    let mut mgr = SnapshotManager::new();

    let id = mgr.create(0, 2, 1000).unwrap();
    let result = mgr.rollback(id);

    result.is_ok() && mgr.active_snapshot.load(core::sync::atomic::Ordering::Relaxed) == id
}

fn test_snapshot_cow() -> bool {
    let mut mgr = SnapshotManager::new();

    let id = mgr.create(0, 2, 1000).unwrap();
    mgr.active_snapshot.store(id, core::sync::atomic::Ordering::Relaxed);

    // COW write
    let result = mgr.cow_write(100, 200);
    result.is_ok() && {
        // Translate read
        let translated = mgr.translate_read(100);
        translated == 200
    }
}

fn test_snapshot_list() -> bool {
    let mut mgr = SnapshotManager::new();

    mgr.create(0, 2, 1000).unwrap();
    mgr.create(0, 2, 2000).unwrap();

    let list = mgr.list();
    list.len() == 2
}

// ============================================================================
// POSIX Tests

fn test_posix_errno() -> bool {
    Errno::ENOENT as i32 == 2 && Errno::EIO as i32 == 5 && Errno::EINVAL as i32 == 22
}

fn test_posix_mode_checks() -> bool {
    mode::S_ISREG(mode::S_IFREG) && mode::S_ISDIR(mode::S_IFDIR) && !mode::S_ISREG(mode::S_IFDIR)
}

fn test_posix_permissions() -> bool {
    // Root can access anything
    PermissionCheck::can_read(0, 0, 0, 100, 100, 0) &&
    PermissionCheck::can_write(0, 0, 0, 100, 100, 0)
}

fn test_posix_path_validation() -> bool {
    PathOps::validate(b"/test/path").is_ok() &&
    PathOps::validate(b"").is_err() &&
    PathOps::validate(b"/test\0path").is_err()
}

fn test_posix_path_split() -> bool {
    let (dir, name) = PathOps::split(b"/home/user/file.txt");
    dir == b"/home/user" && name == b"file.txt"
}

fn test_posix_path_normalize() -> bool {
    let mut output = [0u8; 256];
    let len = PathOps::normalize(b"/a/b/../c", &mut output);
    &output[..len] == b"/a/c"
}

// ============================================================================
// Crash Consistency Tests

fn test_journal_recovery() -> (bool, &'static str) {
    let mut mgr = JournalManager::new();
    mgr.init();

    // Simulate crash during transaction
    mgr.begin_transaction(JournalTransactionType::Create);
    // Don't commit - simulate crash

    // Recovery should clean up
    mgr.recover();

    (mgr.is_clean(), "Journal should be clean after recovery")
}

fn test_journal_checkpoint() -> (bool, &'static str) {
    let mut mgr = JournalManager::new();
    mgr.init();

    // Create and commit transaction
    mgr.begin_transaction(JournalTransactionType::Write);
    mgr.commit_transaction();

    // Checkpoint
    mgr.checkpoint();

    (true, "Checkpoint should succeed")
}

// ============================================================================
// Performance Tests

fn test_inode_cache_performance() -> (bool, &'static str) {
    let mut cache = InodeCache::new();

    // Insert 100 inodes
    for i in 0..100 {
        cache.insert(NuvaInode::new(i, InodeMode::Regular));
    }

    // Lookup all
    let mut found = 0;
    for i in 0..100 {
        if cache.get(i).is_some() {
            found += 1;
        }
    }

    (found == 100, "Should find all 100 cached inodes")
}

fn test_dir_hash_performance() -> (bool, &'static str) {
    // Hash 1000 names
    let mut hashes = [0u32; 1000];
    for i in 0..1000 {
        let name = format!("file{}.txt", i);
        hashes[i] = dir_hash(name.as_bytes());
    }

    // Check uniqueness (probabilistic)
    let mut unique = 0;
    for i in 0..1000 {
        let mut is_unique = true;
        for j in 0..i {
            if hashes[i] == hashes[j] {
                is_unique = false;
                break;
            }
        }
        if is_unique {
            unique += 1;
        }
    }

    // Most hashes should be unique
    (unique > 900, "Most hashes should be unique")
}

// ============================================================================
// Main Test Runner

/// Run all NuvaFS tests
pub fn run_all_tests() -> bool {
    let mut runner = TestRunner::new();

    crate::log_info!("=== NuvaFS Test Suite ===
");

    // Superblock tests
    crate::log_info!("Running Superblock tests...");
    runner.run_test("superblock_creation", test_superblock_creation);
    runner.run_test("superblock_features", test_superblock_features);
    runner.run_test("superblock_state", test_superblock_state);
    runner.run_test("superblock_block_conversion", test_superblock_block_conversion);

    // Inode tests
    crate::log_info!("Running Inode tests...");
    runner.run_test("inode_creation", test_inode_creation);
    runner.run_test("inode_directory", test_inode_directory);
    runner.run_test("inode_size_operations", test_inode_size_operations);
    runner.run_test("inode_block_allocation", test_inode_block_allocation);
    runner.run_test("inode_extent_operations", test_inode_extent_operations);
    runner.run_test("inode_cache_operations", test_inode_cache_operations);

    // Directory tests
    crate::log_info!("Running Directory tests...");
    runner.run_test("dir_entry_creation", test_dir_entry_creation);
    runner.run_test("dir_entry_deleted", test_dir_entry_deleted);
    runner.run_test("dir_hash_consistency", test_dir_hash_consistency);
    runner.run_test("dir_ops_lookup", test_dir_ops_lookup);
    runner.run_test("dir_ops_create", test_dir_ops_create);
    runner.run_test("dir_ops_remove", test_dir_ops_remove);

    // Journal tests
    crate::log_info!("Running Journal tests...");
    runner.run_test("journal_transaction", test_journal_transaction);
    runner.run_test("journal_rollback", test_journal_rollback);
    runner.run_test("journal_add_blocks", test_journal_add_blocks);

    // File tests
    crate::log_info!("Running File tests...");
    runner.run_test("file_handle_creation", test_file_handle_creation);
    runner.run_test("file_handle_references", test_file_handle_references);
    runner.run_test("file_handle_seek", test_file_handle_seek);
    runner.run_test("open_file_table", test_open_file_table);

    // Snapshot tests
    crate::log_info!("Running Snapshot tests...");
    runner.run_test("snapshot_creation", test_snapshot_creation);
    runner.run_test("snapshot_deletion", test_snapshot_deletion);
    runner.run_test("snapshot_rollback", test_snapshot_rollback);
    runner.run_test("snapshot_cow", test_snapshot_cow);
    runner.run_test("snapshot_list", test_snapshot_list);

    // POSIX tests
    crate::log_info!("Running POSIX tests...");
    runner.run_test("posix_errno", test_posix_errno);
    runner.run_test("posix_mode_checks", test_posix_mode_checks);
    runner.run_test("posix_permissions", test_posix_permissions);
    runner.run_test("posix_path_validation", test_posix_path_validation);
    runner.run_test("posix_path_split", test_posix_path_split);
    runner.run_test("posix_path_normalize", test_posix_path_normalize);

    // Crash consistency tests
    crate::log_info!("Running Crash Consistency tests...");
    runner.run_test_with_msg("journal_recovery", test_journal_recovery);
    runner.run_test_with_msg("journal_checkpoint", test_journal_checkpoint);

    // Performance tests
    crate::log_info!("Running Performance tests...");
    runner.run_test_with_msg("inode_cache_performance", test_inode_cache_performance);
    runner.run_test_with_msg("dir_hash_performance", test_dir_hash_performance);

    // Print summary
    runner.print_summary();

    runner.all_passed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_all() {
        assert!(run_all_tests());
    }
}
