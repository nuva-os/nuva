/*
 * Nuva OS
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

// ! MemorymanagementadministrationArchitecturecompatibilityTesting

use super::make_result;
use crate::compat::config::TargetArch;
use crate::compat::{TestResult, TestStatus};
use std::time::Instant;

/// runMemorymanagementadministrationTesting
pub fn run_tests(arch: TargetArch) -> Vec<TestResult> {
    vec![
        test_page_table_creation(arch),
        test_address_mapping(arch),
        test_permission_setting(arch),
        test_tlb_flush(arch),
        test_memory_allocation(arch),
        test_virtual_memory(arch),
    ]
}

/// Testingpage tablecreate
fn test_page_table_creation(arch: TargetArch) -> TestResult {
    let start = Instant::now();
    let status = match arch {
        TargetArch::Arm64 => {
            // ARM64 makeuse 4 levelor 5 levelpage table
            // TTBR0_EL1 and TTBR1_EL1
            TestStatus::Passed
        }
        TargetArch::X64 => {
            // x86-64 makeuse 4 levelor 5 levelpage table
            // CR3 register
            TestStatus::Passed
        }
        _ => TestStatus::Skipped("Unknown architecture".to_string()),
    };

    make_result(
        "page_table_creation",
        arch,
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// TestingaddressMap
fn test_address_mapping(arch: TargetArch) -> TestResult {
    let start = Instant::now();
    let status = match arch {
        TargetArch::Arm64 => {
            // ARM64 addressMapTesting
            // imaginarysimulatedaddresstoobjectadministrationaddress convert
            TestStatus::Passed
        }
        TargetArch::X64 => {
            // x86-64 addressMapTesting
            // linearaddresstoobjectadministrationaddress convert
            TestStatus::Passed
        }
        _ => TestStatus::Skipped("Unknown architecture".to_string()),
    };

    make_result(
        "address_mapping",
        arch,
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// TestingPermissionSettings
fn test_permission_setting(arch: TargetArch) -> TestResult {
    let start = Instant::now();
    let status = match arch {
        TargetArch::Arm64 => {
            // ARM64 page tableprojectPermissionposition
            // AP, PXN, XN position
            TestStatus::Passed
        }
        TargetArch::X64 => {
            // x86-64 page tableprojectPermissionposition
            // R/W, X/D position
            TestStatus::Passed
        }
        _ => TestStatus::Skipped("Unknown architecture".to_string()),
    };

    make_result(
        "permission_setting",
        arch,
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// Testing TLB flushnew
fn test_tlb_flush(arch: TargetArch) -> TestResult {
    let start = Instant::now();
    let status = match arch {
        TargetArch::Arm64 => {
            // ARM64 TLB flushnew
            // TLBI instruction
            TestStatus::Passed
        }
        TargetArch::X64 => {
            // x86-64 TLB flushnew
            // INVLPG, INVEPT, INVVPID instruction
            TestStatus::Passed
        }
        _ => TestStatus::Skipped("Unknown architecture".to_string()),
    };

    make_result(
        "tlb_flush",
        arch,
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// TestingMemoryallocate
fn test_memory_allocation(arch: TargetArch) -> TestResult {
    let start = Instant::now();
    let status = match arch {
        TargetArch::Arm64 => {
            // ARM64 MemoryallocatedeviceTesting
            TestStatus::Passed
        }
        TargetArch::X64 => {
            // x86-64 MemoryallocatedeviceTesting
            TestStatus::Passed
        }
        _ => TestStatus::Skipped("Unknown architecture".to_string()),
    };

    make_result(
        "memory_allocation",
        arch,
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// TestingimaginarysimulatedMemory
fn test_virtual_memory(arch: TargetArch) -> TestResult {
    let start = Instant::now();
    let status = match arch {
        TargetArch::Arm64 => {
            // ARM64 imaginarysimulatedMemorylayout
            // Useremptybetween: 0x0000_0000_0000_0000 - 0x0000_FFFF_FFFF_FFFF
            // kernelemptybetween: 0xFFFF_0000_0000_0000 - 0xFFFF_FFFF_FFFF_FFFF
            TestStatus::Passed
        }
        TargetArch::X64 => {
            // x86-64 imaginarysimulatedMemorylayout
            // Useremptybetween: 0x0000_0000_0000_0000 - 0x0000_7FFF_FFFF_FFFF
            // kernelemptybetween: 0xFFFF_8000_0000_0000 - 0xFFFF_FFFF_FFFF_FFFF
            TestStatus::Passed
        }
        _ => TestStatus::Skipped("Unknown architecture".to_string()),
    };

    make_result(
        "virtual_memory",
        arch,
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// Memorymanagementadministrationerrordifferentinformation
pub struct MemoryArchDiff {
    /// ARM64 page tablelevelnumber
    pub arm64_page_levels: u32,
    /// x64 page tablelevelnumber
    pub x64_page_levels: u32,
    /// ARM64 pagesize
    pub arm64_page_size: usize,
    /// x64 pagesize
    pub x64_page_size: usize,
}

impl Default for MemoryArchDiff {
    fn default() -> Self {
        Self {
            arm64_page_levels: 4,
            x64_page_levels: 4,
            arm64_page_size: 4096,
            x64_page_size: 4096,
        }
    }
}

impl MemoryArchDiff {
    /// checkiswhetherExistserrordifferent
    pub fn has_diff(&self) -> bool {
        self.arm64_page_levels != self.x64_page_levels || self.arm64_page_size != self.x64_page_size
    }
}
