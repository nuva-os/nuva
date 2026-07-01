/*
 * Nuva OS - Kernel - Core - CrossLevel
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
 * Nuva OS - Kernel - Cross-Level Memory Access Enforcement
 *
 * Copyright (C) 2026 Nuva OS Team
 *
 * Enforces memory isolation between three privilege levels.
 * Direct cross-level memory access is always denied.
 * Only controlled paths are allowed:
 *   EL1→EL2: NvSupervisorCall (cap-controlled)
 *   EL1↔EL0: NvIPC port message passing
 *   EL2→EL1/EL0: NvIPC port (kernel-mediated delivery)
 *
 * INVARIANT: source_level ≠ target_level → enforce_memory_isolation == CrossLevelAccessDenied
 */

use crate::kernel::types::NvPrivilegeLevel;
use crate::kernel::error::{KernelError, KernelResult};
use alloc::vec::Vec;

/// Cross-level access type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NvCrossLevelAccessType {
    /// Direct memory read/write
    DirectMemoryAccess,
    /// Through NvSupervisorCall (allowed EL1→EL2)
    SupervisorCall,
    /// Through NvIPC port message (allowed EL1↔EL0, EL2→EL1/EL0)
    IpcMessage,
}

/// NvCrossLevelAccessEnforcement: enforces memory isolation between privilege levels
///
/// INVARIANT: source_level ≠ target_level → enforce == CrossLevelAccessDenied
pub struct NvCrossLevelAccessEnforcement;

impl NvCrossLevelAccessEnforcement {
    /// Enforce memory isolation between privilege levels.
    ///
    /// Direct cross-level memory access is always denied.
    /// Only controlled paths (SupervisorCall, IpcMessage) are allowed.
    ///
    /// INVARIANT: source_level ≠ target_level → always returns CrossLevelAccessDenied for DirectMemoryAccess
    pub fn enforce_memory_isolation(
        source_level: NvPrivilegeLevel,
        target_level: NvPrivilegeLevel,
        access_type: NvCrossLevelAccessType,
    ) -> KernelResult<()> {
        if source_level == target_level {
            return Ok(());
        }

        match access_type {
            NvCrossLevelAccessType::DirectMemoryAccess => {
                Err(KernelError::CrossLevelAccessDenied)
            }
            NvCrossLevelAccessType::SupervisorCall => {
                if source_level == NvPrivilegeLevel::EquipmentMode
                    && target_level == NvPrivilegeLevel::KernelMode
                {
                    Ok(())
                } else {
                    Err(KernelError::CrossLevelAccessDenied)
                }
            }
            NvCrossLevelAccessType::IpcMessage => {
                match (source_level, target_level) {
                    (NvPrivilegeLevel::EquipmentMode, NvPrivilegeLevel::UserMode)
                    | (NvPrivilegeLevel::UserMode, NvPrivilegeLevel::EquipmentMode)
                    | (NvPrivilegeLevel::KernelMode, NvPrivilegeLevel::EquipmentMode)
                    | (NvPrivilegeLevel::KernelMode, NvPrivilegeLevel::UserMode) => Ok(()),
                    _ => Err(KernelError::CrossLevelAccessDenied),
                }
            }
        }
    }

    /// Check if a specific cross-level path is allowed
    pub fn is_path_allowed(
        source_level: NvPrivilegeLevel,
        target_level: NvPrivilegeLevel,
        access_type: NvCrossLevelAccessType,
    ) -> bool {
        Self::enforce_memory_isolation(source_level, target_level, access_type).is_ok()
    }

    /// Get the allowed access types for a cross-level transition
    pub fn allowed_access_types(
        source_level: NvPrivilegeLevel,
        target_level: NvPrivilegeLevel,
    ) -> alloc::vec::Vec<NvCrossLevelAccessType> {
        let mut allowed = alloc::vec::Vec::new();
        for atype in &[
            NvCrossLevelAccessType::DirectMemoryAccess,
            NvCrossLevelAccessType::SupervisorCall,
            NvCrossLevelAccessType::IpcMessage,
        ] {
            if Self::is_path_allowed(source_level, target_level, *atype) {
                allowed.push(*atype);
            }
        }
        allowed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_level_access_always_allowed() {
        assert!(NvCrossLevelAccessEnforcement::enforce_memory_isolation(
            NvPrivilegeLevel::KernelMode, NvPrivilegeLevel::KernelMode,
            NvCrossLevelAccessType::DirectMemoryAccess
        ).is_ok());
        assert!(NvCrossLevelAccessEnforcement::enforce_memory_isolation(
            NvPrivilegeLevel::UserMode, NvPrivilegeLevel::UserMode,
            NvCrossLevelAccessType::DirectMemoryAccess
        ).is_ok());
    }

    #[test]
    fn test_cross_level_direct_always_denied() {
        assert_eq!(
            NvCrossLevelAccessEnforcement::enforce_memory_isolation(
                NvPrivilegeLevel::UserMode, NvPrivilegeLevel::KernelMode,
                NvCrossLevelAccessType::DirectMemoryAccess
            ),
            Err(KernelError::CrossLevelAccessDenied)
        );
        assert_eq!(
            NvCrossLevelAccessEnforcement::enforce_memory_isolation(
                NvPrivilegeLevel::EquipmentMode, NvPrivilegeLevel::KernelMode,
                NvCrossLevelAccessType::DirectMemoryAccess
            ),
            Err(KernelError::CrossLevelAccessDenied)
        );
    }

    #[test]
    fn test_supervisor_call_only_el1_to_el2() {
        assert!(NvCrossLevelAccessEnforcement::enforce_memory_isolation(
            NvPrivilegeLevel::EquipmentMode, NvPrivilegeLevel::KernelMode,
            NvCrossLevelAccessType::SupervisorCall
        ).is_ok());
        assert_eq!(
            NvCrossLevelAccessEnforcement::enforce_memory_isolation(
                NvPrivilegeLevel::UserMode, NvPrivilegeLevel::KernelMode,
                NvCrossLevelAccessType::SupervisorCall
            ),
            Err(KernelError::CrossLevelAccessDenied)
        );
    }

    #[test]
    fn test_ipc_message_allowed_paths() {
        assert!(NvCrossLevelAccessEnforcement::enforce_memory_isolation(
            NvPrivilegeLevel::EquipmentMode, NvPrivilegeLevel::UserMode,
            NvCrossLevelAccessType::IpcMessage
        ).is_ok());
        assert!(NvCrossLevelAccessEnforcement::enforce_memory_isolation(
            NvPrivilegeLevel::UserMode, NvPrivilegeLevel::EquipmentMode,
            NvCrossLevelAccessType::IpcMessage
        ).is_ok());
        assert!(NvCrossLevelAccessEnforcement::enforce_memory_isolation(
            NvPrivilegeLevel::KernelMode, NvPrivilegeLevel::UserMode,
            NvCrossLevelAccessType::IpcMessage
        ).is_ok());
    }

    #[test]
    fn test_user_cannot_supervisor_call_to_kernel() {
        assert_eq!(
            NvCrossLevelAccessEnforcement::enforce_memory_isolation(
                NvPrivilegeLevel::UserMode, NvPrivilegeLevel::KernelMode,
                NvCrossLevelAccessType::SupervisorCall
            ),
            Err(KernelError::CrossLevelAccessDenied)
        );
    }
}
