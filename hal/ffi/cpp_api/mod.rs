/*
 * Nuva OS - HAL - FFI - C++ API
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

// C++ API bindings for HAL
// These functions wrap the C FFI bindings with C++-compatible types

pub use crate::hal::ffi::c_api::bindings::*;

/// C++ GPU device wrapper
#[repr(C)]
pub struct NuvaGpuDeviceCpp {
    pub handle: nuva_gpu_device_t,
    pub info: NuvaGpuInfo,
}

/// C++ NPU model wrapper
#[repr(C)]
pub struct NuvaNpuModelCpp {
    pub handle: nuva_npu_model_t,
    pub input_size: usize,
    pub output_size: usize,
}

/// C++-compatible GPU initialization
#[no_mangle]
pub extern "C" fn nuva_cpp_gpu_init(device: *mut NuvaGpuDeviceCpp) -> NuvaResult {
    if device.is_null() {
        return NuvaResult::InvalidParam;
    }
    let result = nuva_gpu_init();
    if matches!(result, NuvaResult::Ok) {
        // SAFETY: device pointer validated above
        unsafe {
            (*device).handle = 1;
        }
    }
    result
}

/// C++-compatible NPU model load
#[no_mangle]
pub extern "C" fn nuva_cpp_npu_load_model_cpp(
    device: nuva_npu_device_t,
    model_data: *const u8,
    model_size: usize,
    model: *mut NuvaNpuModelCpp,
) -> NuvaResult {
    if model_data.is_null() || model.is_null() {
        return NuvaResult::InvalidParam;
    }
    // SAFETY: model pointer validated above
    unsafe {
        let mut handle: nuva_npu_model_t = 0;
        let result = nuva_npu_load_model(device, model_data, model_size, &mut handle);
        if matches!(result, NuvaResult::Ok) {
            (*model).handle = handle;
            (*model).input_size = 0;
            (*model).output_size = 0;
        }
        result
    }
}

/// C++-compatible power state set
#[no_mangle]
pub extern "C" fn nuva_cpp_power_set_state(
    device: nuva_handle_t,
    state: NuvaPowerState,
) -> NuvaResult {
    nuva_power_set_state(device, state)
}
