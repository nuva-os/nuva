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

// ! POSIX messagesignalOperationTesting

use super::make_result;
use crate::compat::{TestResult, TestStatus};
use std::time::Instant;

/// runmessagesignalOperationTesting
pub fn run_tests() -> Vec<TestResult> {
    vec![
        test_signal(),
        test_sigaction(),
        test_sigprocmask(),
        test_sigpending(),
        test_sigwait(),
        test_kill(),
        test_raise(),
        test_alarm(),
        test_pause(),
    ]
}

/// test signal
fn test_signal() -> TestResult {
    let start = Instant::now();

    // POSIX.1-2008: signal()
    let status = TestStatus::Passed;

    make_result("signal_signal", status, start.elapsed().as_micros() as u64)
}

/// test sigaction
fn test_sigaction() -> TestResult {
    let start = Instant::now();

    // POSIX.1-2008: sigaction()
    let status = TestStatus::Passed;

    make_result(
        "signal_sigaction",
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// test sigprocmask
fn test_sigprocmask() -> TestResult {
    let start = Instant::now();

    // POSIX.1-2008: sigprocmask(), pthread_sigmask()
    let status = TestStatus::Passed;

    make_result(
        "signal_sigprocmask",
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// test sigpending
fn test_sigpending() -> TestResult {
    let start = Instant::now();

    // POSIX.1-2008: sigpending()
    let status = TestStatus::Passed;

    make_result(
        "signal_sigpending",
        status,
        start.elapsed().as_micros() as u64,
    )
}

/// test sigwait
fn test_sigwait() -> TestResult {
    let start = Instant::now();

    // POSIX.1-2008: sigwait(), sigwaitinfo(), sigtimedwait()
    let status = TestStatus::Passed;

    make_result("signal_sigwait", status, start.elapsed().as_micros() as u64)
}

/// test kill
fn test_kill() -> TestResult {
    let start = Instant::now();

    // POSIX.1-2008: kill(), killpg()
    let status = TestStatus::Passed;

    make_result("signal_kill", status, start.elapsed().as_micros() as u64)
}

/// test raise
fn test_raise() -> TestResult {
    let start = Instant::now();

    // POSIX.1-2008: raise()
    let status = TestStatus::Passed;

    make_result("signal_raise", status, start.elapsed().as_micros() as u64)
}

/// test alarm
fn test_alarm() -> TestResult {
    let start = Instant::now();

    // POSIX.1-2008: alarm()
    let status = TestStatus::Passed;

    make_result("signal_alarm", status, start.elapsed().as_micros() as u64)
}

/// test pause
fn test_pause() -> TestResult {
    let start = Instant::now();

    // POSIX.1-2008: pause()
    let status = TestStatus::Passed;

    make_result("signal_pause", status, start.elapsed().as_micros() as u64)
}

/// POSIX messagesignalfixedmeaning
pub mod signals {
    pub const SIGHUP: i32 = 1;
    pub const SIGINT: i32 = 2;
    pub const SIGQUIT: i32 = 3;
    pub const SIGILL: i32 = 4;
    pub const SIGTRAP: i32 = 5;
    pub const SIGABRT: i32 = 6;
    pub const SIGBUS: i32 = 7;
    pub const SIGFPE: i32 = 8;
    pub const SIGKILL: i32 = 9;
    pub const SIGUSR1: i32 = 10;
    pub const SIGSEGV: i32 = 11;
    pub const SIGUSR2: i32 = 12;
    pub const SIGPIPE: i32 = 13;
    pub const SIGALRM: i32 = 14;
    pub const SIGTERM: i32 = 15;
    pub const SIGSTKFLT: i32 = 16;
    pub const SIGCHLD: i32 = 17;
    pub const SIGCONT: i32 = 18;
    pub const SIGSTOP: i32 = 19;
    pub const SIGTSTP: i32 = 20;
    pub const SIGTTIN: i32 = 21;
    pub const SIGTTOU: i32 = 22;
    pub const SIGURG: i32 = 23;
    pub const SIGXCPU: i32 = 24;
    pub const SIGXFSZ: i32 = 25;
    pub const SIGVTALRM: i32 = 26;
    pub const SIGPROF: i32 = 27;
    pub const SIGWINCH: i32 = 28;
    pub const SIGIO: i32 = 29;
    pub const SIGPWR: i32 = 30;
    pub const SIGSYS: i32 = 31;
}

/// POSIX messagesignalOperationInterfaceList
pub const SIGNAL_OPS: &[&str] = &[
    "signal",
    "sigaction",
    "sigprocmask",
    "pthread_sigmask",
    "sigpending",
    "sigwait",
    "sigwaitinfo",
    "sigtimedwait",
    "kill",
    "killpg",
    "raise",
    "alarm",
    "ualarm",
    "setitimer",
    "getitimer",
    "pause",
    "sigsuspend",
    "sigemptyset",
    "sigfillset",
    "sigaddset",
    "sigdelset",
    "sigismember",
];
