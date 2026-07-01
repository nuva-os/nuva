/*
 * Nuva OS - HAL - Kernel Callback Interface
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 */

//! HAL callback interface for kernel services.
//!
//! HAL (L0) must NOT directly import kernel (L1) modules.
//! Instead, the kernel registers callback functions at init time,
//! and HAL calls them through function pointers stored here.

use core::sync::atomic::{AtomicBool, Ordering};

type PageAllocFn = fn() -> u64;
type TimeMsFn = fn() -> u64;
type SbiHartStartFn = fn(u64, u64, u64) -> SbiRet;
type SbiHartSuspendFn = fn(u64, u64, u64) -> SbiRet;
type AiWakeupBoostFn = fn(u32, u32) -> i32;
type AiLatencyPickFn = fn(u32, usize) -> usize;

/// SBI return structure (mirrors kernel::arch::riscv64::sbi::SbiRet)
#[derive(Debug, Clone, Copy)]
pub struct SbiRet {
    pub error: i64,
    pub value: i64,
}

pub const SBI_SUCCESS: i64 = 0;

static PAGE_ALLOC: spin::Mutex<Option<PageAllocFn>> = spin::Mutex::new(None);
static TIME_MS: spin::Mutex<Option<TimeMsFn>> = spin::Mutex::new(None);
static SBI_HART_START: spin::Mutex<Option<SbiHartStartFn>> = spin::Mutex::new(None);
static SBI_HART_SUSPEND: spin::Mutex<Option<SbiHartSuspendFn>> = spin::Mutex::new(None);
static AI_WAKEUP_BOOST: spin::Mutex<Option<AiWakeupBoostFn>> = spin::Mutex::new(None);
static AI_LATENCY_PICK: spin::Mutex<Option<AiLatencyPickFn>> = spin::Mutex::new(None);
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Register the page allocation callback from kernel
pub fn register_page_alloc(f: PageAllocFn) {
    *PAGE_ALLOC.lock() = Some(f);
}

/// Register the time source callback from kernel
pub fn register_time_ms(f: TimeMsFn) {
    *TIME_MS.lock() = Some(f);
}

/// Register SBI hart_start callback from kernel
pub fn register_sbi_hart_start(f: SbiHartStartFn) {
    *SBI_HART_START.lock() = Some(f);
}

/// Register SBI hart_suspend callback from kernel
pub fn register_sbi_hart_suspend(f: SbiHartSuspendFn) {
    *SBI_HART_SUSPEND.lock() = Some(f);
}

/// Register AI scheduler wakeup boost callback from kernel
pub fn register_ai_wakeup_boost(f: AiWakeupBoostFn) {
    *AI_WAKEUP_BOOST.lock() = Some(f);
}

/// Register AI scheduler latency pick callback from kernel
pub fn register_ai_latency_pick(f: AiLatencyPickFn) {
    *AI_LATENCY_PICK.lock() = Some(f);
}

/// Mark callbacks as initialized
pub fn mark_initialized() {
    INITIALIZED.store(true, Ordering::Release);
}

/// HAL-internal: allocate a page via kernel callback.
/// Returns 0 if no callback registered or allocation fails.
pub fn hal_alloc_page() -> u64 {
    if let Some(f) = *PAGE_ALLOC.lock() {
        f()
    } else {
        0
    }
}

/// HAL-internal: get current time in ms via kernel callback.
/// Returns 0 if no callback registered.
pub fn hal_get_time_ms() -> u64 {
    if let Some(f) = *TIME_MS.lock() {
        f()
    } else {
        0
    }
}

/// HAL-internal: start a hart via SBI callback.
/// Returns SbiRet with error=-1 if no callback registered.
pub fn hal_sbi_hart_start(hartid: u64, start_addr: u64, opaque: u64) -> SbiRet {
    if let Some(f) = *SBI_HART_START.lock() {
        f(hartid, start_addr, opaque)
    } else {
        SbiRet { error: -1, value: 0 }
    }
}

/// HAL-internal: suspend a hart via SBI callback.
/// Returns SbiRet with error=-1 if no callback registered.
pub fn hal_sbi_hart_suspend(suspend_type: u64, resume_addr: u64, opaque: u64) -> SbiRet {
    if let Some(f) = *SBI_HART_SUSPEND.lock() {
        f(suspend_type, resume_addr, opaque)
    } else {
        SbiRet { error: -1, value: 0 }
    }
}

/// HAL-internal: get AI wakeup boost from kernel scheduler.
/// Returns 0 if no callback registered.
pub fn hal_ai_wakeup_boost(task_class: u32, expected_latency_ms: u32) -> i32 {
    if let Some(f) = *AI_WAKEUP_BOOST.lock() {
        f(task_class, expected_latency_ms)
    } else {
        0
    }
}

/// HAL-internal: get AI latency-aware CPU pick from kernel scheduler.
/// Returns prev_cpu if no callback registered.
pub fn hal_ai_latency_pick(task_class: u32, prev_cpu: usize) -> usize {
    if let Some(f) = *AI_LATENCY_PICK.lock() {
        f(task_class, prev_cpu)
    } else {
        prev_cpu
    }
}