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

// ! Structlayout ABI Testing

use super::make_result;
use crate::compat::config::TargetArch;
use crate::compat::{TestResult, TestStatus};
use std::mem::{align_of, size_of};
use std::time::Instant;
use alloc::vec;
use alloc::format;
use alloc::vec::Vec;

/// runStructlayoutTesting
pub fn run_tests(arch: TargetArch) -> Vec<TestResult> {
    vec![
        test_basic_types(arch),
        test_pointer_size(arch),
        test_struct_alignment(arch),
        test_array_layout(arch),
        test_enum_layout(arch),
    ]
}

/// TestingbasebookTypesize
fn test_basic_types(arch: TargetArch) -> TestResult {
    let start = Instant::now();

    // VerificationbasebookTypesize
    let status = match arch {
        TargetArch::Arm64 | TargetArch::X64 => {
            // 64 positionArchitecture
            let u8_size = size_of::<u8>();
            let u16_size = size_of::<u16>();
            let u32_size = size_of::<u32>();
            let u64_size = size_of::<u64>();
            let usize_size = size_of::<usize>();

            if u8_size == 1 && u16_size == 2 && u32_size == 4 && u64_size == 8 && usize_size == 8 {
                TestStatus::Passed
            } else {
                TestStatus::Failed("Basic type size mismatch".to_string())
            }
        }
        _ => TestStatus::Skipped("Unknown architecture".to_string()),
    };

    make_result(
        "basic_types",
        arch,
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// Testingpointersize
fn test_pointer_size(arch: TargetArch) -> TestResult {
    let start = Instant::now();

    let status = match arch {
        TargetArch::Arm64 | TargetArch::X64 => {
            // 64 positionpointer
            let ptr_size = size_of::<*const u8>();
            if ptr_size == 8 {
                TestStatus::Passed
            } else {
                TestStatus::Failed(format!("Pointer size is {}, expected 8", ptr_size))
            }
        }
        _ => TestStatus::Skipped("Unknown architecture".to_string()),
    };

    make_result(
        "pointer_size",
        arch,
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// TestingStructalignment
fn test_struct_alignment(arch: TargetArch) -> TestResult {
    let start = Instant::now();

    #[repr(C)]
    struct TestStruct {
        a: u8,
        b: u32,
        c: u64,
    }

    let status = match arch {
        TargetArch::Arm64 | TargetArch::X64 => {
            let size = size_of::<TestStruct>();
            let align = align_of::<TestStruct>();

            // #[repr(C)] protectedcertificatelayout
            // a: 1 byte + 3 padding
            // b: 4 bytes
            // c: 8 bytes
            // total: 16 bytes, align 8
            if size == 16 && align == 8 {
                TestStatus::Passed
            } else {
                TestStatus::Failed(format!(
                    "Struct size={}, align={}, expected 16, 8",
                    size, align
                ))
            }
        }
        _ => TestStatus::Skipped("Unknown architecture".to_string()),
    };

    make_result(
        "struct_alignment",
        arch,
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// TestingArraylayout
fn test_array_layout(arch: TargetArch) -> TestResult {
    let start = Instant::now();

    let status = match arch {
        TargetArch::Arm64 | TargetArch::X64 => {
            let array_size = size_of::<[u32; 4]>();
            if array_size == 16 {
                TestStatus::Passed
            } else {
                TestStatus::Failed(format!("Array size is {}, expected 16", array_size))
            }
        }
        _ => TestStatus::Skipped("Unknown architecture".to_string()),
    };

    make_result(
        "array_layout",
        arch,
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// TestingEnumlayout
fn test_enum_layout(arch: TargetArch) -> TestResult {
    let start = Instant::now();

    #[repr(C)]
    enum TestEnum {
        A,
        B,
        C,
    }

    let status = match arch {
        TargetArch::Arm64 | TargetArch::X64 => {
            let enum_size = size_of::<TestEnum>();
            if enum_size == 4 {
                TestStatus::Passed
            } else {
                TestStatus::Failed(format!("Enum size is {}, expected 4", enum_size))
            }
        }
        _ => TestStatus::Skipped("Unknown architecture".to_string()),
    };

    make_result(
        "enum_layout",
        arch,
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// StructlayoutVerificationdevice
pub struct StructLayoutValidator {
    name: String,
    expected_size: usize,
    expected_align: usize,
}

impl StructLayoutValidator {
    pub fn new(name: impl Into<String>, expected_size: usize, expected_align: usize) -> Self {
        Self {
            name: name.into(),
            expected_size,
            expected_align,
        }
    }

    pub fn validate<T>(&self) -> Result<(), String> {
        let actual_size = size_of::<T>();
        let actual_align = align_of::<T>();

        if actual_size != self.expected_size {
            return Err(format!(
                "{}: size mismatch, expected {}, got {}",
                self.name, self.expected_size, actual_size
            ));
        }

        if actual_align != self.expected_align {
            return Err(format!(
                "{}: alignment mismatch, expected {}, got {}",
                self.name, self.expected_align, actual_align
            ));
        }

        Ok(())
    }
}
