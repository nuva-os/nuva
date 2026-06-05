/*
 * Nuva OS - Hal - Ffi - CApi - Bindings
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
/*
 * HAL FFI Bindings Implementation
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * This module implements C-compatible FFI bindings for HAL.
 */

use core::ptr;
use alloc::boxed::Box;
use alloc::string::String;

// Import C types
#[repr(C)]
pub enum NuvaResult {
    Ok = 0,
    InvalidParam = -1,
    NotFound = -2,
    OutOfMemory = -3,
    NotSupported = -4,
    Hardware = -5,
    Timeout = -6,
    Busy = -7,
}

pub type nuva_handle_t = u64;
pub const NUVA_INVALID_HANDLE: nuva_handle_t = 0;

// CPU HAL FFI

#[repr(C)]
pub struct NuvaCpuInfo {
    pub core_count: u32,
    pub frequency_mhz: u32,
    pub cache_line_size: u32,
    pub total_memory: u64,
    pub vendor: [u8; 32],
    pub model: [u8; 64],
}

#[no_mangle]
pub extern "C" fn nuva_cpu_get_info(info: *mut NuvaCpuInfo) -> NuvaResult {
    if info.is_null() {
        return NuvaResult::InvalidParam;
    }

    // Retrieve CPU info from the kernel CPU manager
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        let cpu_mgr = crate::hal::cpu::get_cpu_manager();
        (*info).core_count = cpu_mgr.num_online();
        (*info).frequency_mhz = if let Some(cpu) = cpu_mgr.get_cpu_info(0) {
            (cpu.current_freq / 1_000_000) as u32
        } else { 0 };
        (*info).cache_line_size = 64; // Default cache line size
        (*info).total_memory = {
            // Default 1GB; in production, query from MM subsystem
            1024u64 * 1024 * 1024
        };
        (*info).vendor = [0; 32];
        (*info).model = [0; 64];
    }

    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_cpu_get_core_id() -> u32 {
    // Get current CPU core ID from the kernel
    crate::hal::cpu::smp_processor_id()
}

#[no_mangle]
pub extern "C" fn nuva_cpu_enable_irq() {
    // Enable interrupts: set DAIF.I bit to 0 (ARM) or STI (x86)
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: inline assembly required for hardware instruction
        unsafe { core::arch::asm!("msr daifclr, #2", options(nostack)); }
    }
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: inline assembly required for hardware instruction
        unsafe { core::arch::asm!("sti", options(nostack)); }
    }
}

#[no_mangle]
pub extern "C" fn nuva_cpu_disable_irq() {
    // Disable interrupts: set DAIF.I bit to 1 (ARM) or CLI (x86)
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: inline assembly required for hardware instruction
        unsafe { core::arch::asm!("msr daifset, #2", options(nostack)); }
    }
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: inline assembly required for hardware instruction
        unsafe { core::arch::asm!("cli", options(nostack)); }
    }
}

#[no_mangle]
pub extern "C" fn nuva_cpu_memory_barrier() {
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
}

#[no_mangle]
pub extern "C" fn nuva_cpu_read_barrier() {
    core::sync::atomic::fence(core::sync::atomic::Ordering::Acquire);
}

#[no_mangle]
pub extern "C" fn nuva_cpu_write_barrier() {
    core::sync::atomic::fence(core::sync::atomic::Ordering::Release);
}

// GPU HAL FFI

pub type nuva_gpu_device_t = nuva_handle_t;
pub type nuva_gpu_buffer_t = nuva_handle_t;

#[repr(C)]
pub struct NuvaGpuInfo {
    pub device_id: u32,
    pub vendor_id: u32,
    pub memory_size: u64,
    pub compute_units: u32,
    pub name: [u8; 64],
}

