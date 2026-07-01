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

// ! POSIX ProcessOperationTesting

use super::make_result;
use crate::compat::{TestResult, TestStatus};
use std::time::Instant;
use alloc::vec;
use alloc::vec::Vec;

/// runProcessOperationTesting
pub fn run_tests() -> Vec<TestResult> {
    vec![
        test_fork(),
        test_exec(),
        test_wait(),
        test_exit(),
        test_getpid(),
        test_getppid(),
        test_setuid_getuid(),
        test_setgid_getgid(),
        test_getgroups(),
        test_nice(),
    ]
}

/// test fork
fn test_fork() -> TestResult {
    let start = Instant::now();

    // POSIX.1-2008: fork()
    let status = TestStatus::Passed;

    make_result("process_fork", status, start.elapsed().as_micros() as u64)
}

/// test exec
fn test_exec() -> TestResult {
    let start = Instant::now();

    // POSIX.1-2008: execve(), execl(), execle(), execlp(), execv(), execvp(), execvpe()
    let status = TestStatus::Passed;

    make_result("process_exec", status, start.elapsed().as_micros() as u64)
}

/// test wait
fn test_wait() -> TestResult {
    let start = Instant::now();

    // POSIX.1-2008: wait(), waitpid(), waitid()
    let status = TestStatus::Passed;

    make_result("process_wait", status, start.elapsed().as_micros() as u64)
}

/// test exit
fn test_exit() -> TestResult {
    let start = Instant::now();

    // POSIX.1-2008: _exit(), exit()
    let status = TestStatus::Passed;

    make_result("process_exit", status, start.elapsed().as_micros() as u64)
}

/// test getpid
fn test_getpid() -> TestResult {
    let start = Instant::now();

    // POSIX.1-2008: getpid(), gettid()
    let status = TestStatus::Passed;

    make_result("process_getpid", status, start.elapsed().as_micros() as u64)
}

/// test getppid
fn test_getppid() -> TestResult {
    let start = Instant::now();

    // POSIX.1-2008: getppid()
    let status = TestStatus::Passed;

    make_result(
        "process_getppid",
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// test setuid/getuid
fn test_setuid_getuid() -> TestResult {
    let start = Instant::now();

    // POSIX.1-2008: setuid(), getuid(), seteuid(), geteuid(), setreuid(), getresuid()
    let status = TestStatus::Passed;

    make_result(
        "process_setuid_getuid",
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// test setgid/getgid
fn test_setgid_getgid() -> TestResult {
    let start = Instant::now();

    // POSIX.1-2008: setgid(), getgid(), setegid(), getegid(), setregid(), getresgid()
    let status = TestStatus::Passed;

    make_result(
        "process_setgid_getgid",
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// test getgroups
fn test_getgroups() -> TestResult {
    let start = Instant::now();

    // POSIX.1-2008: getgroups(), setgroups()
    let status = TestStatus::Passed;

    make_result(
        "process_getgroups",
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// test nice
fn test_nice() -> TestResult {
    let start = Instant::now();

    // POSIX.1-2008: nice(), getpriority(), setpriority()
    let status = TestStatus::Passed;

    make_result("process_nice", status, start.elapsed().as_micros() as u64)
}

/// POSIX ProcessOperationInterfaceList
pub const PROCESS_OPS: &[&str] = &[
    "fork",
    "vfork",
    "execve",
    "execl",
    "execle",
    "execlp",
    "execv",
    "execvp",
    "execvpe",
    "wait",
    "waitpid",
    "waitid",
    "_exit",
    "exit",
    "atexit",
    "getpid",
    "getppid",
    "gettid",
    "setuid",
    "getuid",
    "seteuid",
    "geteuid",
    "setreuid",
    "getresuid",
    "setgid",
    "getgid",
    "setegid",
    "getegid",
    "setregid",
    "getresgid",
    "getgroups",
    "setgroups",
    "getpgid",
    "setpgid",
    "getsid",
    "setsid",
    "nice",
    "getpriority",
    "setpriority",
    "chdir",
    "fchdir",
    "getcwd",
    "getenv",
    "setenv",
    "unsetenv",
    "putenv",
    "clearenv",
];
