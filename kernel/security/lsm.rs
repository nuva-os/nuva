/*
 * Nuva OS - Kernel - Security - Lsm
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
 * Nuva OS - Kernel - NSM Native Security Module
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * NSM (Nuva Security Module) native entry point.
 * Provides composable security framework via SecurityHook trait,
 * CapSet capability model, and Credentials for subject identity.
 */

// Re-export from submodules
pub use super::capability::{CapSet, cap};
pub use super::credential::Credentials;
pub use super::nsm_manager::{SecId, SecStats, SecurityManager as NsmSecurityManager};
pub use super::nsm_manager::{capable, has_capability, get_security_manager, init_security};
pub use super::security_hook::{SecurityHook, SecurityModule};