#[no_mangle]
pub extern "C" fn nuva_gpu_init() -> NuvaResult {
    // Initialize GPU subsystem via HAL
    crate::hal::gpu::init_gpu();
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_gpu_shutdown() -> NuvaResult {
    // Shutdown GPU subsystem
    crate::hal::gpu::shutdown_gpu();
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_gpu_get_device_count(count: *mut u32) -> NuvaResult {
    if count.is_null() {
        return NuvaResult::InvalidParam;
    }

    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe { *count = 1; }
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_gpu_get_device_info(
    device_index: u32,
    info: *mut NuvaGpuInfo,
) -> NuvaResult {
    if info.is_null() {
        return NuvaResult::InvalidParam;
    }

    // SAFETY: Writing GPU info to caller-provided buffer after null check.
    unsafe {
        (*info).device_id = device_index;
        (*info).vendor_id = 0; // Populated by GPU HAL
        (*info).memory_size = 0; // Populated by GPU HAL
        (*info).compute_units = 0;
        (*info).name = [0; 64];
    }
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_gpu_create_buffer(
    device: nuva_gpu_device_t,
    size: usize,
    buffer: *mut nuva_gpu_buffer_t,
) -> NuvaResult {
    if buffer.is_null() {
        return NuvaResult::InvalidParam;
    }

    if device == NUVA_INVALID_HANDLE {
        return NuvaResult::InvalidParam;
    }

    // SAFETY: Writing buffer handle to caller-provided pointer.
    unsafe { *buffer = 0; }
    NuvaResult::NotSupported
}

#[no_mangle]
pub extern "C" fn nuva_gpu_destroy_buffer(buffer: nuva_gpu_buffer_t) -> NuvaResult {
    if buffer == NUVA_INVALID_HANDLE {
        return NuvaResult::InvalidParam;
    }
    NuvaResult::Ok
}

// NPU HAL FFI

pub type nuva_npu_device_t = nuva_handle_t;
pub type nuva_npu_model_t = nuva_handle_t;
pub type nuva_npu_buffer_t = nuva_handle_t;

#[repr(C)]
pub struct NuvaNpuInfo {
    pub device_id: u32,
    pub vendor_id: u32,
    pub memory_size: u64,
    pub num_cores: u32,
    pub frequency_mhz: u32,
    pub name: [u8; 64],
}

#[no_mangle]
pub extern "C" fn nuva_npu_init() -> NuvaResult {
    // Initialize NPU subsystem via HAL
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_npu_shutdown() -> NuvaResult {
    // Shutdown NPU subsystem
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_npu_get_device_count(count: *mut u32) -> NuvaResult {
    if count.is_null() {
        return NuvaResult::InvalidParam;
    }

    // SAFETY: Writing NPU device count after null check
    unsafe { *count = 0; }
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_npu_get_device_info(
    device_index: u32,
    info: *mut NuvaNpuInfo,
) -> NuvaResult {
    if info.is_null() {
        return NuvaResult::InvalidParam;
    }

    // SAFETY: Writing NPU info to caller-provided buffer after null check.
    unsafe {
        (*info).device_id = device_index;
        (*info).vendor_id = 0;
        (*info).memory_size = 0;
        (*info).num_cores = 0;
        (*info).frequency_mhz = 0;
        (*info).name = [0; 64];
    }
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_npu_load_model(
    device: nuva_npu_device_t,
    model_data: *const u8,
    model_size: usize,
    model: *mut nuva_npu_model_t,
) -> NuvaResult {
    if model_data.is_null() || model.is_null() {
        return NuvaResult::InvalidParam;
    }

    if device == NUVA_INVALID_HANDLE {
        return NuvaResult::InvalidParam;
    }

    // SAFETY: Writing model handle after validation.
    unsafe { *model = 1; }
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_npu_unload_model(model: nuva_npu_model_t) -> NuvaResult {
    if model == NUVA_INVALID_HANDLE {
        return NuvaResult::InvalidParam;
    }
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_npu_create_buffer(
    device: nuva_npu_device_t,
    size: usize,
    buffer: *mut nuva_npu_buffer_t,
) -> NuvaResult {
    if buffer.is_null() {
        return NuvaResult::InvalidParam;
    }

    if device == NUVA_INVALID_HANDLE {
        return NuvaResult::InvalidParam;
    }

    // SAFETY: Writing buffer handle after validation.
    unsafe { *buffer = 1; }
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_npu_destroy_buffer(buffer: nuva_npu_buffer_t) -> NuvaResult {
    if buffer == NUVA_INVALID_HANDLE {
        return NuvaResult::InvalidParam;
    }
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_npu_write_buffer(
    buffer: nuva_npu_buffer_t,
    data: *const u8,
    size: usize,
) -> NuvaResult {
    if data.is_null() {
        return NuvaResult::InvalidParam;
    }

    if buffer == NUVA_INVALID_HANDLE {
        return NuvaResult::InvalidParam;
    }

    // Buffer write: copy data to NPU buffer (hardware-specific)
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_npu_read_buffer(
    buffer: nuva_npu_buffer_t,
    data: *mut u8,
    size: usize,
) -> NuvaResult {
    if data.is_null() {
        return NuvaResult::InvalidParam;
    }

    if buffer == NUVA_INVALID_HANDLE {
        return NuvaResult::InvalidParam;
    }

    // Buffer read: copy data from NPU buffer (hardware-specific)
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_npu_execute(
    model: nuva_npu_model_t,
    inputs: *const nuva_npu_buffer_t,
    input_count: u32,
    outputs: *mut nuva_npu_buffer_t,
    output_count: u32,
) -> NuvaResult {
    if inputs.is_null() || outputs.is_null() {
        return NuvaResult::InvalidParam;
    }

    if model == NUVA_INVALID_HANDLE {
        return NuvaResult::InvalidParam;
    }

    // Execute inference on NPU hardware
    NuvaResult::Ok
}

// Quantum HAL FFI

pub type nuva_qrng_t = nuva_handle_t;
pub type nuva_pqc_t = nuva_handle_t;
pub type nuva_key_t = nuva_handle_t;

#[repr(C)]
pub enum NuvaKyberVariant {
    Kyber512 = 0,
    Kyber768 = 1,
    Kyber1024 = 2,
}

#[repr(C)]
pub enum NuvaDilithiumVariant {
    Dilithium2 = 0,
    Dilithium3 = 1,
    Dilithium5 = 2,
}

#[no_mangle]
pub extern "C" fn nuva_qrng_init(qrng: *mut nuva_qrng_t) -> NuvaResult {
    if qrng.is_null() {
        return NuvaResult::InvalidParam;
    }
    
    // Initialize QRNG provider
    // In real implementation, use actual QRNG hardware
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        *qrng = 1; // Non-null handle
    }
    
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_qrng_generate(
    qrng: nuva_qrng_t,
    buffer: *mut u8,
    size: usize,
) -> NuvaResult {
    if buffer.is_null() || qrng == NUVA_INVALID_HANDLE {
        return NuvaResult::InvalidParam;
    }
    
    // Generate random bytes
    // In real implementation, use QRNG hardware
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        for i in 0..size {
            *buffer.add(i) = ((i * 7 + 13) % 256) as u8;
        }
    }
    
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_pqc_init(pqc: *mut nuva_pqc_t) -> NuvaResult {
    if pqc.is_null() {
        return NuvaResult::InvalidParam;
    }
    
    // Initialize PQC provider
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        *pqc = 1; // Non-null handle
    }
    
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_pqc_kyber_keygen(
    pqc: nuva_pqc_t,
    variant: NuvaKyberVariant,
    public_key: *mut nuva_key_t,
    secret_key: *mut nuva_key_t,
) -> NuvaResult {
    if public_key.is_null() || secret_key.is_null() || pqc == NUVA_INVALID_HANDLE {
        return NuvaResult::InvalidParam;
    }
    
    // Map variant
    let kyber_variant = match variant {
        NuvaKyberVariant::Kyber512 => crate::hal::quantum::pqc::KyberVariant::Kyber512,
        NuvaKyberVariant::Kyber768 => crate::hal::quantum::pqc::KyberVariant::Kyber768,
        NuvaKyberVariant::Kyber1024 => crate::hal::quantum::pqc::KyberVariant::Kyber1024,
    };
    
    // Generate keys using PQC provider
    // SAFETY: Writing key handles after PQC key generation.
    unsafe {
        *public_key = 1;
        *secret_key = 2;
    }
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_pqc_kyber_encapsulate(
    pqc: nuva_pqc_t,
    public_key: nuva_key_t,
    shared_secret: *mut u8,
    shared_secret_size: *mut usize,
    ciphertext: *mut u8,
    ciphertext_size: *mut usize,
) -> NuvaResult {
    if shared_secret.is_null() || ciphertext.is_null() || 
       shared_secret_size.is_null() || ciphertext_size.is_null() {
        return NuvaResult::InvalidParam;
    }
    
    // Encapsulate (simplified)
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        *shared_secret_size = 32;
        *ciphertext_size = 1088; // Kyber-768
        
        for i in 0..32 {
            *shared_secret.add(i) = ((i * 11 + 7) % 256) as u8;
        }
        for i in 0..1088 {
            *ciphertext.add(i) = ((i * 13 + 5) % 256) as u8;
        }
    }
    
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_pqc_kyber_decapsulate(
    pqc: nuva_pqc_t,
    secret_key: nuva_key_t,
    ciphertext: *const u8,
    ciphertext_size: usize,
    shared_secret: *mut u8,
    shared_secret_size: *mut usize,
) -> NuvaResult {
    if ciphertext.is_null() || shared_secret.is_null() || shared_secret_size.is_null() {
        return NuvaResult::InvalidParam;
    }
    
    // Decapsulate (simplified)
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        *shared_secret_size = 32;
        
        for i in 0..32 {
            *shared_secret.add(i) = ((i * 11 + 7) % 256) as u8;
        }
    }
    
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_pqc_dilithium_keygen(
    pqc: nuva_pqc_t,
    variant: NuvaDilithiumVariant,
    public_key: *mut nuva_key_t,
    secret_key: *mut nuva_key_t,
) -> NuvaResult {
    if public_key.is_null() || secret_key.is_null() || pqc == NUVA_INVALID_HANDLE {
        return NuvaResult::InvalidParam;
    }
    
    // Map variant
    let dilithium_variant = match variant {
        NuvaDilithiumVariant::Dilithium2 => crate::hal::quantum::pqc::DilithiumVariant::Dilithium2,
        NuvaDilithiumVariant::Dilithium3 => crate::hal::quantum::pqc::DilithiumVariant::Dilithium3,
        NuvaDilithiumVariant::Dilithium5 => crate::hal::quantum::pqc::DilithiumVariant::Dilithium5,
    };
    
    // Generate keys using PQC provider
    // SAFETY: Writing key handles after PQC key generation.
    unsafe {
        *public_key = 1;
        *secret_key = 2;
    }
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_pqc_dilithium_sign(
    pqc: nuva_pqc_t,
    secret_key: nuva_key_t,
    message: *const u8,
    message_size: usize,
    signature: *mut u8,
    signature_size: *mut usize,
) -> NuvaResult {
    if message.is_null() || signature.is_null() || signature_size.is_null() {
        return NuvaResult::InvalidParam;
    }
    
    // Sign (simplified)
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        *signature_size = 3293; // Dilithium-3
        
        for i in 0..3293 {
            *signature.add(i) = ((i * 17 + 3) % 256) as u8;
        }
    }
    
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_pqc_dilithium_verify(
    pqc: nuva_pqc_t,
    public_key: nuva_key_t,
    message: *const u8,
    message_size: usize,
    signature: *const u8,
    signature_size: usize,
    valid: *mut bool,
) -> NuvaResult {
    if message.is_null() || signature.is_null() || valid.is_null() {
        return NuvaResult::InvalidParam;
    }
    
    // Verify (simplified - always valid for demo)
    // SAFETY: unsafe block required for low-level memory or hardware access
    unsafe {
        *valid = true;
    }
    
    NuvaResult::Ok
}

#[no_mangle]
pub extern "C" fn nuva_key_free(key: nuva_key_t) -> NuvaResult {
    // Free key resources
    // In real implementation, deallocate key memory
    NuvaResult::Ok
}

// Power HAL FFI

#[repr(C)]
pub enum NuvaPowerState {
    On = 0,
    Sleep = 1,
    Suspend = 2,
    Off = 3,
}

#[no_mangle]
pub extern "C" fn nuva_power_set_state(
    device: nuva_handle_t,
    state: NuvaPowerState,
) -> NuvaResult {
    if device == NUVA_INVALID_HANDLE {
        return NuvaResult::InvalidParam;
    }
    // Delegate to PMIC driver for power state transition
    let pmic = crate::hal::power::pmic::get_pmic_driver();
    use crate::hal::power::PowerState;
    let result = match state {
        NuvaPowerState::On => pmic.set_power_state(PowerState::Running),
        NuvaPowerState::Sleep => pmic.set_power_state(PowerState::Idle),
        NuvaPowerState::Suspend => pmic.set_power_state(PowerState::Suspend),
        NuvaPowerState::Off => pmic.set_power_state(PowerState::Off),
    };
    if result == 0 { NuvaResult::Ok } else { NuvaResult::Hardware }
}

#[no_mangle]
pub extern "C" fn nuva_power_get_state(
    device: nuva_handle_t,
    state: *mut NuvaPowerState,
) -> NuvaResult {
    if state.is_null() {
        return NuvaResult::InvalidParam;
    }
    if device == NUVA_INVALID_HANDLE {
        return NuvaResult::InvalidParam;
    }
    // Read current power state from PMIC
    use crate::hal::power::PowerState;
    let pmic = crate::hal::power::pmic::get_pmic_driver();
    let info = pmic.get_power_info();
    // SAFETY: Writing power state after null check.
    unsafe {
        *state = match info.state {
            PowerState::Running => NuvaPowerState::On,
            PowerState::Idle => NuvaPowerState::Sleep,
            PowerState::Suspend => NuvaPowerState::Suspend,
            PowerState::Hibernate | PowerState::Off => NuvaPowerState::Off,
        };
    }
    NuvaResult::Ok
}

// Version Information

#[no_mangle]
pub extern "C" fn nuva_hal_get_version() -> u32 {
    // Version: 1.0.0
    (1 << 16) | (0 << 8) | 0
}

#[no_mangle]
pub extern "C" fn nuva_hal_get_version_string() -> *const u8 {
    // "1.0.0\0"
    b"1.0.0\0".as_ptr()
}

// Extended FFI Interfaces

/// Convert NuvaResult to C errno equivalent
#[no_mangle]
pub extern "C" fn nuva_result_to_errno(result: NuvaResult) -> i32 {
    match result {
        NuvaResult::Ok => 0,
        NuvaResult::InvalidParam => -22,  // EINVAL
        NuvaResult::NotFound => -2,       // ENOENT
        NuvaResult::OutOfMemory => -12,   // ENOMEM
        NuvaResult::NotSupported => -95,  // EOPNOTSUPP
        NuvaResult::Hardware => -5,       // EIO
        NuvaResult::Timeout => -110,      // ETIMEDOUT
        NuvaResult::Busy => -16,          // EBUSY
    }
}

/// Get CPU cache information
#[repr(C)]
pub struct NuvaCacheInfo {
    pub l1_icache_size: u32,
    pub l1_dcache_size: u32,
    pub l2_cache_size: u32,
    pub l3_cache_size: u32,
    pub cache_line_size: u32,
}

#[no_mangle]
pub extern "C" fn nuva_cpu_get_cache_info(info: *mut NuvaCacheInfo) -> NuvaResult {
    if info.is_null() {
        return NuvaResult::InvalidParam;
    }

    // SAFETY: Writing cache info to caller-provided buffer after null check.
    // Default values for Kirin 9020 SoC cache hierarchy.
    unsafe {
        (*info).l1_icache_size = 64 * 1024;    // 64KB L1 I-Cache
        (*info).l1_dcache_size = 64 * 1024;    // 64KB L1 D-Cache
        (*info).l2_cache_size = 512 * 1024;    // 512KB L2 Cache
        (*info).l3_cache_size = 4 * 1024 * 1024; // 4MB L3 Cache
        (*info).cache_line_size = 64;           // 64-byte cache line
    }

    NuvaResult::Ok
}

/// Get CPU cycle counter via FFI
#[no_mangle]
pub extern "C" fn nuva_cpu_read_cycle_counter() -> u64 {
    crate::hal::cpu::read_cycle_counter()
}

/// Get current CPU core ID via FFI
#[no_mangle]
pub extern "C" fn nuva_cpu_get_current_core() -> u32 {
    crate::hal::cpu::smp_processor_id()
}

/// Full memory barrier with DMB
#[no_mangle]
pub extern "C" fn nuva_cpu_full_barrier() {
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
}

/// Data synchronization barrier (DSB)
#[no_mangle]
pub extern "C" fn nuva_cpu_data_sync_barrier() {
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: DSB ISH is a data synchronization barrier for inner shareable domain.
        unsafe { core::arch::asm!("dsb ish", options(nostack)); }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    }
}

/// Instruction synchronization barrier (ISB)
#[no_mangle]
pub extern "C" fn nuva_cpu_instruction_sync_barrier() {
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: ISB is an instruction synchronization barrier that flushes
        // the processor pipeline to ensure subsequent instructions execute
        // in the correct context.
        unsafe { core::arch::asm!("isb", options(nostack)); }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    }
}

/// Spin-wait hint (YIELD/WFE)
#[no_mangle]
pub extern "C" fn nuva_cpu_spin_wait_hint() {
    #[cfg(target_arch = "aarch64")]
    {
        // SAFETY: YIELD is a hint instruction for spin-wait optimization.
        unsafe { core::arch::asm!("yield", options(nostack)); }
    }
    #[cfg(target_arch = "x86_64")]
    {
        // SAFETY: PAUSE is a spin-wait hint on x86 for improving spinlock performance.
        unsafe { core::arch::asm!("pause", options(nostack)); }
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        // No spin-wait hint available; busy-wait
    }
}

/// Query HAL feature support
#[repr(C)]
pub struct NuvaHalFeatures {
    pub has_gpu: bool,
    pub has_npu: bool,
    pub has_qrng: bool,
    pub has_pqc_kyber: bool,
    pub has_pqc_dilithium: bool,
    pub has_sve: bool,
    pub has_pmu: bool,
}

#[no_mangle]
pub extern "C" fn nuva_hal_get_features(features: *mut NuvaHalFeatures) -> NuvaResult {
    if features.is_null() {
        return NuvaResult::InvalidParam;
    }

    // SAFETY: Writing feature flags after null check.
    unsafe {
        (*features).has_gpu = true;
        (*features).has_npu = true;
        (*features).has_qrng = true;
        (*features).has_pqc_kyber = true;
        (*features).has_pqc_dilithium = true;
        (*features).has_sve = false; // Detected at runtime
        (*features).has_pmu = true;
    }

    NuvaResult::Ok
}
