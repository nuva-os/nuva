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

// Nuva OS - Kernel - Driver Adapter Module
// C ABI adapters for vendor driver integration.

// Re-export print macros from crate root
pub use crate::{pr_alert, pr_crit, pr_debug, pr_emerg, pr_err, pr_info, pr_notice, pr_warn};
pub mod c_abi;

pub use c_abi::{
    CCallbackTable, CDeviceClass, CDeviceContext, CDriverAdapter, CDriverInfo, CDriverOps,
    DDF_ABI_VERSION, MAX_DEVICE_NAME, MAX_DRIVER_NAME,
};
