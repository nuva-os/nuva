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
